use alloc::string::String;

use miden_lib::utils::DeserializationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport layer is not enabled")]
    Disabled,
    #[error("connection error: {0}")]
    Connection(String),
    #[error("deserialization error: {0}")]
    Deserialization(#[from] DeserializationError),
    #[error("transport error: {0}")]
    Other(#[from] anyhow::Error),
}
