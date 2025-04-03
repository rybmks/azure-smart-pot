//! azure.rs
//!
//! This module is responsible for establishing a connection to the Azure IoT Hub and handling messages.
//! It provides functions to connect to the IoT Hub directly or via the Device Provisioning Service (DPS),
//! as well as the main logic for processing incoming messages and sending telemetry data.

mod private {
    use crate::core::{Result, Error, Updates};

    use azure_iot_sdk::{DirectMethodResponse, MessageType};
    use azure_iot_sdk::{DeviceKeyTokenSource, IoTHubClient, Message};
    
    use reqwest;
    use log::{error, info};
    use serde_json::json;
    use tokio::sync::watch;
    use tokio::time;

    /// ## get_hub
    ///
    /// Connects directly to the Azure IoT Hub.
    ///
    /// This function uses the following environment variables:
    /// - `IOTHUB_HOSTNAME` – the IoT Hub hostname.
    /// - `DEVICE_ID` – the device identifier.
    /// - `SHARED_ACCESS_KEY` – the shared access key.
    ///
    /// ## Errors
    /// - Returns an error if any required environment variable is missing.
    /// - Returns an error if the IoT Hub client initialization fails.
    ///
    /// ## Returns
    /// Returns an instance of `IoTHubClient` upon a successful connection.
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
    
    /// ## get_hub_with_dps
    ///
    /// Connects to the Azure IoT Hub using the Device Provisioning Service (DPS).
    ///
    /// This function uses the following environment variables:
    /// - `SCOPE_ID` – the DPS scope identifier.
    /// - `DEVICE_ID` – the device identifier.
    /// - `DEVICE_KEY` – the device key.
    ///
    /// ## Errors
    /// - Returns an error if any required environment variable is missing.
    /// - Returns an error if the connection to the IoT Hub via DPS fails.
    ///
    /// ## Returns
    /// Returns an instance of `IoTHubClient` upon a successful connection.
    async fn get_hub_with_dps() -> Result<IoTHubClient> {
        let scope_id = std::env::var("SCOPE_ID")?;
        let device_id = std::env::var("DEVICE_ID")?;
        let device_key = std::env::var("DEVICE_KEY")?;

        IoTHubClient::from_provision_service(&scope_id, device_id, &device_key, 4)
            .await
            .map_err(|err| Error::HubError(err.to_string()))
    }

