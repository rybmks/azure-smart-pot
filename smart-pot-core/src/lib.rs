use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ## Updates
///
/// A struct representing configuration updates for sensor data processing.
#[derive(Debug, Serialize, Deserialize)]
pub struct Updates {
    ///   Optional flag to indicate if temperature values should be converted to Fahrenheit.
    pub convert_to_far: Option<bool>,
    ///   Optional telemetry interval (in seconds) determining how frequently telemetry is sent.
    pub telemetry_interval: Option<u32>,
}

/// # TemperatureWithHumidity
///
/// Struct representing temperature and humidity readings from a DHT sensor.
#[derive(Debug, Serialize)]
pub struct TemperatureWithHumidity {
    pub temperature: f32,
    pub humidity: f32,
}

/// # Telemetry
///
/// Enum representing different types of telemetry data that can be read from various sensors.
#[derive(Debug, Serialize)]
pub enum Telemetry {
    ///   Represents a temperature reading (in Celsius).
    Temperature(f32),
    ///   Represents both temperature and humidity readings.
    TemperatureWithHumidity(TemperatureWithHumidity),
    ///   Represents a light intensity reading (in lux).
    LightValue(f32),
}

/// # SensorData
///
/// A struct representing sensor data with a timestamp. This struct stores the telemetry
/// data (such as temperature, humidity, or light values) along with the time at which the data was collected.
#[derive(Debug, Serialize)]
pub struct SensorData {
    ///   The timestamp of when the data was collected. This field uses the `ts_seconds` format for serialization.
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    ///   The telemetry data associated with the sensor reading (temperature, humidity, or light value).
    pub telemetry: Telemetry,
}
