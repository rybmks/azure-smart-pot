mod private {
    use crate::core::Result;
    use crate::core::Error;
    use azure_iot_sdk::Message;
    use azure_iot_sdk::{DeviceKeyTokenSource, IoTHubClient};
    use shared::SensorData;
    use shared::Updates;
    use reqwest;
    use log::{error, info};
    use serde_json::json;
    use tokio::sync::watch;
    use tokio::time::{self, Duration};
    use azure_iot_sdk::{DirectMethodResponse, MessageType};

    #[allow(unused)]
    async fn get_hub() -> Result<IoTHubClient> {
        let hostname = std::env::var("IOTHUB_HOSTNAME")?;
        let device_id = std::env::var("DEVICE_ID")?;
        let shared_access_key = std::env::var("SHARED_ACCESS_KEY")?;
    
        let token_source = DeviceKeyTokenSource::new(
            &hostname,
            &device_id,
            &shared_access_key,
        ).unwrap();
    
        IoTHubClient::new(&hostname, device_id, token_source)
            .await
            .map_err(|err| Error::HubError(err.to_string()))
    }
    
    async fn get_hub_with_dps() -> Result<IoTHubClient> {
        let scope_id = std::env::var("SCOPE_ID")?;
        let device_id = std::env::var("DEVICE_ID")?;
        let device_key = std::env::var("DEVICE_KEY")?;
    
        IoTHubClient::from_provision_service(&scope_id, device_id, &device_key, 4)
            .await
            .map_err(|err| Error::HubError(err.to_string()))
    }

    pub async fn run() -> Result<()> {
        let base_url = std::env::var("BASE_URL")?;
        let base_url = base_url.as_str();
        
        // Initial interval value
        let initial_interval = 60_u64;
        // Creating a watch channel for interval updates
        let (interval_tx, mut interval_rx) = watch::channel::<u64>(initial_interval);
    
        let mut client = get_hub_with_dps().await?;
        info!("Initialized client");
    
        // -------------------------------
        // receive_loop
        // -------------------------------
        let mut recv_client = client.clone();
        let mut receiver = client.get_receiver().await;
    
        // - recv_client
        // - interval_tx (interval updating)
        let receive_loop = async move {
            info!("Started reveive loop.");

            loop {
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        MessageType::C2DMessage(msg) => {
                            // info!("Received C2D message {:?}", msg);
                            
                            let updates: Updates = serde_json::from_slice(&msg.body)?;
                            info!("{:?}", updates);

                            if let Some(num) = updates.telemetry_interval {
                                // Update the interval using watch channel
                                interval_tx.send(num as u64).ok();
                            }
    
                            let endpoint = format!("{}/c2d", base_url);
                            let response = reqwest::Client::new()
                                .post(&endpoint)
                                .json(&json!(updates))
                                .send()
                                .await?;
                            
                            let status = response.status();
                            let text = response.text().await?;
                            if !status.is_success() {
                                error!("C2D error: {}", text);
                            } else {
                                info!("C2D response: {}", text);
                            }
                        },
                        MessageType::DirectMethod(msg) => {
                            info!("Received direct method {:?}", msg);

                            let endpoint = if msg.method_name.eq_ignore_ascii_case("light-on") {
                                format!("{}/direct-method/light-on", base_url)
                            } else if msg.method_name.eq_ignore_ascii_case("light-off") {
                                format!("{}/direct-method/light-off", base_url)
                            } else {
                                error!("No such direct method: {}", msg.method_name);
                                continue;
                            };
                            
                            let response = reqwest::Client::new()
                                .post(&endpoint)
                                .body(msg.method_name.clone())
                                .send()
                                .await?;

                            let status = response.status();
                            let text = response.text().await?;
                            if !status.is_success() {
                                error!("Direct method error: {}", text);
                            } else {
                                info!("Direct method response: {}", text);
                            }
    
                            if let Err(err) = recv_client
                                .respond_to_direct_method(DirectMethodResponse::new(
                                    msg.request_id,
                                    0,
                                    Some(std::str::from_utf8(&msg.message.body)
                                        .unwrap_or_default()
                                        .to_string()),
                                ))
                                .await
                            {
                                error!("Error responding to direct method: {}", err);
                            }
                        },
                        MessageType::DesiredPropertyUpdate(msg) => {
                            info!("Desired properties updated {:?}", msg);
    
                            let updates: Updates = serde_json::from_slice(&msg.body)?;
                            if let Some(num) = updates.telemetry_interval {
                                // Update interval
                                interval_tx.send(num as u64).ok();
                            }
    
                            let endpoint = format!("{}/desired-props", base_url);
                            let response = reqwest::Client::new()
                                .post(&endpoint)
                                .json(&json!(updates))
                                .send()
                                .await?;
                            
                            let status = response.status();
                            let text = response.text().await?;
                            if !status.is_success() {
                                error!("Desired props error: {}", text);
                            } else {
                                info!("Desired props response: {}", text);
                            }
                        },
                        MessageType::ErrorReceive(err) => {
                            error!("Error during receive {:?}", err);
                        },
                    }
                }
            }
    
            #[allow(unreachable_code)]
            Ok::<(), Error>(())
        };
    
        // -------------------------------
        // telemetry_sender_loop
        // -------------------------------
        let mut temp_client = client.clone();
        let telemetry_sender = async move {
            let mut count = 0u32;
            info!("Started telemetry loop.");
    
            loop {
                // The current interval (in seconds)
                let current_interval = *interval_rx.borrow();
    
                // Trying to wait either for a timeout or for the channel to change
                // If the interval wasn't changed, we'll send telemetry in current_interval seconds
                match time::timeout(Duration::from_secs(current_interval), interval_rx.changed()).await {
                    // 1) The channel changed before the timeout
                    Ok(Ok(())) => {
                        // That means a new value appeared in interval_rx
                        // We simply move on to the next iteration of the loop
                        // and then use the new current_interval
                        // (retrieved on the next iteration)
                        continue;
                    },
                    // 2) The channel was closed
                    Ok(Err(_closed)) => {
                        error!("Interval watch channel closed unexpectedly");
                        break;
                    },
                    // 3) The timeout fired (the interval was not changed),
                    // so it's time to send telemetry
                    Err(_timeout) => {
                        // Request telemetry data
                        let endpoint = format!("{}/telemetry", base_url);
                        let response = reqwest::get(endpoint).await?;
                        
                        if !response.status().is_success() {
                            let text = response.text().await?;
                            error!("Error requesting telemetry: {}", text);
                        } else {
                            let telemetry = response.json::<SensorData>().await?;
                            info!("Send telemetry with id: {} to Azure portal", count);
    
                            let msg = Message::builder()
                                .set_body(serde_json::to_vec(&telemetry).unwrap())
                                .set_message_id(format!("{}-t", count))
                                .build();
    
                            temp_client.send_message(msg).await?;
                        }
    
                        count += 1;
                    }
                }
            }
    
            #[allow(unreachable_code)]
            Ok::<(), Box<dyn std::error::Error>>(())
        };
    
        // Start both loops concurrently
        let (receive_res, telemetry_res) 
            = tokio::join!(receive_loop, telemetry_sender);
    
        if let Err(e) = receive_res {
            error!("Receive loop failed: {:?}", e);
        }
        if let Err(e) = telemetry_res {
            error!("Telemetry loop failed: {:?}", e);
        }
    
        Ok(())
    }
}

crate::mod_interface!{
    orphan use {
        run
    };
}