mod private {
    use thiserror::Error;
    
    #[derive(Error, Debug)]
    pub enum Error {
        #[error("IO error:\n{0}")]
        IoError(#[from] std::io::Error),

        #[error("{0}")]
        EnvError(#[from] std::env::VarError),

        #[error("{0}")]
        HubError(String),

        #[error("{0}")]
        ParseError(#[from] serde_json::Error),

        #[error("{0}")]
        ReqwestError(#[from] reqwest::Error)
    }
}

crate::mod_interface! {
    orphan use {
        Error
    };
}