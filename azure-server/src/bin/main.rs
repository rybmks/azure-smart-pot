use dotenv::dotenv;
use log::{error, info};
use azure_server::core::{Result, run};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let reconections = std::env::var("RECONNECTIONS_ATTEMPTS")?.parse()?;
    let base_url = std::env::var("BASE_URL")?;

    for i in 0..reconections {
        info!("Start main loops. Attempt {}", i);
        let (receive_res, send_res) = run(&base_url).await;
    
        if let Err(error) = receive_res {
            error!("{}. Reconnecting...", error);
            continue;
        }

        if let Err(error) = send_res {
            error!("{}. Reconnecting...", error);
            continue;
        }
    }
    

    Ok(())
}
