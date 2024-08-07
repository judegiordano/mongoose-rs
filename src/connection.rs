use async_once::AsyncOnce;
use lazy_static::lazy_static;
use mongodb::{options::ClientOptions, Client, Database};

pub struct Connection {
    pub database: Database,
    pub client: Client,
}

const LOCAL_URI: &str =
    "mongodb://localhost:27017/mongoose-rs-local?connectTimeoutMS=10000&maxPoolSize=500";

lazy_static! {
    pub static ref POOL: AsyncOnce<Connection> = AsyncOnce::new(async {
        let mongo_uri = std::env::var("MONGO_URI").map_or(LOCAL_URI.to_string(), |uri| uri);
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
