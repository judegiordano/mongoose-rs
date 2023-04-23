use async_trait::async_trait;
use bson::{doc, Document};
use convert_case::{Case, Casing};
use futures::StreamExt;
use mongodb::{
    options::{CreateCollectionOptions, FindOneAndUpdateOptions, FindOptions, ReturnDocument},
    results::{CreateIndexesResult, DeleteResult, InsertManyResult, UpdateResult},
    Client, Collection, Database, IndexModel,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use crate::{
    connection::POOL,
    types::{ListOptions, MongooseError, PipelineStage},
};

#[async_trait]
pub trait Model
where
    Self: Serialize + DeserializeOwned + Unpin + Sync + Sized + Send + Default + Clone + Debug,
{
    async fn client() -> &'static Client {
        &POOL.get().await.client
    }
    async fn database() -> &'static Database {
        &POOL.get().await.database
    }
    async fn collection() -> Collection<Self> {
        POOL.get().await.database.collection::<Self>(&Self::name())
    }
    async fn create_view(source: &str, pipeline: Vec<Document>) -> bool {
        let db = Self::database().await;
        match db
            .create_collection(
                Self::name(),
                CreateCollectionOptions::builder()
                    .view_on(source.to_string())
                    .pipeline(pipeline)
                    .build(),
            )
            .await
        {
            Ok(_) => true,
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
    fn generate_nanoid() -> String {
        use nanoid::nanoid;
        // ~2 million years needed, in order to have a 1% probability of at least one collision.
        // https://zelark.github.io/nano-id-cc/
        nanoid!(
            20,
            &[
                'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
                'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
            ]
        )
    }
    fn create_pipeline(pipeline: &[PipelineStage]) -> Vec<Document> {
        pipeline
            .iter()
            .map(|stage| match stage {
                PipelineStage::Match(doc) => doc! { "$match": doc },
                PipelineStage::Lookup(doc) => doc! {
                    "$lookup": {
                        "from": doc.from.to_string(),
                        "localField": doc.local_field.to_string(),
                        "foreignField": doc.foreign_field.to_string(),
                        "as": doc.as_field.to_string()
                    }
                },
                PipelineStage::Project(doc) => doc! { "$project": doc },
                PipelineStage::Unwind(path) => doc! { "$unwind": { "path": path } },
                PipelineStage::AddFields(doc) => doc! { "$addFields": doc },
                PipelineStage::Limit(limit) => doc! { "$limit": limit },
                PipelineStage::Sort(doc) => doc! { "$sort": doc },
            })
            .collect::<Vec<_>>()
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
        set_updates.insert("updated_at", chrono::Utc::now());
        document_updates.insert("$set", set_updates);
        // overall document now looks something like:
        // { $set: { "updated_at": Date, ... }, "$inc": { ... }, "$push": { ... } }
        document_updates
    }

    // client api methods
    async fn save(&self) -> Result<Self, MongooseError> {
        match Self::collection().await.insert_one(self, None).await {
            Ok(_) => Ok(self.clone()),
            Err(err) => {
                tracing::error!(
                    "error inserting {:?} document: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::Insert(Self::name()))
            }
        }
    }

    async fn bulk_insert(docs: &[Self]) -> Result<InsertManyResult, MongooseError> {
        match Self::collection().await.insert_many(docs, None).await {
            Ok(inserted) => Ok(inserted),
            Err(err) => {
                tracing::error!(
                    "error bulk inserting {:?} documents: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::BulkInsert(Self::name()))
            }
        }
    }

    async fn read(filter: Document) -> Result<Self, MongooseError> {
        match Self::collection().await.find_one(filter, None).await {
            Ok(result) => result.map_or_else(
                || Err(MongooseError::NotFound(Self::name())),
                |result| Ok(result),
            ),
            Err(err) => {
                tracing::error!(
                    "error reading {:?} document: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::NotFound(Self::name()))
            }
        }
    }

    async fn read_by_id(id: &str) -> Result<Self, MongooseError> {
        Self::read(doc! { "_id": id }).await
    }

    async fn list(
        filter: Option<Document>,
        options: Option<ListOptions>,
    ) -> Result<Vec<Self>, MongooseError> {
        let opts = match options {
            Some(opts) => {
                let limit = if opts.limit.is_some() {
                    opts.limit
                } else {
                    Some(1_000)
                };
                Some(
                    FindOptions::builder()
                        .skip(opts.skip)
                        .limit(limit)
                        .sort(opts.sort)
                        .projection(None)
                        .build(),
                )
            }
            None => None,
        };
        let mut result_cursor = match Self::collection().await.find(filter, opts).await {
            Ok(cursor) => cursor,
            Err(err) => {
                tracing::error!(
                    "error listing {:?} documents: {:?}",
                    Self::name(),
                    err.to_string()
                );
                return Err(MongooseError::List(Self::name()));
            }
        };
        let mut list_result = vec![];
        while let Some(cursor) = result_cursor.next().await {
            match cursor {
                Ok(document) => list_result.push(document),
                Err(err) => {
                    tracing::error!(
                        "error iterating {:?} cursor: {:?}",
                        Self::name(),
                        err.to_string()
                    );
                    continue;
                }
            }
        }
        Ok(list_result)
    }

    async fn update(filter: Document, updates: Document) -> Result<Self, MongooseError> {
        match Self::collection()
            .await
            .find_one_and_update(
                filter,
                Self::normalize_updates(&updates),
                FindOneAndUpdateOptions::builder()
                    .return_document(ReturnDocument::After)
                    .build(),
            )
            .await
        {
            Ok(updated) => updated.map_or_else(
                || Err(MongooseError::NotFound(Self::name())),
                |result| Ok(result),
            ),
            Err(err) => {
                tracing::error!(
                    "error updating {:?} document: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::Update(Self::name()))
            }
        }
    }

    async fn bulk_update(
        filter: Document,
        updates: Document,
    ) -> Result<UpdateResult, MongooseError> {
        match Self::collection()
            .await
            .update_many(filter, Self::normalize_updates(&updates), None)
            .await
        {
            Ok(updates) => Ok(updates),
            Err(err) => {
                tracing::error!(
                    "error updating {:?} documents: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::BulkUpdate(Self::name()))
            }
        }
    }

    async fn delete(filter: Document) -> Result<DeleteResult, MongooseError> {
        match Self::collection().await.delete_one(filter, None).await {
            Ok(found) => Ok(found),
            Err(err) => {
                tracing::error!(
                    "error deleting {:?} document: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::Delete(Self::name()))
            }
        }
    }

    async fn bulk_delete(filter: Document) -> Result<DeleteResult, MongooseError> {
        match Self::collection().await.delete_many(filter, None).await {
            Ok(found) => Ok(found),
            Err(err) => {
                tracing::error!(
                    "error bulk deleting {:?} documents: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::BulkDelete(Self::name()))
            }
        }
    }

    async fn count(filter: Option<Document>) -> Result<u64, MongooseError> {
        match Self::collection().await.count_documents(filter, None).await {
            Ok(count) => Ok(count),
            Err(err) => {
                tracing::error!(
                    "error counting {:?} documents: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::Count(Self::name()))
            }
        }
    }

    async fn aggregate_raw<T: DeserializeOwned + Send>(
        pipeline: Vec<Document>,
    ) -> Result<Vec<T>, MongooseError> {
        let mut result_cursor = match Self::collection().await.aggregate(pipeline, None).await {
            Ok(cursor) => cursor,
            Err(err) => {
                tracing::error!(
                    "error creating {:?} aggregate cursor: {:?}",
                    Self::name(),
                    err.to_string()
                );
                return Err(MongooseError::Aggregate(Self::name()));
            }
        };
        let mut aggregate_docs = vec![];
        while let Some(cursor) = result_cursor.next().await {
            match cursor {
                Ok(document) => match bson::from_document::<T>(document) {
                    Ok(data) => aggregate_docs.push(data),
                    Err(err) => {
                        tracing::error!(
                            "error converting {:?} bson in aggregation: {:?}",
                            Self::name(),
                            err.to_string()
                        );
                        return Err(MongooseError::Aggregate(Self::name()));
                    }
                },
                Err(err) => {
                    tracing::error!(
                        "error iterating {:?} aggregate cursor: {:?}",
                        Self::name(),
                        err.to_string()
                    );
                    return Err(MongooseError::Aggregate(Self::name()));
                }
            }
        }
        Ok(aggregate_docs)
    }

    async fn aggregate<T: DeserializeOwned + Send>(
        pipeline: &[PipelineStage],
    ) -> Result<Vec<T>, MongooseError> {
        let pipeline = Self::create_pipeline(pipeline);
        Self::aggregate_raw::<T>(pipeline).await
    }

    async fn create_indexes(options: &[IndexModel]) -> Result<CreateIndexesResult, MongooseError> {
        match Self::collection()
            .await
            .create_indexes(options.to_vec(), None)
            .await
        {
            Ok(result) => Ok(result),
            Err(err) => {
                tracing::error!(
                    "error creating {:?} indexes: {:?}",
                    Self::name(),
                    err.to_string()
                );
                Err(MongooseError::CreateIndex(Self::name()))
            }
        }
    }
}
