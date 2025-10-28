//! Std-aware client runtime for interacting with the Miden network.
//!
//! This crate layers ergonomic, multi-threaded orchestration utilities on top of the
//! `miden-client-core` primitives. It provides a service-oriented API for running the core
//! light client in a background task, registering observers, executing transactions in sequence,
//! and exposing read handles for front-ends.

#![cfg_attr(not(feature = "std"), no_std)]

pub use miden_client_core::*;

#[cfg(feature = "std")]
pub mod service;
