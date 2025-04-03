use dotenv::dotenv;
use azure_server::core::{Result, run};


#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    run().await?;

    Ok(())
}
