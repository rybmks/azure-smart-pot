//!
//! DHT Sensor Module
//!

mod private {
    use crate::core::esp::private::TemperatureWithHumidity;
    use crate::core::esp::{DhtType, Sensor};
    use crate::core::{Result, SensorData, SmartPotError, Telemetry};
    use dht_sensor::*;
    use esp_idf_hal::gpio::{InputOutput, InputPin, OutputPin, PinDriver};

    type Driver<T> = PinDriver<'static, T, InputOutput>;

    /// # DhtSensor
    ///
    /// This struct represents a DHT sensor (either DHT11 or DHT22). It allows you to read temperature and humidity data
    /// from the sensor using the GPIO pins.
    ///
    /// ## Type Parameters:
    /// - `T`: The type of GPIO pin used for communication with the DHT sensor, which must implement both `InputPin` and `OutputPin`.
    pub struct DhtSensor<T>
    where
        T: InputPin + OutputPin,
    {
        pin_driver: Driver<T>,
        dht_type: DhtType,
    }

    impl<T> DhtSensor<T>
    where
        T: InputPin + OutputPin,
    {
        /// Constructs a new `DhtSensor` instance.
        ///
        /// # Parameters:
        /// - `pin`: The pin driver `PinDriver<'static, T, InputOutput>` used to communicate with the DHT sensor.
        /// - `dht_type`: The type of DHT sensor being used (`Dht11` or `Dht22`).
        ///
        /// # Returns:
        /// A new instance of the `DhtSensor` struct.
        pub fn new(pin: Driver<T>, dht_type: DhtType) -> Self {
            DhtSensor {
                pin_driver: pin,
                dht_type,
            }
        }
    }

    impl<T> Sensor<'_> for DhtSensor<T>
    where
        T: InputPin + OutputPin,
    {
        /// Returns the name of the sensor.
        fn get_name(&self) -> String {
            "DHT".to_string()
        }

        /// Reads temperature and humidity data from the DHT sensor.
        ///
        /// Depending on the `dht_type` (either DHT11 or DHT22), this function reads the temperature and humidity
        /// values from the sensor and returns them in a `SensorData` struct.
        ///
        /// # Returns:
        /// - `Result<SensorData>`: A `SensorData` instance containing the timestamp and telemetry data (temperature and humidity).
        ///
        /// # Errors:
        /// This function returns a `SmartPotError::DhtError` if an error occurs during the sensor's reading process.
        fn read_data(&mut self) -> Result<SensorData> {
            let mut delay = esp_idf_hal::delay::Delay::new_default();

            match self.dht_type {
                DhtType::Dht11 => match dht11::Reading::read(&mut delay, &mut self.pin_driver) {
                    Ok(data) => Ok(SensorData {
                        timestamp: chrono::Utc::now(),
                        telemetry: Telemetry::TemperatureWithHumidity(TemperatureWithHumidity {
                            temperature: data.temperature as f32,
                            humidity: data.relative_humidity as f32,
                        }),
                    }),
                    Err(err) => Err(SmartPotError::DhtError(err.into())),
                },

                DhtType::Dht22 => match dht22::Reading::read(&mut delay, &mut self.pin_driver) {
                    Ok(data) => Ok(SensorData {
                        timestamp: chrono::Utc::now(),
                        telemetry: Telemetry::TemperatureWithHumidity(TemperatureWithHumidity {
                            temperature: data.temperature,
                            humidity: data.relative_humidity,
                        }),
                    }),
                    Err(err) => Err(SmartPotError::DhtError(err.into())),
                },
            }
        }
    }
}

crate::mod_interface! {
    orphan use {
        private::DhtSensor,
    };
}
