//! Background service utilities for the Miden client.
//!
//! The service owns a [`miden_client_core::Client`] and runs a loop that can:
//! - continuously sync with the chain on a configured interval,
//! - trigger user-defined handlers after each sync,
//! - enqueue transactions so they execute sequentially, and
//! - update tracked tags via the same serialized command queue.

mod config;
use std::error::Error;

pub use config::ServiceConfig;

mod runtime;
use miden_client_core::ClientError;
pub use runtime::{AsyncHandler, BlockingHandler, ClientHandle, ClientService};
use thiserror::Error;

/// Errors that can occur while interacting with the client service.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("client error")]
    Client(#[source] ClientError),
    #[error("service channel is closed")]
    ChannelClosed,
    #[error("service has already shut down")]
    Shutdown,
    #[error("handler error")]
    HandlerError { source: Option<Box<dyn Error>> },
}

impl From<ClientError> for ServiceError {
    fn from(err: ClientError) -> Self {
        Self::Client(err)
    }
}
