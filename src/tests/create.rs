#[cfg(test)]
mod create_tests {
    use anyhow::Result;

    use crate::tests::mock::{self, user_model::User};
    use mongoose::Model;

    #[tokio::test]
    async fn create_one() -> Result<()> {
        let new_user = mock::user();
        let inserted = new_user.save().await?;
        assert_eq!(inserted.username, new_user.username);
        assert_eq!(inserted.age, new_user.age);
        Ok(())
    }

    #[tokio::test]
    async fn bulk_insert() -> Result<()> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        let inserted = User::bulk_insert(&users).await?;
        assert!(inserted.inserted_ids.len() == 5);
        Ok(())
    }
}
