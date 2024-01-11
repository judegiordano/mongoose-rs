#[cfg(test)]
mod read {
    use bson::DateTime;
    use serde::{Deserialize, Serialize};

    use crate::tests::mock::{self, Address, PopulatedPost, Post, User};
    use crate::types::MongooseError;
    use crate::{doc, types::ListOptions, Model};

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
        let users = User::list(Default::default(), Default::default()).await?;
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
            Default::default(),
            ListOptions {
                limit: 10,
                skip: 0,
                sort: doc! { "age": 1 },
                ..Default::default()
            },
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
            Default::default(),
            ListOptions {
                limit: 2,
                sort: doc! { "created_at": -1 },
                ..Default::default()
            },
        )
        .await?;
        let ids = users.iter().map(|a| a.id.to_string()).collect::<Vec<_>>();
        let matches = User::list(
            doc! {
                "_id": { "$in": &ids }
            },
            Default::default(),
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
        let user_one = mock::user().save().await?;
        let user_two = mock::user().save().await?;
        let count = User::count(Some(doc! { "$or": [
            { "username": user_one.username },
            { "username": user_two.username }
        ] }))
        .await?;
        assert!(count == 2);
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
        let pipeline = vec![
            doc! {
                "$match": { "user": user.id.to_string() },
            },
            doc! {
                "$lookup": {
                    "from": User::name(),
                    "localField": "user".to_string(),
                    "foreignField": "_id".to_string(),
                    "as": "user"
                },
            },
            doc! { "$unwind": { "path": "$user".to_string() } },
        ];
        let results = Post::aggregate::<PopulatedPost>(pipeline).await?;
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
        let pipeline = vec![
            doc! {
                "$match": { "user": user.id }
            },
            doc! {
                "$limit": 2
            },
            doc! {
                "$lookup": {
                    "from": User::name(),
                    "localField": "user".to_string(),
                    "foreignField": "_id".to_string(),
                    "as": "user"
                }
            },
            doc! { "$unwind": { "path": "$user".to_string() } },
            doc! { "$project": {
                "content": 1,
                "created_at": 1,
                "user._id": 1,
                "user.username": 1,
                "user.example_array": 1,
            } },
            doc! { "$addFields": {
                "array_sum": doc! { "$sum": "$user.example_array" },
                "post_date": "$created_at",
            } },
            doc! { "$sort": { "post_date": -1 } },
        ];
        let results = Post::aggregate::<serde_json::Value>(pipeline).await?;
        assert!(results.len() == 2);
        Ok(())
    }

    #[tokio::test]
    async fn join_to_many() -> Result<(), MongooseError> {
        #[derive(Debug, Deserialize, Serialize, Clone)]
        struct ShallowPost {
            content: String,
            created_at: DateTime,
        }
        #[derive(Debug, Deserialize, Serialize, Clone)]
        struct UserPosts {
            #[serde(rename = "_id")]
            id: String,
            username: String,
            age: u32,
            address: Address,
            example_array: Vec<u32>,
            posts: Vec<ShallowPost>,
            created_at: DateTime,
            updated_at: DateTime,
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
        let pipeline = vec![
            doc! {
                "$match": { "_id": &user.id }
            },
            doc! {
                "$lookup": {
                    "from": Post::name(),
                    "localField": "_id".to_string(),
                    "foreignField": "user".to_string(),
                    "as": "posts"
                }
            },
            doc! { "$project": {
                "posts": {
                    "user": 0,
                    "_id": 0,
                    "updated_at": 0
                }
            } },
        ];
        let results = User::aggregate::<UserPosts>(pipeline).await?;
        let populated_user = results.first().unwrap();
        assert!(populated_user.id == user.id);
        assert!(populated_user.posts.len() == 10);
        Ok(())
    }

    #[tokio::test]
    async fn raw_aggregate() -> Result<(), MongooseError> {
        let user = mock::user().save().await?;
        let pipeline = vec![doc! {
            "$match": {
                "username": &user.username
            }
        }];
        let found = User::aggregate::<User>(pipeline).await?;
        assert!(found.first().unwrap().username == user.username);
        assert!(found.first().unwrap().id == user.id);
        Ok(())
    }
}
