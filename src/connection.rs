use std::time::Duration;

use async_once::AsyncOnce;
use lazy_static::lazy_static;
use mongodb::{options::ClientOptions, Client, Database};

pub struct Connection {
    pub database: Database,
    pub client: Client,
}

lazy_static! {
    pub static ref POOL: AsyncOnce<Connection> = AsyncOnce::new(async {
        let mongo_uri = std::env::var("MONGO_URI").map_or(
            "mongodb://localhost:27017/mongoose-rs-local".to_string(),
            |uri| uri,
        );
        let mut client_options = ClientOptions::parse(mongo_uri).await.map_or_else(
            |err| {
                tracing::error!("error parsing client options {:?}", err);
                std::process::exit(1);
            },
            |opts| opts,
        );
        client_options.max_pool_size = Some(500);
        client_options.connect_timeout = Some(Duration::from_secs(10));
        let client = Client::with_options(client_options).map_or_else(
            |err| {
                tracing::error!("error connecting client: {:?}", err);
                std::process::exit(1);
            },
            |client| client,
        );
        let database = client.default_database().map_or_else(
            || {
                tracing::error!("no default database found");
                std::process::exit(1);
            },
            |db| db,
        );
        Connection { database, client }
    });
}
