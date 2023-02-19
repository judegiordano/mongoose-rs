#[cfg(test)]
mod create {
    use crate::tests::mock::{self, User};
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
