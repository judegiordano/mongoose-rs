use bson::Document;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// This is still under development
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ReadQueryOptions {
    pub projection: Option<Document>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ListQueryOptions {
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
