//!
//! Error Handling Module
//!

mod private {
    use bh1750::BH1750Error;
    use dht_sensor::DhtError;
    use esp_idf_hal::io::EspIOError;
    use esp_idf_sys::EspError;
    use one_wire_bus::OneWireError;
    use std::error::Error;
    use std::fmt;
    use thiserror::Error;
    /// # SmartPotError
    ///
    /// Enum representing all possible errors in the SmartPot system. Each variant corresponds to a specific type of error,
    /// such as IO errors, hardware communication issues, or data parsing errors.
    #[derive(Error, Debug)]
    pub enum SmartPotError {
        #[error("EspIO error:\n{0}")]
        EspIoError(#[from] EspIOError),

        #[error("EspIO error:\n{0}")]
        SerializationError(#[from] serde_json::error::Error),

        #[error("Esp error:\n{0}")]
        EspError(#[from] EspError),

        #[error("OneWire error:\n{0}")]
        OneWireError(#[from] OneWireErrorWrapper),

        #[error("Bh1750 error:\n{0:?}")]
        Bh1750Error(#[from] Bh1750ErrorWrapper),

        #[error("Dht error:\n{0}")]
        DhtError(#[from] DhtErrorWrapper),

        #[error("Board lock failed")]
        MutexError,
        
        #[error("Some error: {0}")]
        SomeError(#[from] Box<dyn std::error::Error>),
    }

    /// # DhtErrorWrapper
    ///
    /// A wrapper for `DhtError`, specifically for handling errors related to DHT temperature and humidity sensors.
    /// It provides a user-friendly display format for errors.
    #[derive(Debug)]
    pub struct DhtErrorWrapper(DhtError<EspError>);

    impl fmt::Display for DhtErrorWrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "DHT sensor error: {:?}", self.0)
        }
    }
    impl Error for DhtErrorWrapper {}
    impl From<DhtError<EspError>> for DhtErrorWrapper {
        fn from(err: DhtError<EspError>) -> Self {
            DhtErrorWrapper(err)
        }
    }

    /// # OneWireErrorWrapper
    ///
    /// A wrapper for `OneWireError`, used to handle errors related to OneWire communication. It provides a custom display
    /// format for errors encountered during OneWire operations.
    #[derive(Debug)]
    pub struct OneWireErrorWrapper(OneWireError<EspError>);

    impl fmt::Display for OneWireErrorWrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "OneWire Error: {:?}", self.0)
        }
    }
    impl Error for OneWireErrorWrapper {}

    impl From<OneWireError<EspError>> for OneWireErrorWrapper {
        fn from(err: OneWireError<EspError>) -> Self {
            OneWireErrorWrapper(err)
        }
    }

    /// # Bh1750ErrorWrapper
    ///
    /// A wrapper for `BH1750Error`, used for handling errors that occur during interactions with BH1750 light sensors.
    /// It also provides a custom display format for errors related to I2C communication issues.
    #[derive(Debug)]
    pub struct Bh1750ErrorWrapper(BH1750Error<esp_idf_hal::i2c::I2cError>);

    impl fmt::Display for Bh1750ErrorWrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "BH1750 Error: {:?}", self.0)
        }
    }

    impl Error for Bh1750ErrorWrapper {}

    impl From<BH1750Error<esp_idf_hal::i2c::I2cError>> for Bh1750ErrorWrapper {
        fn from(err: BH1750Error<esp_idf_hal::i2c::I2cError>) -> Self {
            Bh1750ErrorWrapper(err)
        }
    }

    /// Shortcut for std::result::Result<T, SmartPotError>
    pub type Result<T> = std::result::Result<T, SmartPotError>;
}

crate::mod_interface! {
    orphan use {
        SmartPotError,
        Result,
        Bh1750ErrorWrapper
    };
}
