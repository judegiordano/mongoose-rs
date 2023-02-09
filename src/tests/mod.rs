pub mod create_tests;
pub mod delete_tests;
pub mod read_tests;
pub mod update_tests;

#[cfg(test)]
mod mock {
    use serde::{Deserialize, Serialize};

    use crate::{
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

    pub fn nanoid() -> String {
        use nanoid::nanoid;
        nanoid!(
            20,
            &[
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5', '6',
                '7', '8', '9', '0',
            ]
        )
    }

    pub fn number() -> u32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(0..99999)
    }

    pub fn user() -> User {
        let bool = number() % 2 == 0;
        User {
            username: format!("username_{}", nanoid()),
            age: number(),
            example_array: (0..=2).map(|_| number()).collect::<Vec<_>>(),
            address: Address {
                address: number(),
                street: "Fake Street Name".to_string(),
                city: "Fake City".to_string(),
                state: "CA".to_string(),
                zip: "F1256".to_string(),
                country: "US".to_string(),
                apt_number: if bool { Some("F35".to_string()) } else { None },
            },
            ..Default::default()
        }
    }

    pub fn post(user_id: String) -> Post {
        Post {
            user: user_id,
            content: format!("here's my post: {}", nanoid()),
            ..Default::default()
        }
    }
}
