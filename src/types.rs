use bson::Document;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListOptions {
    pub limit: i64,
    pub skip: u64,
    pub sort: Document,
}

impl Default for ListOptions {
    fn default() -> Self {
        Self {
            limit: 1_000,
            skip: 0,
            sort: Document::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum MongooseError {
    #[error("no document found: {0}")]
    NotFound(String),
    #[error("error inserting document: {0}")]
    InsertOne(String),
    #[error("error bulk inserting documents: {0}")]
    BulkInsert(String),
    #[error("error listing documents: {0}")]
    List(String),
    #[error("error updating document: {0}")]
    Update(String),
    #[error("error bulk updating documents: {0}")]
    BulkUpdate(String),
    #[error("error deleting document: {0}")]
    Delete(String),
    #[error("error bulk deleting documents: {0}")]
    BulkDelete(String),
    #[error("error counting documents: {0}")]
    Count(String),
    #[error("error aggregating documents: {0}")]
    Aggregate(String),
    #[error("error creating indexes: {0}")]
    CreateIndex(String),
}

impl MongooseError {
    pub fn not_found(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR FINDING DOCUMENTS]: {:?}", error);
        Self::NotFound(error.to_string())
    }
    pub fn insert_one(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR INSERTING DOCUMENT]: {:?}", error);
        Self::InsertOne(error.to_string())
    }
    pub fn bulk_insert(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR BULK INSERTING DOCUMENTS]: {:?}", error);
        Self::BulkInsert(error.to_string())
    }
    pub fn list(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR LISTING DOCUMENTS]: {:?}", error);
        Self::List(error.to_string())
    }
    pub fn update(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR UPDATING DOCUMENT]: {:?}", error);
        Self::Update(error.to_string())
    }
    pub fn bulk_update(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR BULK UPDATING DOCUMENTS]: {:?}", error);
        Self::BulkUpdate(error.to_string())
    }
    pub fn delete(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR DELETING DOCUMENT]: {:?}", error);
        Self::Delete(error.to_string())
    }
    pub fn bulk_delete(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR BULK DELETING DOCUMENTS]: {:?}", error);
        Self::BulkDelete(error.to_string())
    }
    pub fn count(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR COUNTING DOCUMENTS]: {:?}", error);
        Self::Count(error.to_string())
    }
    pub fn aggregate(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR AGGREGATING DOCUMENTS]: {:?}", error);
        Self::Aggregate(error.to_string())
    }
    pub fn create_index(error: impl std::error::Error) -> Self {
        tracing::error!("[MONGODB ERROR CREATING INDEX]: {:?}", error);
        Self::CreateIndex(error.to_string())
    }
}
