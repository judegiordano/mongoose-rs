use anyhow::Result;
use bson::doc;

pub mod database;
pub mod logger;
pub mod models;

use crate::{database::Model, models::user::User};

#[tokio::main]
async fn main() -> Result<()> {
    logger::init()?;
    let increase_user_age = User::update_many(
        doc! { "username": { "$exists": true } },
        doc! { "$inc": { "age": 1 } },
    )
    .await?;
    tracing::info!("{:?}", increase_user_age);
    Ok(())
}
