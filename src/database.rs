use anyhow::Result;
use async_once::AsyncOnce;
use async_trait::async_trait;
use bson::doc;
use lazy_static::lazy_static;
use mongodb::{options::ClientOptions, Client, Collection, Database};
use serde::Serialize;

use crate::models::user::User;

#[async_trait]
pub trait Model: Serialize + Sized + Send + Default {
    fn collection_name<'a>() -> &'a str;
    async fn create_indexes(db: &Database);
    async fn collection() -> Collection<Self> {
        POOL.get().await.collection::<Self>(Self::collection_name())
    }
    fn generate_id() -> String {
        use nanoid::nanoid;
        // ~2 million years needed, in order to have a 1% probability of at least one collision.
        // https://zelark.github.io/nano-id-cc/
        let alphabet = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        ];
        nanoid!(20, &alphabet)
    }
    async fn save(&self) -> Result<&Self> {
        Self::collection().await.insert_one(self, None).await?;
        Ok(self)
    }
}

lazy_static! {
    pub static ref POOL: AsyncOnce<Database> = AsyncOnce::new(async {
        let mongo_uri = std::env::var("MONGO_URI").map_or(
            "mongodb://localhost:27017/local-database".to_string(),
            |uri| uri,
        );
        let client_options = ClientOptions::parse(mongo_uri).await.map_or_else(
            |err| {
                tracing::error!("error parsing client options {:?}", err);
                std::process::exit(1);
            },
            |opts| opts,
        );
        let client = Client::with_options(client_options).map_or_else(
            |err| {
                tracing::error!("error connecting client: {:?}", err);
                std::process::exit(1);
            },
            |client| client,
        );
        let default_database = client.default_database().map_or_else(
            || {
                tracing::error!("no default database found");
                std::process::exit(1);
            },
            |db| db,
        );
        tracing::info!("connected to {:?}", default_database.name());
        {
            // migrate indexes on connection
            User::create_indexes(&default_database).await;
        }
        default_database
    });
}
