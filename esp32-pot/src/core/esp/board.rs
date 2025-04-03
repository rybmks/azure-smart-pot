//!
//!  Board module
//!

mod private {
    use crate::core::esp::{Bh1750, DhtConfig, DhtSensor, Ds18B20Sensor, Sensor, wifi};
    use crate::core::{Result, SmartPotError, Telemetry};
    use esp_idf_hal::delay::FreeRtos;
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
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;

    /// # Board
    ///
    /// Represents a smart board with supported sensors and Wi-Fi module integration.
    /// The board is capable of managing environmental sensors (e.g., DHT, BH1750, DS18B20)
    /// and controlling output pins such as light.
    ///
    /// ## Fields:
    /// - `wifi`:  
    ///   An instance of `AsyncWifi` that manages Wi-Fi connectivity.
    /// - `sensors`:  
    ///   A vector of boxed sensor objects implementing the `Sensor` trait.
    /// - `light_pin`:  
    ///   An output pin used to enable or disable the pot's light.
    ///
    /// ## Usage
    /// A `Board` instance is created using the [`BoardBuilder`] pattern.
    pub struct Board<T>
    where
        T: OutputPin,
    {
        pub wifi: AsyncWifi<EspWifi<'static>>,
        pub sensors: Vec<Box<dyn Sensor<'static> + Send>>,
        pub light_pin: PinDriver<'static, T, Output>,
        pub is_fahrenheit: bool,
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
        ///
        /// # Returns
        /// A new `BoardBuilder` with no sensors registered yet.
        pub fn new() -> Self {
            BoardBuilder { sensors: vec![] }
        }

        /// Adds a BH1750 light sensor to the board.
        ///
        /// # Parameters
        /// - `bh1750_i2c`: The I2C driver used for communication.
        /// - `resolutin`: The measurement resolution of the BH1750 sensor.
        ///
        /// # Returns
        /// The updated builder instance.
        pub fn add_bh1750_sensor(
            mut self,
            bh1750_i2c: I2cDriver<'static>,
            resolutin: bh1750::Resolution,
        ) -> Self {
            let bh = Box::new(Bh1750::new(bh1750_i2c, resolutin));
            self.sensors.push(bh);
            self
        }

        /// Adds DHT11/DHT22 sensors using the provided configurations.
        ///
        /// # Parameters
        /// - `dht_configs`: A vector of `DhtConfig` instances specifying pins and sensor types.
        ///
        /// # Returns
        /// The updated builder instance or error.
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
        ///
        /// # Returns
        /// The updated builder instance or error.
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
        ///
        /// # Returns
        /// A fully configured and connected `Board`.
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
                is_fahrenheit: false,
            })
        }
    }
    impl<T: OutputPin> Board<T> {
        /// Turns on the light (sets the light control pin high).
        ///
        /// # Returns
        /// `Ok(())` if successful, or an error if pin control fails.
        pub fn light_on(&mut self) -> Result<()> {
            self.light_pin.set_high().map_err(SmartPotError::EspError)
        }

        /// Turns off the light (sets the light control pin low).
        ///
        /// # Returns
        /// `Ok(())` if successful, or an error if pin control fails.
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
        pub fn set_is_fahrenheit(&mut self, is_fahrenheit: bool) {
            self.is_fahrenheit = is_fahrenheit;
        }

        /// Retrieves telemetry data from all connected sensors with 3 retry attempts.
        ///
        /// # Returns
        /// A vector of successfully collected `SensorData`.
        ///
        /// Any sensor that fails after retries will be logged and skipped.
        pub fn get_telemetry(&mut self) -> Vec<crate::core::SensorData> {
            self.sensors
                .iter_mut()
                .filter_map(|sensor| {
                    for i in 1..=3 {
                        match sensor.read_data() {
                            Ok(mut data) => {
                                if self.is_fahrenheit {
                                    match data.telemetry {
                                        Telemetry::LightValue(_) => {}
                                        Telemetry::Temperature(ref mut temperature_data) => {
                                            *temperature_data =
                                                (*temperature_data * 9.0 / 5.0) + 32.0;
                                        }
                                        Telemetry::TemperatureWithHumidity(
                                            ref mut temperature_with_humidity_data,
                                        ) => {
                                            temperature_with_humidity_data.temperature =
                                                (temperature_with_humidity_data.temperature * 9.0
                                                    / 5.0)
                                                    + 32.0;
                                        }
                                    }
                                }

                                log::info!("Sensor #{} => {:?}", sensor.get_name(), data);
                                return Some(data);
                            }
                            Err(e) => log::warn!(
                                "Retry #{i}: Sensor #{} read error: {:?}",
                                sensor.get_name(),
                                e
                            ),
                        }
                        FreeRtos::delay_ms(50);
                    }
                    log::error!("Sensor #{} failed to read after retries", sensor.get_name());
                    None
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
