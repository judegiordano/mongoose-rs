use serde::{Deserialize, Serialize};

use mongoose::{
    async_trait,
    chrono::{DateTime, Utc},
    connection::connect,
    doc,
    mongodb::{options::IndexOptions, Collection, IndexModel},
    Model, Timestamp,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Address {
    pub address: u32,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub apt_number: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    pub age: u32,
    pub address: Address,
    pub example_array: Vec<u32>,
    #[serde(with = "Timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "Timestamp")]
    pub updated_at: DateTime<Utc>,
}

impl Default for User {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Self::generate_id(),
            username: String::new(),
            example_array: Vec::new(),
            address: Address {
                address: u32::default(),
                street: String::new(),
                city: String::new(),
                state: String::new(),
                zip: String::new(),
                country: String::new(),
                apt_number: None,
            },
            age: u32::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
impl Model for User {
    async fn collection() -> Collection<Self> {
        let database = &connect().await.database;
        {
            // migrate indexes
            let username_index = IndexModel::builder()
                .keys(doc! { "username": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build();
            let indexes = [username_index];
            if let Err(err) = database
                .collection::<Self>(&Self::name())
                .create_indexes(indexes, None)
                .await
            {
                tracing::error!("error creating {:?} indexes: {:?}", Self::name(), err);
            }
        }
        database.collection(&Self::name())
    }
}
