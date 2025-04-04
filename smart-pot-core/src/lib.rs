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

pub enum TemperatureUnits {
    Celsius,
    Fahrenheit,
}

/// Struct representing temperature (Celsius or Fahrenheit)
#[derive(Debug, Serialize, PartialEq)]
pub enum Temperature {
    CelsiusTemperature(f32),
    FahrenheitTemperature(f32),
}
impl Temperature {
    pub fn to_units(&mut self, unit: &TemperatureUnits) {
        match (&self, unit) {
            (Temperature::CelsiusTemperature(value), TemperatureUnits::Fahrenheit) => {
                *self = Temperature::FahrenheitTemperature(*value * 9.0 / 5.0 + 32.0)
            }
            (Temperature::FahrenheitTemperature(value), TemperatureUnits::Celsius) => {
                *self = Temperature::CelsiusTemperature((*value - 32.0) * 5.0 / 9.0)
            }
            _ => return,
        };
    }
}

/// # TemperatureWithHumidity
///
/// Struct representing temperature and humidity readings from a DHT sensor.
#[derive(Debug, Serialize)]
pub struct TemperatureWithHumidity {
    pub temperature: Temperature,
    pub humidity: f32,
}

/// # Telemetry
///
/// Enum representing different types of telemetry data that can be read from various sensors.
#[derive(Debug, Serialize)]
pub enum Telemetry {
    ///   Represents a temperature reading (in Celsius).
    Temperature(Temperature),
    ///   Represents both temperature and humidity readings.
    TemperatureWithHumidity(TemperatureWithHumidity),
    ///   Represents a light intensity reading (in lux).
    LightIntensityLux(f32),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        let mut temp = Temperature::CelsiusTemperature(0.0);
        temp.to_units(&TemperatureUnits::Fahrenheit);
        assert_eq!(temp, Temperature::FahrenheitTemperature(32.0));

        let mut temp = Temperature::CelsiusTemperature(100.0);
        temp.to_units(&TemperatureUnits::Fahrenheit);
        assert_eq!(temp, Temperature::FahrenheitTemperature(212.0));
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        let mut temp = Temperature::FahrenheitTemperature(32.0);
        temp.to_units(&TemperatureUnits::Celsius);
        assert_eq!(temp, Temperature::CelsiusTemperature(0.0));

        let mut temp = Temperature::FahrenheitTemperature(212.0);
        temp.to_units(&TemperatureUnits::Celsius);
        assert_eq!(temp, Temperature::CelsiusTemperature(100.0));
    }

    #[test]
    fn test_no_conversion_needed() {
        let mut temp = Temperature::CelsiusTemperature(25.0);
        temp.to_units(&TemperatureUnits::Celsius);
        assert_eq!(temp, Temperature::CelsiusTemperature(25.0));

        let mut temp = Temperature::FahrenheitTemperature(77.0);
        temp.to_units(&TemperatureUnits::Fahrenheit);
        assert_eq!(temp, Temperature::FahrenheitTemperature(77.0));
    }
}
