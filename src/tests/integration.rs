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
    async fn read() -> Result<()> {
        let new_user = mock::user().save().await?;
        let user = User::read(doc! { "username": &new_user.username })
            .await
            .unwrap();
        assert_eq!(user.username.clone(), new_user.username.clone());
        Ok(())
    }

    #[tokio::test]
    async fn read_by_id() -> Result<()> {
        let new_user = mock::user().save().await?;
        let user = User::read_by_id(new_user.id.clone()).await.unwrap();
        assert_eq!(user.username.clone(), new_user.username.clone());
        assert_eq!(user.id.clone(), new_user.id.clone());
        Ok(())
    }

    #[tokio::test]
    async fn bulk_insert() -> Result<()> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        let inserted = User::bulk_insert(&users).await?;
        assert!(inserted.inserted_ids.len() == 5);
        Ok(())
    }

    #[tokio::test]
    async fn list() -> Result<()> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        User::bulk_insert(&users).await?;
        let users = User::list(None, None).await;
        assert_eq!(users.len() > 0, true);
        Ok(())
    }

    #[tokio::test]
    async fn increment() -> Result<()> {
        let user = mock::user().save().await?;
        let updated = User::update(
            doc! { "_id": &user.id },
            doc! {
                "$inc": { "address.address": 1 }
            },
        )
        .await?;
        assert!(&updated.unwrap().address.address > &user.address.address);
        Ok(())
    }

    #[tokio::test]
    async fn delete_one() -> Result<()> {
        let inserted = mock::user().save().await?;
        let id = inserted.id.clone();
        let found = User::read_by_id(id.clone()).await;
        assert!(found.is_some());
        // delete
        let deleted = User::delete(doc! { "_id": id }).await;
        assert!(deleted.is_some());
        // should not exist
        let found = User::read_by_id(inserted.id).await;
        assert!(found.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn decrement() -> Result<()> {
        let user = mock::user().save().await?;
        let updated = User::update(
            doc! { "_id": &user.id },
            doc! {
                "$inc": { "age": -1 }
            },
        )
        .await?;
        assert!(&updated.unwrap().age < &user.age);
        Ok(())
    }

    #[tokio::test]
    async fn bulk_delete() -> Result<()> {
        let users = (0..100)
            .into_iter()
            .map(|_| mock::user())
            .collect::<Vec<_>>();
        User::bulk_insert(&users).await?;
        // delete any null address
        User::bulk_delete(doc! {
            "address.apt_number": None::<String>
        })
        .await;
        let null_addresses = User::list(
            Some(doc! {
                "address.apt_number": None::<String>
            }),
            None,
        )
        .await;
        assert!(null_addresses.len() == 0);
        Ok(())
    }

    #[tokio::test]
    async fn pagination() -> Result<()> {
        let users = (0..100)
            .into_iter()
            .map(|_| mock::user())
            .collect::<Vec<_>>();
        User::bulk_insert(&users).await?;

        let users = User::list(
            None,
            Some(ListQueryOptions {
                limit: Some(10),
                skip: Some(0),
                sort: Some(doc! { "age": 1 }),
                ..Default::default()
            }),
        )
        .await;
        assert!(users.len() == 10);
        for slice in users.windows(2) {
            assert!(slice[0].age <= slice[1].age);
        }
        Ok(())
    }

    #[tokio::test]
    async fn push() -> Result<()> {
        let user = mock::user().save().await?;
        assert!(user.example_array.len() == 3);
        let updated = User::update(
            doc! { "_id": user.id },
            doc! {
                "$push": { "example_array": 1234 }
            },
        )
        .await?
        .unwrap();
        assert!(updated.example_array.len() == 4);
        Ok(())
    }

    #[tokio::test]
    async fn pull() -> Result<()> {
        let user = mock::user().save().await?;
        assert!(user.example_array.len() == 3);
        let updated = User::update(
            doc! { "_id": user.id },
            doc! {
                "$pull": { "example_array": user.example_array[0] }
            },
        )
        .await?
        .unwrap();
        assert!(updated.example_array.len() == 2);
        Ok(())
    }

    #[tokio::test]
    async fn in_operator() -> Result<()> {
        let users = (0..5).into_iter().map(|_| mock::user()).collect::<Vec<_>>();
        User::bulk_insert(&users).await?;

        let users = User::list(
            None,
            Some(ListQueryOptions {
                limit: Some(2),
                sort: Some(doc! { "created_at": -1 }),
                ..Default::default()
            }),
        )
        .await;
        let ids = users.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
        let matches = User::list(
            Some(doc! {
                "_id": { "$in": &ids }
            }),
            None,
        )
        .await;
        assert!(matches.len() == 2);
        let match_ids = matches.iter().map(|a| a.id.clone()).collect::<Vec<_>>();
        assert!(match_ids.contains(&ids[0]));
        assert!(match_ids.contains(&ids[1]));
        Ok(())
    }
}
