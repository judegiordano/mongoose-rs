#[cfg(test)]
mod read_tests {
    use anyhow::Result;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    use crate::tests::mock::{
        self,
        post_model::{PopulatedPost, Post},
        user_model::{Address, User},
    };
    use mongoose::{
        doc,
        types::{ListOptions, LookupStage, PipelineStage},
        Model,
    };

    #[tokio::test]
    async fn read() -> Result<()> {
        let new_user = mock::user().save().await?;
        let user = User::read(doc! { "username": &new_user.username }).await?;
        assert_eq!(user.username, new_user.username);
        Ok(())
    }

    #[tokio::test]
    async fn read_by_id() -> Result<()> {
        let new_user = mock::user().save().await?;
        let user = User::read_by_id(&new_user.id).await?;
        assert_eq!(user.username, new_user.username);
        assert_eq!(user.id, new_user.id);
        Ok(())
    }

    #[tokio::test]
    async fn list() -> Result<()> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        User::bulk_insert(&users).await?;
        let users = User::list(None, None).await?;
        assert_eq!(users.len() > 0, true);
        Ok(())
    }

    #[tokio::test]
    async fn pagination() -> Result<()> {
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
    async fn in_operator() -> Result<()> {
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
    async fn count() -> Result<()> {
        let user = mock::user().save().await?;
        let count = User::count(Some(doc! { "username": user.username })).await?;
        assert!(count == 1);
        Ok(())
    }

    #[tokio::test]
    async fn match_aggregate() -> Result<()> {
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
    async fn aggregate_arbitrary() -> Result<()> {
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
    async fn join_to_many() -> Result<()> {
        use mongoose::Timestamp;
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
