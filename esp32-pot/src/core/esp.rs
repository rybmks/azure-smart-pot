//!
//! Sensor and Telemetry Management Module
//!

mod private {
    use crate::core::Result;
    use esp_idf_hal::gpio::{AnyIOPin, InputOutput, PinDriver};
    use one_wire_bus::OneWire;
    use smart_pot_core::*;

    pub type OneWireType<T> = OneWire<PinDriver<'static, T, InputOutput>>;

    /// # DhtConfig
    ///
    /// A configuration struct for setting up a DHT sensor (DHT11 or DHT22) with a specific pin.
    /// This struct allows specifying the GPIO pin used to connect the DHT sensor and the sensor type (DHT11 or DHT22).
    pub struct DhtConfig {
        ///   The GPIO pin connected to the DHT sensor.
        pub pin: AnyIOPin,
        ///   Specifies the type of the DHT sensor (`DHT11` or `DHT22`).
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
    pub enum DhtType {
        Dht11,
        Dht22,
    }

    /// # Sensor Trait
    ///
    /// The `Sensor` trait defines the behavior for sensor types, allowing them to return their name and read data.
    /// Any struct implementing this trait must provide an implementation for `get_name()` and `read_data()`.
    pub trait Sensor<'a> {
        ///   Returns the name of the sensor as a string.
        fn get_name(&self) -> String;

        ///   Reads data from the sensor and returns it as a `SensorData` struct.
        fn read_data(&mut self) -> Result<SensorData>;
    }
}

crate::mod_interface! {
    layer ds18b20;
    layer dht_sensor;
    layer wifi;
    layer bh1750;
    layer board;
    layer server;
    
    own use {
        Sensor,
        OneWireType,
        DhtType,
        DhtConfig,
    };
}