    /// ## run
    ///
    /// Main function that contains the overall logic of the module.
    ///
    /// This function performs the following tasks:
    /// - Retrieves the base URL from the `BASE_URL` environment variable.
    /// - Creates a watch channel to dynamically update the telemetry interval.
    /// - Establishes a connection to the IoT Hub using DPS via the `get_hub_with_dps` function.
    /// - Launches two asynchronous loops:
    ///   1. `receive_loop` – handles incoming messages (Cloud-to-Device, direct method, desired property updates).
    ///   2. `telemetry_sender_loop` – sends telemetry data by periodically requesting data from the device.
    ///
    /// When a message with an updated telemetry interval is received, the new interval is sent through the watch channel.
    ///
    /// ## Errors
    /// - Returns an error if the `BASE_URL` environment variable is missing.
    /// - May return errors if connecting to the IoT Hub or processing messages fails.
    ///
    /// ## Returns
    /// Returns `(Result<()>, Result<()>)` results of loops.
    pub async fn run(base_url: &str) -> (Result<()>, Result<()>) {
        // Initial interval value
        let initial_interval = 5_u64;
        // Creating a watch channel for interval updates
        let (interval_tx, mut interval_rx) = watch::channel::<u64>(initial_interval);
    
        let mut client = match get_hub_with_dps().await {
            Ok(client) => client,
            Err(err) => return (Err(err), Ok(()))
        };

        info!("Initialized client");
    
        // -------------------------------
        // receive_loop
        // -------------------------------
        let mut recv_client = client.clone();
        let mut receiver = client.get_receiver().await;
    
        // - recv_client
        // - interval_tx (interval updating)
        let receive_loop = async move {
        info!("Started receive loop.");
           
            loop {
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        MessageType::C2DMessage(msg) => {
                            let updates: Updates = match serde_json::from_slice(&msg.body) {
                                Ok(val) => val,
                                Err(err) => {
                                    error!("Failed to deserialize cloud to device message: {}", err);
                                    continue;
                                }
                            };
                            info!("Received C2D message  {:?}", updates);
    
                            if let Some(num) = updates.telemetry_interval {
                                // Update the interval using watch channel
                                info!("Updated interval to {}!", num);
                                interval_tx.send(num as u64).ok();
                            }

                            let endpoint = format!("{}/c2d", base_url);
                            let response = match reqwest::Client::new()
                                .post(&endpoint)
                                .json(&json!(updates))
                                .send()
                                .await
                            {
                                Ok(res) => res,
                                Err(err) => {
                                    error!("Failed send message to device: {}", err);
                                    continue;
                                },
                            };
                            
                            let status = response.status();
                            if !status.is_success() {
                                match response.text().await {
                                    Ok(text) => info!("Cloud to device error: {}", text),
                                    Err(err) => error!("Parse error: {}", err)
                                };
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
                            
                            let response = match reqwest::Client::new()
                                .post(&endpoint)
                                .body(msg.method_name.clone())
                                .send()
                                .await
                            {
                                Ok(res) => res,
                                Err(err) => {
                                    error!("Failed send message to device: {}", err);
                                    continue;
                                },
                            };

                            let status = response.status();
                            if !status.is_success() {
                                match response.text().await {
                                    Ok(text) => info!("Direct message error: {}", text),
                                    Err(err) => error!("Parse error: {}", err)
                                };
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
    
                            let updates: Updates = match serde_json::from_slice(&msg.body) {
                                Ok(val) => val,
                                Err(err) => {
                                    error!("Parse error: {}", err);
                                    continue;
                                }
                            };
                            if let Some(num) = updates.telemetry_interval {
                                // Update interval
                                info!("Updated interval to {}!", num);
                                interval_tx.send(num as u64).ok();
                            }
    
                            let endpoint = format!("{}/desired-props", base_url);
                            let response = match reqwest::Client::new()
                                .post(&endpoint)
                                .json(&json!(updates))
                                .send()
                                .await
                            {
                                Ok(res) => res,
                                Err(err) => {
                                    error!("Failed to send desired properties: {}", err);
                                    continue;
                                }
                            };
                            
                            let status = response.status();
                            if !status.is_success() {
                                match response.text().await {
                                    Ok(text) => info!("Desired properties error: {}", text),
                                    Err(err) => error!("Parse error: {}", err)
                                };
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
                info!("Waiting for timeout");
                match time::timeout(time::Duration::from_secs(current_interval), interval_rx.changed()).await {
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
                        let response = match reqwest::get(endpoint).await {
                            Ok(res) => res,
                            Err(err) => {
                                error!("Failed to get telemetry from device: {}", err);
                                continue;
                            }
                        };
                        
                        if !response.status().is_success() {
                            error!("Error requesting telemetry");
                        } else {
                            let sensor_data = match response.bytes().await {
                                Ok(data) => data,
                                Err(err) => {
                                    error!("Parse error: {}", err);
                                    continue;
                                }
                            }; 
    
                            let msg = Message::builder()
                                .set_body(sensor_data.to_vec())
                                .set_content_type("application/json".to_string())
                                .set_content_encoding("UTF-8".to_string())
                                .set_message_id(format!("{}-t", count)) 
                                .build();
    
                            match temp_client.send_message(msg).await {
                                Ok(_) => info!("Sent telemetry with id: {} to Azure portal", count),
                                Err(err) => error!("Failed to send message to Azure Portal. {}", err)
                            };
                        }
    
                        count += 1;
                    }
                }
            }
    
            #[allow(unreachable_code)]
            Ok::<(), Error>(())
        };
    
        // Start both loops concurrently
        tokio::join!(receive_loop, telemetry_sender)
    }
}

crate::mod_interface!{
    orphan use {
        run
    };
}