use bson::Document;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

/// This is still under development
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ReadOptions {
    pub projection: Option<Document>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ListOptions {
    pub limit: Option<i64>,
    pub skip: Option<u64>,
    pub sort: Option<Document>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LookupStage {
    pub from: String,
    #[serde(rename = "localField")]
    pub local_field: String,
    #[serde(rename = "foreignField")]
    pub foreign_field: String,
    #[serde(rename = "as")]
    pub as_field: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PipelineStage {
    Match(Document),
    Limit(i64),
    Sort(Document),
    Lookup(LookupStage),
    Unwind(String),
    Project(Document),
    AddFields(Document),
}

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum MongooseError {
    #[error("no document found in {0:#?} collection")]
    NotFound(String),
    #[error("error inserting document in {0:#?} collection")]
    Insert(String),
    #[error("error bulk inserting documents in {0:#?} collection")]
    BulkInsert(String),
    #[error("error listing documents in {0:#?} collection")]
    List(String),
    #[error("error updating document in {0:#?} collection")]
    Update(String),
    #[error("error bulk updating documents in {0:#?} collection")]
    BulkUpdate(String),
    #[error("error deleting document in {0:#?} collection")]
    Delete(String),
    #[error("error bulk deleting documents in {0:#?} collection")]
    BulkDelete(String),
    #[error("error counting documents in {0:#?} collection")]
    Count(String),
    #[error("error aggregating documents in {0:#?} collection")]
    Aggregate(String),
}
