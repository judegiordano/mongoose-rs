use anyhow::Result;

pub mod database;
pub mod logger;
pub mod models;

use crate::{database::POOL, models::user::User};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let _ = POOL.get().await;
    let new_user = User {
        username: "hmmm".to_string(),
        ..Default::default()
    };
    tracing::info!("{:?}", new_user);
    println!("Hello, world!");
    Ok(())
}
