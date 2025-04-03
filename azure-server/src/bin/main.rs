use dotenv::dotenv;
use azure_server::core::{Result, run};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let reconections = std::env::var("RECONNECTIONS_ATTEMPTS")?;

    for i in 0..reconections {
        let (receive_res, send_res) = run()
            .await
            .excpect("Failed to start main loops");
    
        if let Err(error) = receive_res {
            error!("{}. Reconnecting...");
            continue;
        }

        if let Err(error) = send_res {
            error!("{}. Reconnecting...");
            continue;
        }
    }
    

    Ok(())
}
