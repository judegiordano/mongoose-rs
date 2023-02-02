#[cfg(test)]
mod delete_tests {
    use anyhow::Result;

    use crate::tests::mock::{self, user_model::User};
    use mongoose::{doc, Model};

    #[tokio::test]
    async fn delete_one() -> Result<()> {
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
    async fn bulk_delete() -> Result<()> {
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
