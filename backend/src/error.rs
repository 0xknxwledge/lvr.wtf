use thiserror::Error;
use tokio::task::JoinError;
use parquet::errors::ParquetError;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Database configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Processing error: {0}")]
    Processing(String),
    
    #[error("IO error: {0}")]
    IO(String),  
    
    #[error("JSON error: {0}")]
    Json(String), 


    #[error("Parquet error: {0}")]
    Parquet(String),

    #[error("General error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<ParquetError> for Error {
    fn from(err: ParquetError) -> Self {
        Error::Parquet(err.to_string())
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}