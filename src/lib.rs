// expose 3rd party crates
pub use async_trait::async_trait;
pub use bson::{doc, serde_helpers::chrono_datetime_as_bson_datetime as Timestamp, Document};
pub use chrono;
pub use mongodb;
// expose crates
pub mod connection;
pub mod types;
// expose model
mod model;
pub use model::Model;

// tests
#[cfg(test)]
mod tests;
