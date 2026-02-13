//! Native Node.js addon for interacting with the Miden network.
//!
//! This crate provides a napi-rs binding layer around the core Rust `miden-client` and
//! `miden-client-sqlite-store`, exposing a native Node.js addon. Node.js users get a
//! native binary instead of WASM, while browser users continue using the existing
//! WASM + idxdb-store path.

#[macro_use]
extern crate napi_derive;

pub mod client;
pub mod error;
pub mod models;
