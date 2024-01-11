// expose 3rd party crates
pub use bson::serde_helpers::chrono_datetime_as_bson_datetime as TimestampSerializer;
pub use bson::{doc, DateTime};
pub use mongodb::{options::IndexOptions, IndexModel};

#[cfg(feature = "uuid")]
pub use bson::uuid::Uuid;

// expose crates
pub mod connection;
pub mod types;

// expose model
mod model;
pub use model::Model;

// tests
#[cfg(test)]
mod tests;
