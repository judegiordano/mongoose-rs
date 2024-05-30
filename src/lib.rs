// expose 3rd party crates
pub use bson::doc;
pub use mongodb::{bson::Regex, options::IndexOptions, IndexModel};

// feature exports
#[cfg(feature = "uuid")]
pub use bson::uuid::Uuid;
#[cfg(feature = "timestamps")]
pub use bson::{serde_helpers::chrono_datetime_as_bson_datetime as TimestampSerializer, DateTime};

// expose crates
pub mod connection;
pub mod types;

// expose model
mod model;
pub use model::Model;

// tests
#[cfg(test)]
mod tests;
