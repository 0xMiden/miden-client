#![cfg_attr(not(feature = "std"), no_std)]

pub use miden_client_core::*;

#[cfg(feature = "std")]
pub mod service;

#[cfg(feature = "std")]
pub use service::{
    ClientHandle,
    ClientRuntime,
    ClientServiceConfig,
    ClientServiceError,
    HandlerError,
    HandlerId,
    SyncEvent,
};
