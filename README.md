# Mongoose

```rust
use async_trait::async_trait;
use bson::doc;
use chrono::{DateTime, Utc};
use mongodb::{options::IndexOptions, Database, IndexModel};
use serde::{Deserialize, Serialize};
use mongoose::Model;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub age: u32,
    pub username: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[async_trait]
impl Model for User {
    fn name() -> String {
        "users"
    }
    async fn create_indexes(db: &Database) {
        let username_index = IndexModel::builder()
            .keys(doc! { "username": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        let indexes = [username_index];
        if let Err(err) = db
            .collection::<Self>(Self::collection_name())
            .create_indexes(indexes, None)
            .await
        {
            tracing::error!(
                "error creating {:?} indexes: {:?}",
                Self::collection_name(),
                err
            );
        }
        tracing::debug!("indexes created for {:?}", Self::collection_name());
    }
}

impl Default for User {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Self::generate_id(),
            age: u32::default(),
            username: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
```
