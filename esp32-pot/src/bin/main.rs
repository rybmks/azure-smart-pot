// main.rs

use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::i2c::config::Config;
use log::*;
use std::sync::Arc;
use std::sync::Mutex;

use esp32_pot::core::esp::board::BoardBuilder;
use esp32_pot::core::esp::{DhtConfig, DhtType};
use esp32_pot::core::{Result, SmartPotError};

use esp_idf_svc::hal::task;
use esp_idf_svc::log::EspLogger;

use esp_idf_hal::gpio::IOPin;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::http::server::EspHttpServer;

// Environment vars or constants
const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let result = task::block_on(async_main());
    if let Err(e) = result {
        error!("Error: {:?}", e);
    } else {
        info!("Done!");
    }
}

/// Main async logic
async fn async_main() -> Result<()> {
    let peripherals = Peripherals::take()?;
    let wifi_modem = peripherals.modem;

    let ds_pins = vec![peripherals.pins.gpio16.downgrade()];
    let dht_configs = vec![
        DhtConfig::new(peripherals.pins.gpio17.downgrade(), DhtType::Dht11),
        DhtConfig::new(peripherals.pins.gpio5.downgrade(), DhtType::Dht22),
    ];

    let bh1750_i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &Config::default(),
    )?;

    let board = BoardBuilder::new()
        .add_bh1750_sensor(bh1750_i2c, bh1750::Resolution::High)
        .add_dht_sensors(dht_configs)?
        .add_ds18b20_sensors(ds_pins)?
        .build(peripherals.pins.gpio2, wifi_modem, SSID, PASS)
        .await?;

    let board = Arc::new(Mutex::new(board));
    let mut server = EspHttpServer::new(&Default::default())?;

    // GET / — telemetry JSON
    {
        let board = Arc::clone(&board);
        server.fn_handler("/", embedded_svc::http::Method::Get, move |req| {
            let mut res = match req.into_ok_response() {
                Ok(r) => r,
                Err(e) => {
                    error!("Response error: {:?}", e);
                    return Err(SmartPotError::EspIoError(e));
                }
            };

            let mut board = match board.lock() {
                Ok(b) => b,
                Err(_) => return Err(SmartPotError::MutexError()),
            };
            let telemetry = board.get_telemetry();

            match serde_json::to_string(&telemetry) {
                Ok(telemetry) => {
                    res.write(telemetry.as_bytes())?;
                    Ok(())
                }
                Err(err) => Err(SmartPotError::SerializationError(err)),
            }
        })?;
    }

    // POST /light/on
    {
        let board = Arc::clone(&board);
        server.fn_handler("/light/on", embedded_svc::http::Method::Post, move |_| {
            if let Ok(mut board) = board.lock() {
                board.light_on()?;
            } else {
                error!("Board lock failed on /light/on");
                return Err(SmartPotError::MutexError());
            }

            Ok::<(), SmartPotError>(())
        })?;
    }

    // POST /light/off
    {
        let board = Arc::clone(&board);
        server.fn_handler("/light/off", embedded_svc::http::Method::Post, move |_| {
            if let Ok(mut board) = board.lock() {
                board.light_off()?;
            } else {
                return Err(SmartPotError::MutexError());
            }

            Ok(())
        })?;
    }

    // Prevent server from dropping
    core::mem::forget(server);

    Ok(())
}
