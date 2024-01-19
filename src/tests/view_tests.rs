#[cfg(test)]
mod views {
    use crate::tests::mock::{Address, Post, User};
    use crate::types::MongooseError;
    use crate::{doc, DateTime, IndexModel, Model};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct UserPosts {
        #[serde(rename = "_id")]
        id: String,
        username: String,
        age: u32,
        address: Address,
        example_array: Vec<u32>,
        posts: Vec<Post>,
        created_at: DateTime,
        updated_at: DateTime,
    }

    impl Default for UserPosts {
        fn default() -> Self {
            Self {
                id: Default::default(),
                username: Default::default(),
                age: Default::default(),
                address: Default::default(),
                example_array: Default::default(),
                posts: Default::default(),
                created_at: bson::DateTime::now(),
                updated_at: bson::DateTime::now(),
            }
        }
    }
    impl Model for UserPosts {}

    #[tokio::test]
    async fn create_view() -> Result<(), MongooseError> {
        // create index on post for joining user
        // ix should be used for aggregation pipeline in view
        let indexes = &[IndexModel::builder().keys(doc! { "user": 1 }).build()];
        let created_names = Post::create_indexes(indexes).await?.index_names;
        assert!(created_names.len() > 0);
        // create readonly view
        let pipeline = vec![
            doc! {
                "$lookup": {
                    "from": Post::name(),
                    "localField": "_id",
                    "foreignField": "user",
                    "as": Post::name(),
                }
            },
            doc! {
                "$match": { "posts.0": { "$exists": true } }
            },
        ];
        // this will error if the view exists, but wont panic
        UserPosts::create_view(&User::name(), pipeline).await;
        Ok(())
    }

    #[ignore = "should wait for view to be created"]
    #[tokio::test]
    async fn read_from_view() -> Result<(), MongooseError> {
        let post = UserPosts::read(doc! {}).await;
        assert!(post.is_ok());
        Ok(())
    }
}
