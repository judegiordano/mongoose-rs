use anyhow::Result;

pub mod database;
pub mod logger;
pub mod models;

use crate::{database::Model, models::user::User};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let result = User::read_by_id("ITMUJQPDJEIRBMXQLKBU", None).await;
    // let result = User::list(None, None).await;
    tracing::info!("{:?}", result);
    Ok(())
}
