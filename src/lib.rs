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

#[cfg(test)]
mod create {
    use crate::mock::{self, User};
    use crate::types::MongooseError;
    use crate::Model;

    #[tokio::test]
    async fn create_one() -> Result<(), MongooseError> {
        let new_user = mock::user().save().await;
        assert!(new_user.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn bulk_insert() -> Result<(), MongooseError> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        let inserted = User::bulk_insert(&users).await?;
        assert!(inserted.inserted_ids.len() == 5);
        Ok(())
    }

    #[tokio::test]
    async fn create_one_with_relation() -> Result<(), MongooseError> {
        let new_user = mock::user();
        let inserted = new_user.save().await?;
        assert_eq!(inserted.username, new_user.username);
        assert_eq!(inserted.age, new_user.age);
        let new_post = mock::post(inserted.id.to_string());
        let new_post = new_post.save().await?;
        assert_eq!(new_post.id, new_post.id);
        assert_eq!(new_post.user, inserted.id);
        Ok(())
    }
}

#[cfg(test)]
mod read {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::mock::{self, Address, PopulatedPost, Post, User};
    use crate::types::MongooseError;
    use crate::{
        doc,
        types::{ListOptions, LookupStage, PipelineStage},
        Model,
    };

    #[tokio::test]
    async fn read() -> Result<(), MongooseError> {
        let new_user = mock::user().save().await?;
        let user = User::read(doc! { "username": &new_user.username }).await?;
        assert_eq!(user.username, new_user.username);
        Ok(())
    }

    #[tokio::test]
    async fn read_by_id() -> Result<(), MongooseError> {
        let new_user = mock::user().save().await?;
        let user = User::read_by_id(&new_user.id).await?;
        assert_eq!(user.username, new_user.username);
        assert_eq!(user.id, new_user.id);
        Ok(())
    }

    #[tokio::test]
    async fn list() -> Result<(), MongooseError> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        User::bulk_insert(&users).await?;
        let users = User::list(None, None).await?;
        assert_eq!(users.len() > 0, true);
        Ok(())
    }

    #[tokio::test]
    async fn pagination() -> Result<(), MongooseError> {
        let users = (0..10)
            .into_iter()
            .map(|_| mock::user())
            .collect::<Vec<_>>();
        User::bulk_insert(&users).await?;

        let users = User::list(
            None,
            Some(ListOptions {
                limit: Some(10),
                skip: Some(0),
                sort: Some(doc! { "age": 1 }),
                ..Default::default()
            }),
        )
        .await?;
        assert!(users.len() == 10);
        for slice in users.windows(2) {
            assert!(slice[0].age <= slice[1].age);
        }
        Ok(())
    }

    #[tokio::test]
    async fn in_operator() -> Result<(), MongooseError> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        User::bulk_insert(&users).await?;

