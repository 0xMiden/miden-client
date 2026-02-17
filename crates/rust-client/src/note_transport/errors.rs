use alloc::boxed::Box;
use alloc::string::String;
use core::error::Error;

use miden_protocol::utils::DeserializationError;
use thiserror::Error;

use crate::errors::ErrorCode;

#[derive(Debug, Error)]
pub enum NoteTransportError {
    #[error("note transport is not enabled")]
    Disabled,
    #[error("connection error: {0}")]
    Connection(#[source] Box<dyn Error + Send + Sync + 'static>),
    #[error("deserialization error: {0}")]
    Deserialization(#[from] DeserializationError),
    #[error("note transport network error: {0}")]
    Network(String),
}

impl ErrorCode for NoteTransportError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::Disabled => "MIDEN-NT-001",
            Self::Connection(_) => "MIDEN-NT-002",
            Self::Deserialization(_) => "MIDEN-NT-003",
            Self::Network(_) => "MIDEN-NT-004",
        }
    }
}
