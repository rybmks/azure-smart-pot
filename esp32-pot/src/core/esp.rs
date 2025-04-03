//!
//! Sensor and Telemetry Management Module
//!

mod private {
    use crate::core::Result;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use esp_idf_hal::gpio::{AnyIOPin, InputOutput, PinDriver};
    use one_wire_bus::OneWire;
    use serde::Serialize;

    pub type OneWireType<T> = OneWire<PinDriver<'static, T, InputOutput>>;

    /// # DhtConfig
    ///
    /// A configuration struct for setting up a DHT sensor (DHT11 or DHT22) with a specific pin.
    /// This struct allows specifying the GPIO pin used to connect the DHT sensor and the sensor type (DHT11 or DHT22).
    ///
    /// ## Fields:
    /// - `pin`:  
    ///   The GPIO pin connected to the DHT sensor.
    /// - `dht_type`:  
    ///   Specifies the type of the DHT sensor (`DHT11` or `DHT22`).
    ///
    pub struct DhtConfig {
        pub pin: AnyIOPin,
        pub dht_type: DhtType,
    }

    impl DhtConfig {
        /// Constructs a new `DhtConfig` instance with the specified pin and DHT sensor type.
        ///
        /// # Parameters:
        /// - `pin`: The GPIO pin to which the DHT sensor is connected.
        /// - `dht_type`: The type of the DHT sensor (either `DHT11` or `DHT22`).
        ///
        /// # Returns:
        /// A new `DhtConfig` struct initialized with the provided values.
        pub fn new(pin: AnyIOPin, dht_type: DhtType) -> Self {
            DhtConfig { pin, dht_type }
        }
    }

    /// # DhtType
    ///
    /// Enumeration representing the two supported DHT sensor types: DHT11 and DHT22.
    ///
    /// - `Dht11`:  
    ///   Represents the DHT11 sensor, which provides less accurate readings and has a narrower temperature range.
    /// - `Dht22`:  
    ///   Represents the DHT22 sensor, which is more accurate and has a wider temperature range.
    pub enum DhtType {
        Dht11,
        Dht22,
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

    /// # Sensor Trait
    ///
    /// The `Sensor` trait defines the behavior for sensor types, allowing them to return their name and read data.
    /// Any struct implementing this trait must provide an implementation for `get_name()` and `read_data()`.
    ///
    /// ## Methods:
    /// - **`get_name()` → String**:  
    ///   Returns the name of the sensor as a string.
    /// - **`read_data()` → Result<SensorData>**:  
    ///   Reads data from the sensor and returns it as a `SensorData` struct.
    pub trait Sensor<'a> {
        fn get_name(&self) -> String;

        fn read_data(&mut self) -> Result<SensorData>;
    }
}

crate::mod_interface! {
    layer ds18b20;
    layer dht_sensor;
    layer wifi;
    layer bh1750;
    layer board;

    own use {
        Sensor,
        OneWireType,
        DhtType,
        DhtConfig,
    };

    orphan use {
        SensorData,
        Telemetry,
    };
}
