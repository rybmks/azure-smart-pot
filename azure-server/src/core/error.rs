mod private {
    use thiserror::Error;
    
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("IO error:\n{0}")]
        IoError(#[from] std::io::Error),

        #[error("{0}")]
        EnvError(#[from] std::env::VarError),

        #[error("{0}")]
        HubError(String)
    }
}

crate::mod_interface! {
    orphan use {
        Error
    };
}