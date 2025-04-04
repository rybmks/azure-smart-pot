//! This module defines the custom error types used throughout the application.
//!
//! It leverages the `thiserror` crate to provide a unified error type that encapsulates errors from
//! various sources including I/O operations, environment variable access, JSON parsing, HTTP requests, and
//! IoT Hub related operations.
//!
//! The `Error` enum includes the following variants:
//! - `IoError`: Wraps errors originating from standard I/O operations.
//! - `EnvError`: Wraps errors related to environment variable access.
//! - `HubError`: Represents errors encountered when connecting to or interacting with the IoT Hub.
//! - `ParseError`: Wraps errors from JSON parsing using `serde_json`.
//! - `ReqwestError`: Wraps errors from HTTP requests using `reqwest`.

mod private {
    use thiserror::Error;

    /// Custom error type for the application.
    #[derive(Error, Debug)]
    pub enum Error {
        /// Represents errors that occur when accessing environment variables.
        #[error(transparent)]
        EnvError(#[from] std::env::VarError),

        /// Represents errors related to IoT Hub operations.
        #[error("{0}")]
        HubError(String),

        /// Represents error related to int parse error.
        #[error(transparent)]
        ParseError(#[from] std::num::ParseIntError),
    }
}

crate::mod_interface! {
    orphan use {
        Error
    };
}
