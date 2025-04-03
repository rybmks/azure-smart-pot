//! This module defines the data structures used for sensor data representation and configuration updates.
//!
//! It includes the following definitions:
//! - **TemperatureWithHumidity**:  
//!   A struct for temperature and humidity readings from a DHT sensor.
//! - **Telemetry**:  
//!   An enum for various telemetry types, including temperature, temperature with humidity, and light intensity.
//! - **SensorData**:  
//!   A struct that combines telemetry data with a timestamp indicating when the data was collected.
//! - **Updates**:  
//!   A struct for configuration updates, such as whether to convert temperatures to Fahrenheit and the telemetry interval.
//!
//! All types implement `Serialize` and `Deserialize` via `serde` to facilitate JSON conversion,
//! and the timestamp in `SensorData` is serialized using a seconds-since-epoch format.

mod private {
    use serde::{Deserialize, Serialize};
    use chrono::{DateTime, Utc};
    use chrono::serde::ts_seconds;

    /// ## TemperatureWithHumidity
    ///
    /// Struct representing temperature and humidity readings from a DHT sensor.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TemperatureWithHumidity {
        pub temperature: f32,
        pub humidity: f32,
    }

    /// ## Telemetry
    ///
    /// Enum representing different types of telemetry data that can be read from various sensors.
    ///
    /// ## Variants:
    /// - `Temperature(f32)`:  
    ///   Represents a temperature reading (in Celsius by default).
    /// - `TemperatureWithHumidity(TemperatureWithHumidity)`:  
    ///   Represents both temperature and humidity readings.
    /// - `LightValue(f32)`:  
    ///   Represents a light intensity reading (in lux).
    #[derive(Debug, Serialize, Deserialize)]
    pub enum Telemetry {
        Temperature(f32),
        TemperatureWithHumidity(TemperatureWithHumidity),
        LightValue(f32),
    }

    /// ## SensorData
    ///
    /// A struct representing sensor data with a timestamp. This struct stores the telemetry
    /// data (such as temperature, humidity, or light values) along with the time at which the data was collected.
    ///
    /// ## Fields:
    /// - `timestamp`:  
    ///   The timestamp of when the data was collected, serialized as seconds since the epoch.
    /// - `telemetry`:  
    ///   The telemetry data associated with the sensor reading.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SensorData {
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub telemetry: Telemetry,
    }

    /// ## Updates
    ///
    /// A struct representing configuration updates for sensor data processing.
    ///
    /// ## Fields:
    /// - `convert_to_far`:  
    ///   Optional flag to indicate if temperature values should be converted to Fahrenheit.
    /// - `telemetry_interval`:  
    ///   Optional telemetry interval (in seconds) determining how frequently telemetry is sent.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Updates {
        pub convert_to_far: Option<bool>,
        pub telemetry_interval: Option<u32>,
    }
}

crate::mod_interface! {
    orphan use {
        Updates,
        SensorData,
        Telemetry,
        TemperatureWithHumidity
    };
}
