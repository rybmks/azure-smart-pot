async fn iot_hub() {
    let iothub_hostname = "smart-pot-hub.azure-devices.net";
    let device_id = "maxim-device";
    let key = "rnXVcrQRnheyMISiJJa3xodF9o7hX925Lbp5kuwdelE=";
    let token_source = DeviceKeyTokenSource::new(
        iothub_hostname,
        device_id,
        key,
    ).unwrap();

    let mut client = IoTHubClient::new(iothub_hostname, device_id.into(), token_source)
    .await
    .expect("Failed to create IoT Hub");

    let mut interval = time::interval(time::Duration::from_secs(5));
    let mut count: u32 = 0;

    loop {
        interval.tick().await;

        let temperature = r#"{"temperature": 40}"#;

        let msg = Message::builder()
            .set_content_type("application/json".to_string())
            .set_content_encoding("utf-8".to_string())
            .set_body(temperature.as_bytes().to_vec())
            .set_message_id(format!("{}", count))
            .build();

        // println!("Se")

        client.send_message(msg).await.expect("Failed to send message to Azure Portal");

        count += 1;
    }
}

async fn dps() {
    let scope_id = "0ne00EE9F99";
    let device_key = "iSQB0P57mjtJSx2x5QOpfu2aT6t+GG9cIaCwk3HfmVhFKJ/0jBZnJEAVxxWNN4RPO7WBnSXKAIIMAIoTEZOFNw==";
    let device_id = "maxim-device";

    let mut client = IoTHubClient::from_provision_service(
        scope_id,
        device_id.into(),
        device_key,
        4).await.expect("Failed to create hub with DPS");

        let mut interval = time::interval(time::Duration::from_secs(5));
        let mut count: u32 = 0;
    
    loop {
        interval.tick().await;

        let temperature = r#"{"temperature": 40}"#;

        let msg = Message::builder()
            .set_content_type("application/json".to_string())
            .set_content_encoding("utf-8".to_string())
            .set_body(temperature.as_bytes().to_vec())
            .set_message_id(format!("{}", count))
            .build();

        // println!("Se")

        client.send_message(msg).await.expect("Failed to send message to Azure Portal");

        count += 1;
    }
}

// Change 
async fn c2d() {
        // env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

        let hostname = "smart-pot-hub.azure-devices.net";
        // .expect("Set IoT Hub hostname in the IOTHUB_HOSTNAME environment variable");
    let device_id = "maxim-device".to_string();
        // .expect("Set the device id in the DEVICE_ID environment variable");
    let shared_access_key = "rnXVcrQRnheyMISiJJa3xodF9o7hX925Lbp5kuwdelE=";
        // .expect("Set the device shared access key in the SHARED_ACCESS_KEY environment variable");

    let token_source = DeviceKeyTokenSource::new(
        &hostname,
        &device_id,
        &shared_access_key,
    )
    .unwrap();

    let mut client = IoTHubClient::new(&hostname, device_id, token_source)
        .await
        .expect("failed to create hub");

    println!("Initialized client");

    let mut recv = client.get_receiver().await;
    let receive_loop = async {
        while let Some(msg) = recv.recv().await {
            match msg {
                MessageType::C2DMessage(msg) => println!("Received message {:?}", msg),
                _ => {}
            }
        }
    };

    let msg = Message::new(b"Hello, world!".to_vec());
    let sender = client.send_message(msg);

    let res = tokio::join!(receive_loop, sender);

    match res {
        (_, Ok(_)) => println!("Good"),
        (_, Err(err)) => eprintln!("{}", err.to_string())
    }
}