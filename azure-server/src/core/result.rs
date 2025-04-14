//! This module defines a custom `Result` type alias for the application.
//!
//! The alias wraps the standard library's `Result` type by specifying the error type as the custom
//! `Error` defined in `crate::core::Error`. This simplifies function signatures across the application,
//! ensuring consistent error handling.

mod private {
    use crate::core::Error;

    /// Custom result type alias.
    ///
    /// This type alias is used throughout the application to return a result that uses the custom
    /// `Error` type defined in `crate::core::Error` for error cases.
    pub type Result<T> = std::result::Result<T, Error>;
}

crate::mod_interface! {
    orphan use {
        Result
    };
}
