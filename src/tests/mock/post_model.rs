use async_trait::async_trait;
use bson::doc;
use chrono::{DateTime, Utc};
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};

use mongoose::{connect, Model};

use super::user_model::User;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: String,
    pub user: String,
    pub content: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PopulatedPost {
    #[serde(rename = "_id")]
    pub id: String,
    pub user: User,
    pub content: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

impl Default for Post {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Self::generate_id(),
            user: String::new(),
            content: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
impl Model for Post {
    fn collection_name<'a>() -> &'a str {
        "posts"
    }
    async fn client() -> Client {
        let (_, client) = connect().await;
        client
    }
    async fn collection() -> Collection<Self> {
        let (database, _) = connect().await;
        {
            // migrate indexes
            Self::create_indexes(&database).await;
        }
        database.collection(Self::collection_name())
    }

    async fn create_indexes(_: &Database) {}
}