        let users = User::list(
            None,
            Some(ListOptions {
                limit: Some(2),
                sort: Some(doc! { "created_at": -1 }),
                ..Default::default()
            }),
        )
        .await?;
        let ids = users.iter().map(|a| a.id.to_string()).collect::<Vec<_>>();
        let matches = User::list(
            Some(doc! {
                "_id": { "$in": &ids }
            }),
            None,
        )
        .await?;
        assert!(matches.len() == 2);
        let match_ids = matches.iter().map(|a| a.id.to_string()).collect::<Vec<_>>();
        assert!(match_ids.contains(&ids[0]));
        assert!(match_ids.contains(&ids[1]));
        Ok(())
    }

    #[tokio::test]
    async fn count() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        let count = User::count(Some(doc! { "username": user.username })).await?;
        assert!(count == 1);
        Ok(())
    }

    #[tokio::test]
    async fn match_aggregate() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        let posts = (0..10)
            .into_iter()
            .map(|_| mock::post(user.id.to_string()))
            .collect::<Vec<_>>();
        Post::bulk_insert(&posts).await?;
        let results = Post::aggregate::<PopulatedPost>(&[
            PipelineStage::Match(doc! {
                "user": user.id.to_string()
            }),
            PipelineStage::Lookup(LookupStage {
                from: User::name(),
                foreign_field: "_id".to_string(),
                local_field: "user".to_string(),
                as_field: "user".to_string(),
            }),
            PipelineStage::Unwind("$user".to_string()),
        ])
        .await?;
        assert!(results.len() >= 1);
        results
            .iter()
            .for_each(|post| assert!(post.user.id == user.id));
        Ok(())
    }

    #[tokio::test]
    async fn aggregate_arbitrary() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        Post::bulk_insert(
            &(0..10)
                .into_iter()
                .map(|_| mock::post(user.id.to_string()))
                .collect::<Vec<_>>(),
        )
        .await?;
        // create aggregate
        // this should demonstrate all supported
        // aggregation features into a generic Value
        let results = Post::aggregate::<serde_json::Value>(&[
            PipelineStage::Match(doc! { "user": user.id }),
            PipelineStage::Limit(2),
            PipelineStage::Lookup(LookupStage {
                from: User::name(),
                foreign_field: "_id".to_string(),
                local_field: "user".to_string(),
                as_field: "user".to_string(),
            }),
            PipelineStage::Unwind("$user".to_string()),
            PipelineStage::Project(doc! {
                "content": 1,
                "created_at": 1,
                "user._id": 1,
                "user.username": 1,
                "user.example_array": 1,
            }),
            PipelineStage::AddFields(doc! {
                "array_sum": doc! { "$sum": "$user.example_array" },
                "post_date": "$created_at",
            }),
            PipelineStage::Sort(doc! { "post_date": -1 }),
        ])
        .await?;
        assert!(results.len() == 2);
        Ok(())
    }

    #[tokio::test]
    async fn join_to_many() -> Result<(), MongooseError> {
        use crate::Timestamp;
        #[derive(Debug, Deserialize, Serialize, Clone)]
        struct ShallowPost {
            content: String,
            #[serde(with = "Timestamp")]
            created_at: DateTime<Utc>,
        }
        #[derive(Debug, Deserialize, Serialize, Clone)]
        struct UserPosts {
            #[serde(rename = "_id")]
            id: String,
            username: String,
            age: u32,
            address: Address,
            example_array: Vec<u32>,
            #[serde(with = "Timestamp")]
            created_at: DateTime<Utc>,
            #[serde(with = "Timestamp")]
            updated_at: DateTime<Utc>,
            posts: Vec<ShallowPost>,
        }
        let user = mock::user().save().await?;
        Post::bulk_insert(
            &(0..10)
                .into_iter()
                .map(|_| mock::post(user.id.to_string()))
                .collect::<Vec<_>>(),
        )
        .await?;
        // build aggregate -> populate many to one
        let results = User::aggregate::<UserPosts>(&[
            PipelineStage::Match(doc! { "_id": &user.id }),
            PipelineStage::Lookup(LookupStage {
                from: Post::name(),
                foreign_field: "user".to_string(),
                local_field: "_id".to_string(),
                as_field: "posts".to_string(),
            }),
            PipelineStage::Project(doc! {
                "posts": {
                    "user": 0,
                    "_id": 0,
                    "updated_at": 0
                }
            }),
        ])
        .await?;
        let populated_user = results.first().unwrap();
        assert!(populated_user.id == user.id);
        assert!(populated_user.posts.len() == 10);
        Ok(())
    }
}

#[cfg(test)]
mod update {
    use crate::mock::{self, User};
    use crate::types::MongooseError;
    use crate::{doc, Model};

    #[tokio::test]
    async fn increment() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        let updated = User::update(
            doc! { "_id": &user.id },
            doc! {
                "$inc": { "address.address": 1 }
            },
        )
        .await?;
        assert!(&updated.address.address > &user.address.address);
        Ok(())
    }

    #[tokio::test]
    async fn decrement() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        let updated = User::update(
            doc! { "_id": &user.id },
            doc! {
                "$inc": { "age": -1 }
            },
        )
        .await?;
        assert!(&updated.age < &user.age);
        Ok(())
    }

    #[tokio::test]
    async fn push() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        assert!(user.example_array.len() == 3);
        let updated = User::update(
            doc! { "_id": user.id },
            doc! {
                "$push": { "example_array": 1234 }
            },
        )
        .await?;
        assert!(updated.example_array.len() == 4);
        Ok(())
    }

    #[tokio::test]
    async fn pull() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        assert!(user.example_array.len() == 3);
        let updated = User::update(
            doc! { "_id": user.id },
            doc! {
                "$pull": { "example_array": user.example_array[0] }
            },
        )
        .await?;
        assert!(updated.example_array.len() == 2);
        Ok(())
    }

    #[tokio::test]
    async fn update_sub_document() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        let new_city = mock::nanoid();
        let updated = User::update(
            doc! { "_id": user.id },
            doc! {
                "address.city": &new_city
            },
        )
        .await?;
        assert!(updated.address.city == new_city);
        Ok(())
    }
}

#[cfg(test)]
mod delete {
    use crate::mock::{self, User};
    use crate::types::MongooseError;
    use crate::{doc, Model};

    #[tokio::test]
    async fn delete_one() -> Result<(), MongooseError> {
        let inserted = mock::user().save().await?;
        let found = User::read_by_id(&inserted.id).await?;
        assert_eq!(found.id, inserted.id);
        // delete
        let deleted = User::delete(doc! { "_id": &inserted.id }).await;
        assert!(deleted.is_ok());
        // should not exist
        let found = User::read_by_id(&inserted.id).await;
        assert!(found.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn bulk_delete() -> Result<(), MongooseError> {
        let users = (0..10)
            .into_iter()
            .map(|_| mock::user())
            .collect::<Vec<_>>();
        User::bulk_insert(&users).await?;
        // delete any null address
        User::bulk_delete(doc! {
            "address.apt_number": None::<String>
        })
        .await?;
        let null_addresses = User::list(
            Some(doc! {
                "address.apt_number": None::<String>
            }),
            None,
        )
        .await?;
        assert!(null_addresses.len() == 0);
        Ok(())
    }
}
