#[cfg(test)]
mod read_tests {
    use anyhow::Result;

    use crate::tests::mock::{
        self,
        post_model::{PopulatedPost, Post},
        user_model::User,
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
                from: "users".to_string(),
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
                from: "users".to_string(),
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
}
