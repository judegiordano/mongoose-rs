#[cfg(test)]
mod update_tests {
    use anyhow::Result;

    use crate::tests::mock::{self, user_model::User};
    use mongoose::{doc, Model};

    #[tokio::test]
    async fn increment() -> Result<()> {
        let user = mock::user().save().await?.to_owned();
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
    async fn decrement() -> Result<()> {
        let user = mock::user().save().await?.to_owned();
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
    async fn push() -> Result<()> {
        let user = mock::user().save().await?.to_owned();
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
        let user = mock::user().save().await?.to_owned();
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
    async fn update_sub_document() -> Result<()> {
        let user = mock::user().save().await?.to_owned();
        let new_city = mock::nanoid();
        let updated = User::update(
            doc! { "_id": user.id },
            doc! {
                "address.city": &new_city
            },
        )
        .await?
        .unwrap();
        assert!(updated.address.city == new_city);
        Ok(())
    }
}
