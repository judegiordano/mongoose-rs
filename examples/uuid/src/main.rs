use anyhow::Result;
use mongoose::{doc, DateTime, Model, Uuid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TestModel {
    #[serde(rename = "_id")]
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl Default for TestModel {
    fn default() -> Self {
        Self {
            id: Self::generate_uuid(),
            username: Self::generate_nanoid(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
        }
    }
}

impl Model for TestModel {}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    // insert one
    {
        let start = std::time::Instant::now();
        let doc = TestModel::default().save().await;
        tracing::info!("{:#?} insert complete in {:?}", doc, start.elapsed());
    }
    // read one
    {
        let start = std::time::Instant::now();
        let doc = TestModel::read_by_uuid("9975506c-008b-4168-8cec-705184713701").await?;
        tracing::info!("{:#?} read complete in {:?}", doc, start.elapsed());
    }
    // update one
    {
        let start = std::time::Instant::now();
        let doc = TestModel::update(
            Default::default(),
            doc! {
                "username": "ive been updated"
            },
        )
        .await?;
        tracing::info!("{:#?} update complete in {:?}", doc, start.elapsed());
    }
    Ok(())
}
