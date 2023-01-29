#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::tests::mock::{self, user_model::User};
    use mongoose::{doc, ListQueryOptions, Model};

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
        let inserted = User::bulk_insert(users).await?;
        assert!(inserted.inserted_ids.len() == 5);
        Ok(())
    }

    #[tokio::test]
    async fn list() -> Result<()> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        User::bulk_insert(users).await?;
        let users = User::list(None, None).await;
        assert_eq!(users.len() > 0, true);
        Ok(())
    }

    #[tokio::test]
    async fn increment() -> Result<()> {
        let users = User::list(
            None,
            Some(ListQueryOptions {
                limit: Some(1),
                sort: Some(doc! { "created_at": -1 }),
                ..Default::default()
            }),
        )
        .await;
        let user = users.first();
        assert_eq!(user.is_some(), true);
        let user = user.unwrap();
        let id = &user.id;
        let current_address = &user.address.address;
        let updated = User::update(
            doc! { "_id": id },
            doc! {
                "$inc": { "address.address": 1 }
            },
        )
        .await?;
        assert!(&updated.unwrap().address.address > &current_address);
        Ok(())
    }

    #[tokio::test]
    async fn delete_one() -> Result<()> {
        let inserted = mock::user().save().await?;
        let found = User::read_by_id(&inserted.id, None).await;
        assert!(found.is_some());
        // delete
        let deleted = User::delete(doc! { "_id": &inserted.id }).await;
        assert!(deleted.is_some());
        // should be deleted
        let found = User::read_by_id(&inserted.id, None).await;
        assert!(found.is_none());
        Ok(())
    }
}
