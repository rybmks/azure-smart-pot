# Smart Pot

## Overview

Smart Pot is an IoT-based system built on the ESP32 platform and the Rust programming language. The system collects data from various sensors (DS18B20, BH1750, DHT11, and DHT22) and sends it to Azure via `azure-server`. It also allows you to control a connected light bulb and switch temperature units between Celsius and Fahrenheit through HTTP endpoints.

### Supported Sensors

- **DS18B20**: Temperature sensor  
- **DHT11 & DHT22**: Humidity and temperature sensors  
- **BH1750**: Light intensity sensor  

### Key Features

- Cloud-to-device messages  
- Direct methods  
- Sending telemetry from the device to the Azure Portal  
- Azure IoT Hub over MQTT protocol  
- Azure DPS  
- Azure Storage  

## Getting Started

1. **Configure Wi-Fi:**  
   In `esp32-pot/.cargo/config.toml`, set your Wi-Fi credentials (SSID, password, etc.).

2. **Set Azure Environment Variables:**  
   In the `.env` file (at the azure-server project root), configure your Azure environment variables (e.g., `SCOPE_ID`, `DEVICE_ID`, `DEVICE_KEY`, `BASE_URL`, etc.).

3. **Build and Run the ESP32 Container:**
   ```bash
   cd esp32-pot

   # Build the Docker image
   docker build -t esp32-pot .

   # Run the image
   ./run.sh
   ```

4. **After the ESP32 container finishes building and running**, proceed with the Azure server:
    ```bash
    cd ../azure-server

    # Build the Docker image
    docker build -t azure-server .

    # Run the image
    ./run.sh    
    ```