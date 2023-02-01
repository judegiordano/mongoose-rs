use anyhow::Result;
use futures::StreamExt;
use mongodb::{
    options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument},
    results::{DeleteResult, InsertManyResult, UpdateResult},
    Client, Collection, Database,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

// expose crates
pub use async_trait::async_trait;
pub use bson::{doc, Document};
pub use chrono;
pub use mongodb;
pub mod connection;
pub mod types;

use connection::POOL;
use types::{ListOptions, PipelineStage};

#[async_trait]
pub trait Model:
    Serialize + DeserializeOwned + Unpin + Sync + Sized + Send + Default + Clone + Debug
{
    fn collection_name<'a>() -> &'a str;
    /// ### In practice, we'd likely want to use a global static pool
    /// ---
    /// ```rs
    ///
    ///  lazy_static! {
    ///    pub static ref POOL: AsyncOnce<(Database, Client)> = AsyncOnce::new(async { connect().await });
    ///  }
    /// //
    ///  async fn client() -> Client {
    ///    POOL.get().await.1
    ///  }
    ///  async fn collection() -> Collection<Self> {
    ///    POOL.get()
    ///    .await.0
    ///    .collection::<Self>(Self::collection_name())
    ///  }
    /// ```
    async fn client() -> Client {
        POOL.get().await.1.clone()
    }
    async fn collection() -> Collection<Self> {
        POOL.get()
            .await
            .0
            .collection::<Self>(Self::collection_name())
    }
    async fn create_indexes(_: &Database) {}
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
    fn create_pipeline(pipeline: &[PipelineStage]) -> Vec<Document> {
        pipeline
            .par_iter()
            .map(|stage| match stage {
                PipelineStage::Match(doc) => doc! { "$match": doc },
                PipelineStage::Lookup(doc) => doc! {
                    "$lookup": doc! {
                        "from": doc.from.to_string(),
                        "localField": doc.local_field.to_string(),
                        "foreignField": doc.foreign_field.to_string(),
                        "as": doc.as_field.to_string()
                    }
                },
                PipelineStage::Project(doc) => doc! { "$project": doc },
                PipelineStage::Unwind(path) => doc! {
                    "$unwind": doc! {
                        "path": path
                    }
                },
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
    async fn save<'a>(&'a self) -> Result<&'a Self> {
        Self::collection().await.insert_one(self, None).await?;
        Ok(self)
    }

    async fn bulk_insert(docs: &[Self]) -> Result<InsertManyResult> {
        Ok(Self::collection().await.insert_many(docs, None).await?)
    }

    async fn read(filter: Document) -> Option<Self> {
        match Self::collection().await.find_one(filter, None).await {
            Ok(result) => result,
            Err(err) => {
                tracing::error!(
                    "error reading {:?} document: {:?}",
                    Self::collection_name(),
                    err.to_string()
                );
                None
            }
        }
    }

    async fn read_by_id(id: &str) -> Option<Self> {
        match Self::collection()
            .await
            .find_one(doc! { "_id": id }, None)
            .await
        {
            Ok(result) => result,
            Err(err) => {
                tracing::error!(
                    "error reading {:?} document: {:?}",
                    Self::collection_name(),
                    err.to_string()
                );
                None
            }
        }
    }

    async fn list(filter: Option<Document>, options: Option<ListOptions>) -> Vec<Self> {
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
                    Self::collection_name(),
                    err.to_string()
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
                        err.to_string()
                    );
                    continue;
                }
            }
        }
        list_result
    }

    async fn update(filter: Document, updates: Document) -> Result<Option<Self>> {
        Ok(Self::collection()
            .await
            .find_one_and_update(
                filter,
                Self::normalize_updates(&updates),
                FindOneAndUpdateOptions::builder()
                    .return_document(ReturnDocument::After)
                    .build(),
            )
            .await?)
    }

    async fn bulk_update(filter: Document, updates: Document) -> Result<UpdateResult> {
        Ok(Self::collection()
            .await
            .update_many(filter, Self::normalize_updates(&updates), None)
            .await?)
    }

    async fn delete(filter: Document) -> Option<DeleteResult> {
        match Self::collection().await.delete_one(filter, None).await {
            Ok(found) => Some(found),
            Err(err) => {
                tracing::error!(
                    "error deleting {:?} document: {:?}",
                    Self::collection_name(),
                    err.to_string()
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
                    err.to_string()
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
                    err.to_string()
                );
                0
            }
        }
    }

    async fn aggregate<T: DeserializeOwned + Send>(pipeline: &[PipelineStage]) -> Vec<T> {
        let pipeline = Self::create_pipeline(pipeline);
        let mut result_cursor = match Self::collection().await.aggregate(pipeline, None).await {
            Ok(cursor) => cursor,
            Err(err) => {
                tracing::error!(
                    "error creating {:?} aggregate cursor: {:?}",
                    Self::collection_name(),
                    err.to_string()
                );
                return Vec::new();
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
                            Self::collection_name(),
                            err.to_string()
                        );
                    }
                },
                Err(err) => {
                    tracing::error!(
                        "error iterating {:?} aggregate cursor: {:?}",
                        Self::collection_name(),
                        err.to_string()
                    );
                }
            }
        }
        aggregate_docs
    }
}
