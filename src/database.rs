use async_once::AsyncOnce;
use lazy_static::lazy_static;
use mongodb::{options::ClientOptions, Client, Database};

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
        default_database
    });
}
