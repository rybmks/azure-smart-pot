//!
//! BH1750 Light Sensor Module for SmartPot
//!

mod private {
    use crate::core::esp::Sensor;
    use crate::core::{Result, SensorData, SmartPotError, Telemetry};
    use bh1750::*;
    use esp_idf_hal::delay::Delay;
    use esp_idf_hal::i2c::I2cDriver;

    /// # Bh1750
    ///
    /// Structure to interact with the BH1750 light sensor using the ESP32's I2C bus.
    ///
    /// ## Type Parameters:
    /// - `'a`: The lifetime of the I2C driver.
    pub struct Bh1750<'a> {
        sensor: bh1750::BH1750<I2cDriver<'a>, Delay>,
        resolution: Resolution,
    }

    impl<'a> Bh1750<'a> {
        /// Constructs a new `Bh1750` instance.
        ///
        /// # Parameters:
        /// - `i2c_driver`: The I2C driver used to communicate with the BH1750 sensor.
        /// - `resolution`: The resolution of the sensor's light measurements. Can be `Resolution::Low`, `Resolution::High` or `Resolution::High2`.
        ///
        /// # Returns:
        /// A new instance of the `Bh1750` struct configured with the provided I2C driver and resolution.
        pub fn new(i2c_driver: I2cDriver<'a>, resolution: Resolution) -> Self {
            let delay = esp_idf_hal::delay::Delay::new_default();

            let sensor = bh1750::BH1750::new(i2c_driver, delay, false);
            Bh1750 { sensor, resolution }
        }
    }

    impl<'a> Sensor<'a> for Bh1750<'a> {
        /// Returns the name of the sensor.
        fn get_name(&self) -> String {
            "Bh1750 light sensor".to_string()
        }

        /// Reads light intensity data from the BH1750 sensor.
        ///
        /// This function gets a one-time light intensity measurement from the sensor using the configured resolution.
        /// It returns the data as `SensorData`, which includes the timestamp and the light intensity value.
        ///
        /// # Returns:
        /// - `Result<SensorData>`: A `SensorData` instance containing the timestamp and light intensity reading.
        ///
        /// # Errors:
        /// This function returns a `SmartPotError::Bh1750Error` if there is an error communicating with the sensor.
        fn read_data(&mut self) -> Result<SensorData> {
            let data = self
                .sensor
                .get_one_time_measurement(self.resolution)
                .map_err(|err| SmartPotError::Bh1750Error(err.into()))?;

            Ok(SensorData {
                timestamp: chrono::Utc::now(),
                telemetry: Telemetry::LightValue(data),
            })
        }
    }
}

crate::mod_interface! {
    orphan use {
        private::Bh1750,
    };
}
