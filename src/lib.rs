// expose 3rd party crates
pub use bson;
pub use bson::serde_helpers::chrono_datetime_as_bson_datetime as TimestampSerializer;
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
