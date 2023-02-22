#[cfg(test)]
mod create {
    use crate::tests::mock::{self, log, Log, Post, User};
    use crate::types::{Index, IndexDirection, IndexField, MongooseError};
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

    #[tokio::test]
    async fn bulk_create_with_relation() -> Result<(), MongooseError> {
        let new_user = mock::user();
        let inserted = new_user.save().await?;
        assert_eq!(inserted.username, new_user.username);
        assert_eq!(inserted.age, new_user.age);
        let posts = (0..5)
            .into_iter()
            .map(|_| mock::post(inserted.id.to_string()))
            .collect::<Vec<_>>();
        let inserted = Post::bulk_insert(&posts).await?;
        assert!(inserted.inserted_ids.len() == 5);
        Ok(())
    }

    #[tokio::test]
    async fn create_indexes() -> Result<(), MongooseError> {
        let indexes = [
            Index {
                keys: &[IndexField {
                    field: "username",
                    direction: IndexDirection::ASC,
                }],
                unique: true,
                ..Default::default()
            },
            Index {
                keys: &[IndexField {
                    field: "slug",
                    direction: IndexDirection::TEXT,
                }],
                sparse: true,
                ..Default::default()
            },
            Index {
                keys: &[
                    IndexField {
                        field: "email",
                        direction: IndexDirection::ASC,
                    },
                    IndexField {
                        field: "created_at",
                        direction: IndexDirection::DESC,
                    },
                ],
                unique: true,
                ..Default::default()
            },
        ];
        let created_names = User::create_indexes(&indexes).await?.index_names;
        let names = User::collection().list_index_names().await.unwrap();
        created_names
            .iter()
            .for_each(|name| assert!(names.contains(name)));
        Ok(())
    }

    #[tokio::test]
    async fn create_ttl_indexes() -> Result<(), MongooseError> {
        Log::create_indexes(&[Index {
            keys: &[IndexField {
                field: "created_at",
                direction: IndexDirection::ASC,
            }],
            expire_after: Some(std::time::Duration::from_millis(1_000)),
            ..Default::default()
        }])
        .await?;
        let new_log = log().save().await?;
        let log = Log::read_by_id(&new_log.id).await?;
        assert!(log.id == new_log.id);
        // must sleep to allow the mongo engine to drop the TTL document
        std::thread::sleep(std::time::Duration::from_secs(60));
        let log = Log::read_by_id(&new_log.id).await;
        // shuold not be found after TTL expires
        assert!(log.is_err());
        Ok(())
    }
}
