use anyhow::Result;
use async_once::AsyncOnce;
use async_trait::async_trait;
use bson::{doc, Document};
use futures::StreamExt;
use lazy_static::lazy_static;
use mongodb::{
    error::Error as MongoError,
    options::{
        ClientOptions, FindOneAndUpdateOptions, FindOneOptions, FindOptions, ReturnDocument,
    },
    results::{DeleteResult, InsertManyResult, UpdateResult},
    Client, Collection, Database,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::models::user::User;

#[derive(Serialize, Default)]
pub struct ReadQueryOptions {
    pub projection: Option<Document>,
}

#[derive(Serialize, Default)]
pub struct ListQueryOptions {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub sort: Option<Document>,
    pub projection: Option<Document>,
}

#[async_trait]
pub trait Model:
    Serialize + DeserializeOwned + Unpin + Sync + Sized + Send + Default + Clone
{
    fn collection_name<'a>() -> &'a str;
    async fn create_indexes(db: &Database);
    async fn client<'a>() -> &'a Client {
        let client = &POOL.get().await.1;
        client
    }
    async fn collection() -> Collection<Self> {
        POOL.get()
            .await
            .0
            .collection::<Self>(Self::collection_name())
    }
    fn generate_id() -> String {
        use nanoid::nanoid;
        // ~2 million years needed, in order to have a 1% probability of at least one collision.
        // https://zelark.github.io/nano-id-cc/
        let alphabet = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        nanoid!(20, &alphabet)
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
                    return acc;
                });
        // update timestamp
        set_updates.insert("updated_at", chrono::Utc::now());
        document_updates.insert("$set", set_updates);
        // overall document now looks something like:
        // { $set: { "updated_at": Date, ... }, "$inc": { ... }, "$push": { ... } }
        document_updates
    }

    async fn save(&self) -> Result<Self> {
        Self::collection().await.insert_one(self, None).await?;
        Ok(self.clone())
    }

    async fn bulk_insert(docs: Vec<Self>) -> Result<InsertManyResult> {
        Ok(Self::collection().await.insert_many(docs, None).await?)
    }

    async fn read(filter: Document, options: Option<ReadQueryOptions>) -> Option<Self> {
        let opts = match options {
            Some(opts) => Some(
                FindOneOptions::builder()
                    .projection(opts.projection)
                    .build(),
            ),
            None => None,
        };
        match Self::collection().await.find_one(filter, opts).await {
            Ok(result) => result,
            Err(err) => {
                tracing::error!(
                    "error reading {:?} document: {:?}",
                    Self::collection_name(),
                    err
                );
                None
            }
        }
    }

    async fn read_by_id(id: &str, options: Option<ReadQueryOptions>) -> Option<Self> {
        let opts = match options {
            Some(opts) => Some(
                FindOneOptions::builder()
                    .projection(opts.projection)
                    .build(),
            ),
            None => None,
        };
        match Self::collection()
            .await
            .find_one(doc! { "_id": id }, opts)
            .await
        {
            Ok(result) => result,
            Err(err) => {
                tracing::error!(
                    "error reading {:?} document: {:?}",
                    Self::collection_name(),
                    err
                );
                None
            }
        }
    }

    async fn list(filter: Option<Document>, options: Option<ListQueryOptions>) -> Vec<Self> {
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
                        .projection(opts.projection)
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
                    Self::collection_name(),
                    err
                );
                return Vec::new();
            }
        };
        let mut list_result = vec![];
        while let Some(cursor) = result_cursor.next().await {
            match cursor {
                Ok(document) => list_result.push(document),
                Err(err) => {
                    tracing::error!(
                        "error iterating {:?} cursor: {:?}",
                        Self::collection_name(),
                        err
                    );
                    continue;
                }
            }
        }
        list_result
    }

    async fn update(filter: Document, updates: Document) -> Result<Option<Self>, MongoError> {
        Self::collection()
            .await
            .find_one_and_update(
                filter,
                Self::normalize_updates(&updates),
                FindOneAndUpdateOptions::builder()
                    .return_document(ReturnDocument::After)
                    .build(),
            )
            .await
    }

    async fn bulk_update(filter: Document, updates: Document) -> Result<UpdateResult, MongoError> {
        Self::collection()
            .await
            .update_many(filter, Self::normalize_updates(&updates), None)
            .await
    }

    async fn delete(filter: Document) -> Option<Self> {
        match Self::collection()
            .await
            .find_one_and_delete(filter, None)
            .await
        {
            Ok(found) => found,
            Err(err) => {
                tracing::error!(
                    "error deleting {:?} document: {:?}",
                    Self::collection_name(),
                    err
                );
                None
            }
        }
    }

    async fn bulk_delete(filter: Document) -> Option<DeleteResult> {
        match Self::collection().await.delete_many(filter, None).await {
            Ok(found) => Some(found),
            Err(err) => {
                tracing::error!(
                    "error bulk deleting {:?} documents: {:?}",
                    Self::collection_name(),
                    err
                );
                None
            }
        }
    }

    async fn count(filter: Option<Document>) -> u64 {
        match Self::collection().await.count_documents(filter, None).await {
            Ok(count) => count,
            Err(err) => {
                tracing::error!(
                    "error counting {:?} documents: {:?}",
                    Self::collection_name(),
                    err
                );
                0
            }
        }
    }
}

lazy_static! {
    pub static ref POOL: AsyncOnce<(Database, Client)> = AsyncOnce::new(async {
        let mongo_uri = std::env::var("MONGO_URI").map_or(
            "mongodb://localhost:27017/local-database".to_string(),
            |uri| uri,
        );
        let client_options = ClientOptions::parse(mongo_uri).await.map_or_else(
            |err| {
                tracing::error!("error parsing client options {:?}", err);
                std::process::exit(1);
            },
            |opts| opts,
        );
        let client = Client::with_options(client_options).map_or_else(
            |err| {
                tracing::error!("error connecting client: {:?}", err);
                std::process::exit(1);
            },
            |client| client,
        );
        let default_database = client.default_database().map_or_else(
            || {
                tracing::error!("no default database found");
                std::process::exit(1);
            },
            |db| db,
        );
        tracing::info!("connected to {:?}", default_database.name());
        {
            // migrate indexes on connection
            User::create_indexes(&default_database).await;
        }
        (default_database, client)
    });
}
