/// IndexedDB-backed Store implementation for miden-client (WASM)

#[cfg(target_arch = "wasm32")]
mod web_store;

#[cfg(target_arch = "wasm32")]
pub use web_store::WebStore;
