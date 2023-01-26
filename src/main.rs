use anyhow::Result;

pub mod database;
pub mod logger;

use crate::database::POOL;

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let _ = POOL.get().await;
    println!("Hello, world!");
    Ok(())
}
