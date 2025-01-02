extern crate alloc;

use alloc::string::String;

pub mod generated;

mod prover;
pub use prover::RemoteTransactionProver;

/// Contains the protobuf definitions
pub const PROTO_MESSAGES: &str = include_str!("../proto/api.proto");

/// ERRORS
/// ===============================================================================================

#[derive(Debug)]
pub enum RemoteTransactionProverError {
    /// Indicates that the provided gRPC server endpoint is invalid.
    InvalidEndpoint(String),

    /// Indicates that the connection to the server failed.
    ConnectionFailed(String),
}

impl std::fmt::Display for RemoteTransactionProverError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RemoteTransactionProverError::InvalidEndpoint(endpoint) => {
                write!(f, "Invalid endpoint: {}", endpoint)
            },
            RemoteTransactionProverError::ConnectionFailed(endpoint) => {
                write!(f, "Failed to connect to transaction prover at: {}", endpoint)
            },
        }
    }
}

impl core::error::Error for RemoteTransactionProverError {}
