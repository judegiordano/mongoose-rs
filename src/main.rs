use anyhow::Result;

pub mod database;
pub mod logger;
pub mod models;

use crate::{database::Model, models::user::User};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let user = User {
        username: "new_user".to_string(),
        age: 26,
        ..Default::default()
    }
    .save()
    .await?;
    tracing::info!("{:#?}", user);
    Ok(())
}
