//!
//! DS18B20 Temperature Sensor Module
//!

mod private {
    use crate::core::esp::{OneWireType, Sensor};
    use crate::core::{Result, SmartPotError};
    use esp_idf_hal::gpio::{InputPin, OutputPin};
    use esp_idf_sys::EspError;
    use smart_pot_core::*;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// # Ds18B20Sensor
    ///
    /// Structure to interact with the DS18B20 temperature sensor using the OneWire protocol.
    pub struct Ds18B20Sensor<T>
    where
        T: InputPin + OutputPin,
    {
        one_wire_bus: Arc<Mutex<OneWireType<T>>>,
        ds_address: ds18b20::Ds18b20,
    }

    impl<T> Ds18B20Sensor<T>
    where
        T: InputPin + OutputPin,
    {
        /// Finds all DS18B20 sensors connected to the OneWire bus.
        ///
        /// This method searches the OneWire bus for connected DS18B20 sensors and returns a list of `Ds18B20Sensor`
        /// instances that represent the found sensors.
        ///
        /// # Parameters:
        /// - `one_wire_bus`: A reference to the OneWire bus used for communication with the DS18B20 sensors.
        ///
        /// # Returns:
        /// - `Result<Vec<Box<Self>>>`: A vector where each element represents a DS18B20 sensor.
        ///
        /// # Errors:
        /// Returns a `SmartPotError::OneWireError` if there is an error during the search process or while initializing sensors.
        pub fn find_all(one_wire_bus: Arc<Mutex<OneWireType<T>>>) -> Result<Vec<Box<Self>>>
        where
            Self: Sized,
        {
            let mut search_state = None;

            let mut delay = esp_idf_hal::delay::Delay::new_default();
            let mut ds_sensors: Vec<ds18b20::Ds18b20> = Vec::new();

            {
                let mut wire_bus_locked =
                    one_wire_bus.lock().map_err(|_| SmartPotError::MutexError)?;

                while let Some((device_address, state)) = wire_bus_locked
                    .device_search(search_state.as_ref(), false, &mut delay)
                    .map_err(|e| SmartPotError::OneWireError(e.into()))?
                {
                    search_state = Some(state);

                    if device_address.family_code() != ds18b20::FAMILY_CODE {
                        continue;
                    }

                    log::trace!("Found ds18b20: {:?}", device_address);
                    let sensor = ds18b20::Ds18b20::new::<EspError>(device_address)
                        .map_err(|e| SmartPotError::OneWireError(e.into()))?;

                    ds_sensors.push(sensor);
                }
            }

            let sensors = ds_sensors
                .into_iter()
                .map(|s| {
                    Box::new(Ds18B20Sensor {
                        one_wire_bus: one_wire_bus.clone(),
                        ds_address: s,
                    })
                })
                .collect::<Vec<Box<Ds18B20Sensor<T>>>>();
            Ok(sensors)
        }
    }

    impl<T> Sensor<'_> for Ds18B20Sensor<T>
    where
        T: InputPin + OutputPin,
    {
        /// Returns the name of the sensor.
        fn get_name(&self) -> String {
            "Ds18B20".to_string()
        }

        /// Reads temperature data from the DS18B20 sensor.
        ///
        /// This method starts a simultaneous temperature measurement on the sensor, waits for the measurement to
        /// complete, and then reads the temperature data. It returns the data as `SensorData`, which includes
        /// a timestamp and the temperature reading.
        ///
        /// # Returns:
        /// - `Result<SensorData>`: A `SensorData` instance containing the timestamp and temperature reading.
        ///
        /// # Errors:
        /// Returns a `SmartPotError::OneWireError` if there is an error starting the measurement or reading the data.
        fn read_data(&mut self) -> Result<SensorData> {
            let mut delay = esp_idf_hal::delay::Delay::new_default();

            let mut locked_wire_bus = self
                .one_wire_bus
                .lock()
                .map_err(|_| SmartPotError::MutexError)?;

            ds18b20::start_simultaneous_temp_measurement(&mut locked_wire_bus, &mut delay)
                .map_err(|e| SmartPotError::OneWireError(e.into()))?;

            ds18b20::Resolution::Bits12.delay_for_measurement_time(&mut delay);

            let sensor_data = self
                .ds_address
                .read_data(&mut locked_wire_bus, &mut delay)
                .map_err(|e| SmartPotError::OneWireError(e.into()))?;

            Ok(SensorData {
                timestamp: chrono::Utc::now(),
                telemetry: Telemetry::Temperature(Temperature::CelsiusTemperature(
                    sensor_data.temperature,
                )),
            })
        }
    }
}

crate::mod_interface! {
    orphan use {
        Ds18B20Sensor,
    };
}
