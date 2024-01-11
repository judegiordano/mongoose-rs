use crate::{
    connection::POOL,
    types::{ListOptions, MongooseError},
};
use bson::{doc, Document};
use convert_case::{Case, Casing};
use futures::StreamExt;
use mongodb::{
    options::{CreateCollectionOptions, FindOneAndUpdateOptions, FindOptions, ReturnDocument},
    results::{CreateIndexesResult, DeleteResult, InsertManyResult, UpdateResult},
    Client, Collection, Database, IndexModel,
};
use serde::{de::DeserializeOwned, Serialize};

#[allow(async_fn_in_trait)]
pub trait Model
where
    Self: Serialize + DeserializeOwned + Unpin + Sync + Sized + Send + Default + Clone,
{
    fn client() -> &'static Client {
        &POOL.client
    }
    fn database() -> &'static Database {
        &POOL.database
    }
    fn collection() -> Collection<Self> {
        POOL.database.collection::<Self>(&Self::name())
    }
    async fn create_view(source: impl ToString, pipeline: Vec<Document>) -> bool {
        match Self::database()
            .create_collection(
                Self::name(),
                CreateCollectionOptions::builder()
                    .view_on(source.to_string())
                    .pipeline(pipeline)
                    .build(),
            )
            .await
        {
            Ok(()) => true,
            Err(err) => {
                tracing::error!(
                    "error creating {:?} view: {:?}",
                    Self::name(),
                    err.to_string()
                );
                false
            }
        }
    }

    fn name() -> String {
        let name = std::any::type_name::<Self>();
        name.split("::").last().map_or_else(
            || name.to_string(),
            |name| {
                let mut normalized = name.to_case(Case::Snake);
                if !normalized.ends_with('s') {
                    normalized.push('s');
                }
                normalized
            },
        )
    }

    #[cfg(feature = "uuid")]
    fn generate_uuid() -> bson::Uuid {
        bson::Uuid::new()
    }

    #[cfg(feature = "nanoid")]
    fn generate_nanoid() -> String {
        // ~2 million years needed, in order to have a 1% probability of at least one collision.
        // https://zelark.github.io/nano-id-cc/
        nanoid::nanoid!(
            20,
            &[
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ]
        )
    }

    fn normalize_updates(updates: &Document) -> Document {
        let (mut set_updates, mut document_updates) =
            updates
                .keys()
                .fold((Document::new(), Document::new()), |mut acc, key| {
                    let val = updates.get(key);
                    if val.is_none() || key == "$set" {
                        // $set is built internally, so skip it
                        return acc;
                    }
                    if key.starts_with('$') {
                        // indicates something like $inc / $push / $pull
                        acc.1.insert(key, val);
                    } else {
                        // all other document field updates contained in $set
                        acc.0.insert(key, val);
                    }
                    acc
                });
        // update timestamp
        set_updates.insert("updated_at", bson::DateTime::now());
        document_updates.insert("$set", set_updates);
        // overall document now looks something like:
        // { $set: { "updated_at": Date, ... }, "$inc": { ... }, "$push": { ... } }
        document_updates
    }

    // client api methods
    async fn save(&self) -> Result<Self, MongooseError> {
        Self::collection()
            .insert_one(self, None)
            .await
            .map_err(MongooseError::insert_one)?;
        Ok(self.clone())
    }

    async fn bulk_insert(docs: &[Self]) -> Result<InsertManyResult, MongooseError> {
        Self::collection()
            .insert_many(docs, None)
            .await
            .map_err(MongooseError::bulk_insert)
    }

    async fn read(filter: Document) -> Result<Self, MongooseError> {
        Self::collection()
            .find_one(filter, None)
            .await
            .map_err(MongooseError::not_found)?
            .ok_or_else(|| {
                MongooseError::NotFound("no documents returned matching filter".to_string())
            })
    }

    async fn read_by_id(id: impl ToString + Send) -> Result<Self, MongooseError> {
        Self::read(doc! { "_id": id.to_string() }).await
    }

    #[cfg(feature = "uuid")]
    async fn read_by_uuid(id: impl ToString + Send) -> Result<Self, MongooseError> {
        let id = bson::Uuid::parse_str(id.to_string()).map_err(MongooseError::not_found)?;
        Self::read(doc! { "_id": id }).await
    }

    async fn list(filter: Document, options: ListOptions) -> Result<Vec<Self>, MongooseError> {
        let opts = FindOptions::builder()
            .skip(options.skip)
            .limit(options.limit)
            .sort(options.sort)
            .projection(None)
            .build();
        let mut result_cursor = Self::collection()
            .find(filter, opts)
            .await
            .map_err(MongooseError::list)?;
        let mut list_result = vec![];
        while let Some(cursor) = result_cursor.next().await {
            list_result.push(cursor.map_err(MongooseError::list)?);
        }
        Ok(list_result)
    }

    async fn update(filter: Document, updates: Document) -> Result<Self, MongooseError> {
        Self::collection()
            .find_one_and_update(
                filter,
                Self::normalize_updates(&updates),
                FindOneAndUpdateOptions::builder()
                    .return_document(ReturnDocument::After)
                    .build(),
            )
            .await
            .map_err(MongooseError::update)?
            .ok_or_else(|| {
                MongooseError::NotFound("no documents returned matching filter".to_string())
            })
    }

    async fn bulk_update(
        filter: Document,
        updates: Document,
    ) -> Result<UpdateResult, MongooseError> {
        Self::collection()
            .update_many(filter, Self::normalize_updates(&updates), None)
            .await
            .map_err(MongooseError::bulk_update)
    }

    async fn delete(filter: Document) -> Result<DeleteResult, MongooseError> {
        Self::collection()
            .delete_one(filter, None)
            .await
            .map_err(MongooseError::delete)
    }

    async fn bulk_delete(filter: Document) -> Result<DeleteResult, MongooseError> {
        Self::collection()
            .delete_many(filter, None)
            .await
            .map_err(MongooseError::bulk_delete)
    }

    async fn count(filter: Option<Document>) -> Result<u64, MongooseError> {
        Self::collection()
            .count_documents(filter, None)
            .await
            .map_err(MongooseError::count)
    }

    async fn aggregate<T: DeserializeOwned + Send>(
        pipeline: Vec<Document>,
    ) -> Result<Vec<T>, MongooseError> {
        let mut result_cursor = Self::collection()
            .aggregate(pipeline, None)
            .await
            .map_err(MongooseError::aggregate)?;
        let mut aggregate_docs = vec![];
        while let Some(cursor) = result_cursor.next().await {
            let document = cursor.map_err(MongooseError::aggregate)?;
            let data = bson::from_document::<T>(document)
                .map_err(|err| MongooseError::Aggregate(err.to_string()))?;
            aggregate_docs.push(data);
        }
        Ok(aggregate_docs)
    }

    async fn create_indexes(options: &[IndexModel]) -> Result<CreateIndexesResult, MongooseError> {
        Self::collection()
            .create_indexes(options.to_vec(), None)
            .await
            .map_err(MongooseError::create_index)
    }
}
