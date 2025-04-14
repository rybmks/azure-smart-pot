//!
//!  Board module
//!

mod private {
    use crate::core::esp::{Bh1750, DhtConfig, DhtSensor, Ds18B20Sensor, Sensor, wifi};
    use crate::core::{Result, SmartPotError};
    use esp_idf_hal::gpio::Output;
    use esp_idf_hal::gpio::{AnyIOPin, OutputPin, PinDriver};
    use esp_idf_hal::i2c::I2cDriver;
    use esp_idf_hal::modem::Modem;
    use esp_idf_svc::sntp::{EspSntp, SyncStatus};
    use esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        nvs::EspDefaultNvsPartition,
        timer::EspTaskTimerService,
        wifi::{AsyncWifi, EspWifi},
    };
    use smart_pot_core::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    const MAX_SENSOR_MEASUREMENT_RETRIES: u8 = 3;

    /// # Board
    ///
    /// Represents a smart board with supported sensors and Wi-Fi module integration.
    /// The board is capable of managing environmental sensors (e.g., DHT, BH1750, DS18B20)
    /// and controlling output pins such as light.
    ///
    /// ## Usage
    /// A `Board` instance is created using the [`BoardBuilder`] pattern.
    pub struct Board<T>
    where
        T: OutputPin,
    {
        ///   An instance of `AsyncWifi` that manages Wi-Fi connectivity.
        pub wifi: AsyncWifi<EspWifi<'static>>,
        ///   A vector of boxed sensor objects implementing the `Sensor` trait.
        pub sensors: Vec<Box<dyn Sensor<'static> + Send>>,
        ///   An output pin used to enable or disable the pot's light.
        pub light_pin: PinDriver<'static, T, Output>,
        pub temperature_units: TemperatureUnits,
    }

    /// # BoardBuilder
    ///
    /// A builder for constructing a `Board` with custom sensor configurations.
    /// Allows step-by-step registration of various sensor types before building the full board.
    #[derive(Default)]
    pub struct BoardBuilder {
        sensors: Vec<Box<dyn Sensor<'static> + Send>>,
    }
    impl BoardBuilder {
        /// Creates a new empty builder instance.
        pub fn new() -> Self {
            BoardBuilder { sensors: vec![] }
        }

        /// Adds a BH1750 light sensor to the board.
        ///
        /// # Parameters
        /// - `bh1750_i2c`: The I2C driver used for communication.
        /// - `resolutin`: The measurement resolution of the BH1750 sensor.
        pub fn add_bh1750_sensor(
            mut self,
            bh1750_i2c: I2cDriver<'static>,
            resolution: bh1750::Resolution,
        ) -> Self {
            let bh = Box::new(Bh1750::new(bh1750_i2c, resolution));
            self.sensors.push(bh);
            self
        }

        /// Adds DHT11/DHT22 sensors using the provided configurations.
        ///
        /// # Parameters
        /// - `dht_configs`: A vector of `DhtConfig` instances specifying pins and sensor types.
        pub fn add_dht_sensors(mut self, dht_configs: Vec<DhtConfig>) -> Result<Self> {
            for dht in dht_configs {
                let dht_driver = PinDriver::input_output_od(dht.pin)?;
                let dht_sensor = Box::new(DhtSensor::new(dht_driver, dht.dht_type));

                self.sensors.push(dht_sensor);
            }
            Ok(self)
        }

        /// Adds DS18B20 temperature sensors on the provided OneWire-capable pins.
        ///
        /// # Parameters
        /// - `ds18b20_pins`: A list of GPIO pins to which DS18B20 sensors are connected.
        pub fn add_ds18b20_sensors(mut self, ds18b20_pins: Vec<AnyIOPin>) -> Result<Self> {
            for ds in ds18b20_pins {
                let ds_driver = PinDriver::input_output_od(ds)?;
                let one_wire_bus = one_wire_bus::OneWire::new(ds_driver)
                    .map_err(|e| SmartPotError::OneWireError(e.into()))?;

                let onewire_ref = Arc::new(Mutex::new(one_wire_bus));
                let ds18b20_sensors = Ds18B20Sensor::find_all(onewire_ref)?;
                let ds18b20_sensors = ds18b20_sensors
                    .into_iter()
                    .map(|s| s as Box<dyn Sensor + Send>)
                    .collect::<Vec<Box<dyn Sensor + Send>>>();
                self.sensors.extend(ds18b20_sensors);
            }
            Ok(self)
        }

        /// Finalizes the board construction with the given light pin and Wi-Fi credentials.
        ///
        /// # Parameters
        /// - `light_pin`: Output pin to control a pot's light.
        /// - `wifi_modem`: Wi-Fi modem instance.
        /// - `wifi_ssid`: Wi-Fi network SSID.
        /// - `wifi_password`: Wi-Fi password.
        pub async fn build<T: OutputPin>(
            self,
            light_pin: T,
            wifi_modem: Modem,
            wifi_ssid: &str,
            wifi_password: &str,
        ) -> Result<Board<T>> {
            let sysloop = EspSystemEventLoop::take()?;
            let timer_service = EspTaskTimerService::new()?;
            let nvs = Some(EspDefaultNvsPartition::take()?);

            let wifi: AsyncWifi<EspWifi<'_>> = wifi(
                wifi_ssid,
                wifi_password,
                wifi_modem,
                &sysloop,
                nvs,
                &timer_service,
            )
            .await?;
            let ntp = EspSntp::new_default()?;
            let light_driver = PinDriver::output(light_pin)?;

            while ntp.get_sync_status() != SyncStatus::Completed {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(Board {
                wifi,
                sensors: self.sensors,
                light_pin: light_driver,
                temperature_units: TemperatureUnits::Celsius,
            })
        }
    }
    impl<T: OutputPin> Board<T> {
        /// Turns on the light (sets the light control pin high).
        pub fn light_on(&mut self) -> Result<()> {
            self.light_pin.set_high().map_err(SmartPotError::EspError)
        }

        /// Turns off the light (sets the light control pin low).
        pub fn light_off(&mut self) -> Result<()> {
            self.light_pin.set_low().map_err(SmartPotError::EspError)
        }

        /// Sets whether the temperature readings should be displayed in Fahrenheit.
        ///
        /// This method allows toggling between displaying temperatures in Celsius or Fahrenheit.
        /// When `is_fahrenheit` is set to `true`, temperatures will be shown in Fahrenheit. Otherwise,
        /// the temperatures are displayed in Celsius.
        ///
        /// # Parameters
        /// - `is_fahrenheit`: A boolean value. If `true`, the temperature will be displayed in Fahrenheit;
        ///   if `false`, it will be in Celsius.
        ///
        pub fn set_temperature_units(&mut self, units: TemperatureUnits) {
            self.temperature_units = units;
        }

        /// Retrieves telemetry data from all connected sensors with 3 retry attempts.
        ///
        /// # Returns
        /// A vector of successfully collected `SensorData`.
        ///
        /// Any sensor that fails after retries will be logged and skipped.
        pub fn get_telemetry(&mut self) -> Vec<SensorData> {
            self.sensors
                .iter_mut()
                .filter_map(|sensor| {
                    sensor.read_sensor_with_retries(
                        MAX_SENSOR_MEASUREMENT_RETRIES,
                        &self.temperature_units,
                    )
                })
                .collect()
        }
    }
}

crate::mod_interface! {
    own use {
        Board,
        BoardBuilder
    };
}
