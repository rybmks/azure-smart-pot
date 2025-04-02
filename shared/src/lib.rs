use serde::Serialize;
use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;

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
///
/// ## Variants:
/// - `Temperature(f32)`:  
///   Represents a temperature reading (in Celsius).
/// - `TemperatureWithHumidity(TemperatureWithHumidity)`:  
///   Represents both temperature and humidity readings.
/// - `LightValue(f32)`:  
///   Represents a light intensity reading (in lux).
#[derive(Debug, Serialize)]
pub enum Telemetry {
    Temperature(f32),
    TemperatureWithHumidity(TemperatureWithHumidity),
    LightValue(f32),
}

/// # SensorData
///
/// A struct representing sensor data with a timestamp. This struct stores the telemetry
/// data (such as temperature, humidity, or light values) along with the time at which the data was collected.
///
/// ## Fields:
/// - `timestamp`:  
///   The timestamp of when the data was collected. This field uses the `ts_seconds` format for serialization.
/// - `telemetry`:  
///   The telemetry data associated with the sensor reading (temperature, humidity, or light value).
#[derive(Debug, Serialize)]
pub struct SensorData {
    #[serde(with = "ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub telemetry: Telemetry,
}