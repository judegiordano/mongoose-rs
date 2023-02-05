use serde::{Deserialize, Serialize};

use mongoose::{
    async_trait,
    chrono::{DateTime, Utc},
    connection::connect,
    doc,
    mongodb::{Collection, IndexModel},
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
        let database = &connect().await.database;
        {
            // migrate indexes
            let user_index = IndexModel::builder().keys(doc! { "user": 1 }).build();
            let indexes = [user_index];
            if let Err(err) = database
                .collection::<Self>(&Self::name())
                .create_indexes(indexes, None)
                .await
            {
                tracing::error!("error creating {:?} indexes: {:?}", Self::name(), err);
            }
            tracing::debug!("indexes created for {:?}", Self::name());
        }
        database.collection(&Self::name())
    }
}
