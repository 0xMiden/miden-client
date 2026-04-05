use alloc::string::String;

pub use crate::RemoteTransactionProver;

/// Default remote prover endpoint for testnet.
pub const TESTNET_PROVER_ENDPOINT: &str = "https://tx-prover.testnet.miden.io";

/// Default remote prover endpoint for devnet.
pub const DEVNET_PROVER_ENDPOINT: &str = "https://tx-prover.devnet.miden.io";

/// Default timeout for note transport connections (10 seconds).
pub const NOTE_TRANSPORT_DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// Configuration for lazy note transport initialization.
///
/// Since `GrpcNoteTransportClient::connect()` is async, this struct allows us to defer
/// the connection until `build()` is called.
pub struct NoteTransportConfig {
    pub endpoint: String,
    pub timeout_ms: u64,
}
