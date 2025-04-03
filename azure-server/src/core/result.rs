mod private {
    use crate::core::Error;

    pub type Result<T> = std::result::Result<T, Error>;
}

crate::mod_interface! {
    orphan use {
        Result
    };
}