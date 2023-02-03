use serde::{Deserialize, Serialize};

use mongoose::{
    async_trait,
    chrono::{DateTime, Utc},
    connection::{connect, Connection},
    doc,
    mongodb::Collection,
    Model, Timestamp,
};

use super::user_model::User;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: String,
    pub user: String,
    pub content: String,
    #[serde(with = "Timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "Timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PopulatedPost {
    #[serde(rename = "_id")]
    pub id: String,
    pub user: User,
    pub content: String,
    #[serde(with = "Timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "Timestamp")]
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
    async fn collection() -> Collection<Self> {
        let Connection { database, .. } = *connect().await;
        {
            // migrate indexes
            Self::create_indexes(&database).await;
        }
        database.collection(&Self::name())
    }
}
