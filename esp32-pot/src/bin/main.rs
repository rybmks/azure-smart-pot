use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::i2c::config::Config;
use log::*;
use std::sync::Arc;
use std::sync::Mutex;

use esp32_pot::core::Result;
use esp32_pot::core::esp::board::BoardBuilder;
use esp32_pot::core::esp::{DhtConfig, DhtType, server::server_init};

use esp_idf_svc::hal::task;
use esp_idf_svc::log::EspLogger;

use esp_idf_hal::gpio::IOPin;
use esp_idf_hal::prelude::Peripherals;

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

    let server = server_init(board);
    // Prevent server from dropping
    core::mem::forget(server);

    Ok(())
}
