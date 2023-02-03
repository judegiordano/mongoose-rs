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
    #[error("no {0} document found")]
    NotFound(String),
    #[error("error inserting {0} document")]
    Insert(String),
    #[error("error bulk inserting {0} documents")]
    BulkInsert(String),
    #[error("error listing {0} documents")]
    List(String),
    #[error("error updating {0} document")]
    Update(String),
    #[error("error bulk updating {0} documents")]
    BulkUpdate(String),
    #[error("error deleting {0} document")]
    Delete(String),
    #[error("error bulk deleting {0} documents")]
    BulkDelete(String),
    #[error("error counting {0} documents")]
    Count(String),
    #[error("error aggregating {0} documents")]
    Aggregate(String),
}
