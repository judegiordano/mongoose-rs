use anyhow::Result;

pub mod database;
pub mod logger;
pub mod models;

use crate::{database::Model, models::user::User};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let now = std::time::Instant::now();
    let users = User::list(None, None).await;
    tracing::info!("{:#?}", users);
    tracing::info!("complete in {:#?}", now.elapsed());
    Ok(())
}
