use anyhow::Result;
use tracing_subscriber::FmtSubscriber;

pub fn init() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
