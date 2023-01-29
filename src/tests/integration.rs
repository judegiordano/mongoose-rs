#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing_subscriber::FmtSubscriber;

    use crate::tests::mock::{
        self,
        user_model::{Address, User},
    };
    use mongoose::{doc, ListQueryOptions, Model, POOL};

    #[tokio::test]
    async fn database_methods() -> Result<()> {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        // create indices
        {
            let db = &POOL.get().await.0;
            User::create_indexes(db).await;
        }
        // create
        {
            let username = format!("username_{}", mock::nanoid());
            let address = Address {
                address: 123,
                street: "Fake Street Name".to_string(),
                city: "Fake City".to_string(),
                state: "CA".to_string(),
                zip: "F1256".to_string(),
                country: "US".to_string(),
                apt_number: None,
            };
            let new_user = User {
                username: username.clone(),
                age: mock::number(),
                address: address.clone(),
                ..Default::default()
            };
            let inserted = new_user.save().await?;
            assert_eq!(inserted.username, username);
            assert_eq!(inserted.age, new_user.age);
        }
        // bulk create
        {
            let username_one = format!("username_{}", mock::nanoid());
            let username_two = format!("username_{}", mock::nanoid());
            let address = Address {
                address: 123,
                street: "Fake Street Name".to_string(),
                city: "Fake City".to_string(),
                state: "CA".to_string(),
                zip: "F1256".to_string(),
                country: "US".to_string(),
                apt_number: Some("A3".to_string()),
            };
            let inserted = User::bulk_insert(vec![
                User {
                    username: username_one.clone(),
                    age: mock::number(),
                    address: address.clone(),
                    ..Default::default()
                },
                User {
                    username: username_two.clone(),
                    age: mock::number(),
                    address: address.clone(),
                    ..Default::default()
                },
            ])
            .await?;
            assert_eq!(inserted.inserted_ids.len(), 2);
        }
        // list
        {
            let users = User::list(None, None).await;
            assert_eq!(users.len() > 0, true);
        }
        // $inc
        {
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
            //
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
        }
        Ok(())
    }
}
