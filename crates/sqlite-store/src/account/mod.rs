// We use const QUERY: &str for SQL queries to increase readability. This style
// triggers this clippy lint error.
#![allow(clippy::items_after_statements)]

#[cfg(not(target_arch = "wasm32"))]
mod accounts;
#[cfg(not(target_arch = "wasm32"))]
mod helpers;
pub(crate) mod shared;
#[cfg(not(target_arch = "wasm32"))]
mod storage;
#[cfg(not(target_arch = "wasm32"))]
mod vault;

#[cfg(test)]
mod tests;
