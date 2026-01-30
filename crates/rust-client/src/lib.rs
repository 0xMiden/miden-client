#![cfg_attr(docsrs, feature(doc_cfg))]

//! A no_std-compatible client library for interacting with the Miden network.
//!
//! This crate provides a lightweight client that handles connections to the Miden node, manages
//! accounts and their state, and facilitates executing, proving, and submitting transactions.
//!
//! For a protocol-level overview and guides for getting started, please visit the official
//! [Miden docs](https://0xMiden.github.io/miden-docs/).
//!
//! ## Overview
//!
//! The library is organized into several key modules:
//!
//! - **Accounts:** Provides types for managing accounts. Once accounts are tracked by the client,
//!   their state is updated with every transaction and validated during each sync.
//!
//! - **Notes:** Contains types and utilities for working with notes in the Miden client.
//!
//! - **RPC:** Facilitates communication with Miden node, exposing RPC methods for syncing state,
//!   fetching block headers, and submitting transactions.
//!
//! - **Store:** Defines and implements the persistence layer for accounts, transactions, notes, and
//!   other entities.
//!
//! - **Sync:** Provides functionality to synchronize the local state with the current state on the
//!   Miden network.
//!
//! - **Transactions:** Offers capabilities to build, execute, prove, and submit transactions.
//!
//! Additionally, the crate re-exports several utility modules:
//!
//! - **Assembly:** Types for working with Miden Assembly.
//! - **Assets:** Types and utilities for working with assets.
//! - **Auth:** Authentication-related types and functionalities.
//! - **Blocks:** Types for handling block headers.
//! - **Crypto:** Cryptographic types and utilities, including random number generators.
//! - **Utils:** Miscellaneous utilities for serialization and common operations.
//!
//! The library is designed to work in both `no_std` and `std` environments and is
//! configurable via Cargo features.
//!
//! ## Usage
//!
//! To use the Miden client library in your project, add it as a dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! miden-client = "0.10"
//! ```
//!
//! ## Example
//!
//! Below is a brief example illustrating how to instantiate the client:
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use miden_client::crypto::RpoRandomCoin;
//! use miden_client::keystore::FilesystemKeyStore;
//! use miden_client::note_transport::NOTE_TRANSPORT_DEFAULT_ENDPOINT;
//! use miden_client::note_transport::grpc::GrpcNoteTransportClient;
//! use miden_client::rpc::{Endpoint, GrpcClient};
//! use miden_client::store::Store;
//! use miden_client::{Client, ExecutionOptions, Felt};
//! use miden_client_sqlite_store::SqliteStore;
//! use miden_protocol::crypto::rand::FeltRng;
//! use miden_protocol::{MAX_TX_EXECUTION_CYCLES, MIN_TX_EXECUTION_CYCLES};
//! use rand::Rng;
//!
//! # pub async fn create_test_client() -> Result<(), Box<dyn std::error::Error>> {
//! // Create the SQLite store from the client configuration.
//! let sqlite_store = SqliteStore::new("path/to/store".try_into()?).await?;
//! let store = Arc::new(sqlite_store);
//!
//! // Generate a random seed for the RpoRandomCoin.
//! let mut rng = rand::rng();
//! let coin_seed: [u64; 4] = rng.random();
//!
//! // Initialize the random coin using the generated seed.
//! let rng = RpoRandomCoin::new(coin_seed.map(Felt::new).into());
//! let keystore = FilesystemKeyStore::new("path/to/keys/directory".try_into()?)?;
//!
//! // Determine the number of blocks to consider a transaction stale.
//! // 20 is simply an example value.
//! let tx_graceful_blocks = Some(20);
//! // Determine the maximum number of blocks that the client can be behind from the network.
//! // 256 is simply an example value.
//! let max_block_number_delta = Some(256);
//!
//! // Optionally, connect to the note transport network to exchange private notes.
//! let note_transport_api =
//!     GrpcNoteTransportClient::connect(NOTE_TRANSPORT_DEFAULT_ENDPOINT.to_string(), 10_000)
//!         .await?;
//!
//! // Instantiate the client using a gRPC client
//! let endpoint = Endpoint::new("https".into(), "localhost".into(), Some(57291));
//! let client: Client<FilesystemKeyStore> = Client::new(
//!     Arc::new(GrpcClient::new(&endpoint, 10_000)),
//!     Box::new(rng),
//!     store,
//!     Some(Arc::new(keystore)), // or None if no authenticator is needed
//!     ExecutionOptions::new(
//!         Some(MAX_TX_EXECUTION_CYCLES),
//!         MIN_TX_EXECUTION_CYCLES,
//!         false,
//!         false, // Set to true for debug mode, if needed.
//!     )
//!     .unwrap(),
//!     tx_graceful_blocks,
//!     max_block_number_delta,
//!     Some(Arc::new(note_transport_api)),
//!     None, // or Some(Arc::new(prover)) for a custom prover
//! )
//! .await
//! .unwrap();
//!
//! # Ok(())
//! # }
//! ```
//!
//! For additional usage details, configuration options, and examples, consult the documentation for
//! each module.

#![no_std]

#[macro_use]
extern crate alloc;
use alloc::boxed::Box;

#[cfg(feature = "std")]
extern crate std;

pub mod account;
pub mod keystore;
pub mod note;
pub mod note_transport;
pub mod rpc;
pub mod settings;
pub mod store;
pub mod sync;
pub mod transaction;
pub mod utils;

#[cfg(feature = "std")]
pub mod builder;

#[cfg(feature = "testing")]
mod test_utils;

pub mod errors;

use miden_protocol::block::BlockNumber;
pub use miden_protocol::utils::{Deserializable, Serializable, SliceReader};

// RE-EXPORTS
// ================================================================================================

pub mod notes {
    pub use miden_protocol::note::NoteFile;
}

/// Provides types and utilities for working with Miden Assembly.
pub mod assembly {
    pub use miden_protocol::MastForest;
    pub use miden_protocol::assembly::debuginfo::SourceManagerSync;
    pub use miden_protocol::assembly::diagnostics::Report;
    pub use miden_protocol::assembly::diagnostics::reporting::PrintDiagnostic;
    pub use miden_protocol::assembly::mast::MastNodeExt;
    pub use miden_protocol::assembly::{
        Assembler,
        DefaultSourceManager,
        Library,
        Module,
        ModuleKind,
        Path,
    };
    pub use miden_standards::code_builder::CodeBuilder;
}

/// Provides types and utilities for working with assets within the Miden network.
pub mod asset {
    pub use miden_protocol::account::delta::{
        AccountStorageDelta,
        AccountVaultDelta,
        FungibleAssetDelta,
        NonFungibleAssetDelta,
        NonFungibleDeltaAction,
    };
    pub use miden_protocol::asset::{
        Asset,
        AssetVault,
        AssetWitness,
        FungibleAsset,
        NonFungibleAsset,
        NonFungibleAssetDetails,
        TokenSymbol,
    };
}

/// Provides authentication-related types and functionalities for the Miden
/// network.
pub mod auth {
    pub use miden_protocol::account::auth::{
        AuthScheme as AuthSchemeId,
        AuthSecretKey,
        PublicKey,
        PublicKeyCommitment,
        Signature,
    };
    pub use miden_standards::AuthScheme;
    pub use miden_standards::account::auth::{AuthEcdsaK256Keccak, AuthFalcon512Rpo, NoAuth};
    pub use miden_tx::auth::{BasicAuthenticator, SigningInputs, TransactionAuthenticator};

    pub const RPO_FALCON_SCHEME_ID: AuthSchemeId = AuthSchemeId::Falcon512Rpo;
    pub const ECDSA_K256_KECCAK_SCHEME_ID: AuthSchemeId = AuthSchemeId::EcdsaK256Keccak;
}

/// Provides types for working with blocks within the Miden network.
pub mod block {
    pub use miden_protocol::block::{BlockHeader, BlockNumber};
}

/// Provides cryptographic types and utilities used within the Miden rollup
/// network. It re-exports commonly used types and random number generators like `FeltRng` from
/// the `miden_standards` crate.
pub mod crypto {
    pub mod rpo_falcon512 {
        pub use miden_protocol::crypto::dsa::falcon512_rpo::{PublicKey, SecretKey, Signature};
    }
    pub use miden_protocol::crypto::hash::blake::{Blake3_160, Blake3Digest};
    pub use miden_protocol::crypto::hash::rpo::Rpo256;
    pub use miden_protocol::crypto::merkle::mmr::{
        Forest,
        InOrderIndex,
        MmrDelta,
        MmrPeaks,
        MmrProof,
        PartialMmr,
    };
    pub use miden_protocol::crypto::merkle::smt::{LeafIndex, SMT_DEPTH, SmtLeaf, SmtProof};
    pub use miden_protocol::crypto::merkle::store::MerkleStore;
    pub use miden_protocol::crypto::merkle::{MerklePath, MerkleTree, NodeIndex, SparseMerklePath};
    pub use miden_protocol::crypto::rand::{FeltRng, RpoRandomCoin};
}

/// Provides types for working with addresses within the Miden network.
pub mod address {
    pub use miden_protocol::address::{
        Address,
        AddressId,
        AddressInterface,
        CustomNetworkId,
        NetworkId,
        RoutingParameters,
    };
}

/// Provides types for working with the virtual machine within the Miden network.
pub mod vm {
    // TODO: Remove this re-export once miden-protocol exposes PackageKind/ProcedureExport.
    pub use miden_mast_package::{PackageKind, ProcedureExport};
    pub use miden_protocol::vm::{
        AdviceInputs,
        AdviceMap,
        AttributeSet,
        MastArtifact,
        Package,
        PackageExport,
        PackageManifest,
        Program,
        QualifiedProcedureName,
        Section,
        SectionId,
    };
}

pub use async_trait::async_trait;
pub use errors::*;
use miden_protocol::assembly::{DefaultSourceManager, SourceManagerSync};
pub use miden_protocol::{
    EMPTY_WORD,
    Felt,
    MAX_TX_EXECUTION_CYCLES,
    MIN_TX_EXECUTION_CYCLES,
    ONE,
    PrettyPrint,
    StarkField,
    Word,
    ZERO,
};
pub use miden_remote_prover_client::remote_prover::tx_prover::RemoteTransactionProver;
pub use miden_tx::ExecutionOptions;

/// Provides test utilities for working with accounts and account IDs
/// within the Miden network. This module is only available when the `testing` feature is
/// enabled.
#[cfg(feature = "testing")]
pub mod testing {
    pub use miden_protocol::testing::account_id;
    pub use miden_standards::testing::note::NoteBuilder;
    pub use miden_standards::testing::*;
    pub use miden_testing::*;

    pub use crate::test_utils::*;
}

use alloc::sync::Arc;

use miden_protocol::crypto::rand::FeltRng;
use miden_tx::LocalTransactionProver;
use miden_tx::auth::TransactionAuthenticator;
use rand::RngCore;
use rpc::NodeRpcClient;
use store::Store;

use crate::note_transport::{NoteTransportClient, init_note_transport_cursor};
use crate::transaction::TransactionProver;

// MIDEN CLIENT
// ================================================================================================

/// A light client for connecting to the Miden network.
///
/// Miden client is responsible for managing a set of accounts. Specifically, the client:
/// - Keeps track of the current and historical states of a set of accounts and related objects such
///   as notes and transactions.
/// - Connects to a Miden node to periodically sync with the current state of the network.
/// - Executes, proves, and submits transactions to the network as directed by the user.
pub struct Client<AUTH> {
    /// The client's store, which provides a way to write and read entities to provide persistence.
    store: Arc<dyn Store>,
    /// An instance of [`FeltRng`] which provides randomness tools for generating new keys,
    /// serial numbers, etc.
    rng: ClientRng,
    /// An instance of [`NodeRpcClient`] which provides a way for the client to connect to the
    /// Miden node.
    rpc_api: Arc<dyn NodeRpcClient>,
    /// An instance of a [`TransactionProver`] which will be the default prover for the
    /// client.
    tx_prover: Arc<dyn TransactionProver + Send + Sync>,
    /// An instance of a [`TransactionAuthenticator`] which will be used by the transaction
    /// executor whenever a signature is requested from within the VM.
    authenticator: Option<Arc<AUTH>>,
    /// Shared source manager used to retain MASM source information for assembled programs.
    source_manager: Arc<dyn SourceManagerSync>,
    /// Options that control the transaction executor’s runtime behaviour (e.g. debug mode).
    exec_options: ExecutionOptions,
    /// The number of blocks that are considered old enough to discard pending transactions.
    tx_graceful_blocks: Option<u32>,
    /// Maximum number of blocks the client can be behind the network for transactions and account
    /// proofs to be considered valid.
    max_block_number_delta: Option<u32>,
    /// An instance of [`NoteTransportClient`] which provides a way for the client to connect to
    /// the Miden Note Transport network.
    note_transport_api: Option<Arc<dyn NoteTransportClient>>,
}

/// Construction and access methods.
impl<AUTH> Client<AUTH>
where
    AUTH: TransactionAuthenticator,
{
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Returns a new instance of [`Client`].
    ///
    /// ## Arguments
    ///
    /// - `rpc_api`: An instance of [`NodeRpcClient`] which provides a way for the client to connect
    ///   to the Miden node.
    /// - `rng`: An instance of [`FeltRng`] which provides randomness tools for generating new keys,
    ///   serial numbers, etc. This can be any RNG that implements [`ClientFeltRng`] (ie `FeltRng`
    ///   + `Send` + `Sync`).
    /// - `store`: An instance of [`Store`], which provides a way to write and read entities to
    ///   provide persistence.
    /// - `authenticator`: Defines the transaction authenticator that will be used by the
    ///   transaction executor whenever a signature is requested from within the VM.
    /// - `exec_options`: Options that control the transaction executor’s runtime behaviour (e.g.
    ///   debug mode).
    /// - `tx_graceful_blocks`: The number of blocks that are considered old enough to discard
    ///   pending transactions.
    /// - `max_block_number_delta`: Determines the maximum number of blocks that the client can be
    ///   behind the network for transactions and account proofs to be considered valid.
    /// - `note_transport_api`: An instance of [`NoteTransportClient`] which provides a way for the
    ///   client to connect to the Miden Note Transport network.
    /// - `tx_prover`: An optional instance of [`TransactionProver`] which will be used as the
    ///   default prover for the client. If not provided, a default local prover will be created.
    ///
    /// # Errors
    ///
    /// Returns an error if the client couldn't be instantiated.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        rpc_api: Arc<dyn NodeRpcClient>,
        rng: ClientRngBox,
        store: Arc<dyn Store>,
        authenticator: Option<Arc<AUTH>>,
        exec_options: ExecutionOptions,
        tx_graceful_blocks: Option<u32>,
        max_block_number_delta: Option<u32>,
        note_transport_api: Option<Arc<dyn NoteTransportClient>>,
        tx_prover: Option<Arc<dyn TransactionProver + Send + Sync>>,
    ) -> Result<Self, ClientError> {
        let tx_prover: Arc<dyn TransactionProver + Send + Sync> =
            tx_prover.unwrap_or_else(|| Arc::new(LocalTransactionProver::default()));

        if let Some((genesis, _)) = store.get_block_header_by_num(BlockNumber::GENESIS).await? {
            // Set the genesis commitment in the RPC API client for future requests.
            rpc_api.set_genesis_commitment(genesis.commitment()).await?;
        }

        // Initialize the note transport cursor if the client uses it
        if note_transport_api.is_some() {
            init_note_transport_cursor(store.clone()).await?;
        }

        // Pre-fetch and cache RPC limits from the node
        let _ = rpc_api.get_rpc_limits().await;

        let source_manager: Arc<dyn SourceManagerSync> = Arc::new(DefaultSourceManager::default());

        Ok(Self {
            store,
            rng: ClientRng::new(rng),
            rpc_api,
            tx_prover,
            authenticator,
            source_manager,
            exec_options,
            tx_graceful_blocks,
            max_block_number_delta,
            note_transport_api,
        })
    }

    /// Returns true if the client is in debug mode.
    pub fn in_debug_mode(&self) -> bool {
        self.exec_options.enable_debugging()
    }

    /// Returns an instance of the `CodeBuilder`
    pub fn code_builder(&self) -> assembly::CodeBuilder {
        assembly::CodeBuilder::with_source_manager(self.source_manager.clone())
    }

    /// Returns a reference to the client's random number generator. This can be used to generate
    /// randomness for various purposes such as serial numbers, keys, etc.
    pub fn rng(&mut self) -> &mut ClientRng {
        &mut self.rng
    }

    pub fn prover(&self) -> Arc<dyn TransactionProver + Send + Sync> {
        self.tx_prover.clone()
    }
}

impl<AUTH> Client<AUTH> {
    // LIMITS
    // --------------------------------------------------------------------------------------------

    /// Checks if the note tag limit has been exceeded.
    pub async fn check_note_tag_limit(&self) -> Result<(), ClientError> {
        let limits = self.rpc_api.get_rpc_limits().await;
        if self.store.get_unique_note_tags().await?.len() >= limits.note_tags_limit {
            return Err(ClientError::NoteTagsLimitExceeded(limits.note_tags_limit));
        }
        Ok(())
    }

    /// Checks if the account limit has been exceeded.
    pub async fn check_account_limit(&self) -> Result<(), ClientError> {
        let limits = self.rpc_api.get_rpc_limits().await;
        if self.store.get_account_ids().await?.len() >= limits.account_ids_limit {
            return Err(ClientError::AccountsLimitExceeded(limits.account_ids_limit));
        }
        Ok(())
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    #[cfg(any(test, feature = "testing"))]
    pub fn test_rpc_api(&mut self) -> &mut Arc<dyn NodeRpcClient> {
        &mut self.rpc_api
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn test_store(&mut self) -> &mut Arc<dyn Store> {
        &mut self.store
    }
}

// CLIENT RNG
// ================================================================================================

// NOTE: The idea of having `ClientRng` is to enforce `Send` and `Sync` over `FeltRng`.
// This allows `Client`` to be `Send` and `Sync`. There may be users that would want to use clients
// with !Send/!Sync RNGs. For this we have two options:
//
// - We can make client generic over R (adds verbosity but is more flexible and maybe even correct)
// - We can optionally (e.g., based on features/target) change `ClientRng` definition to not enforce
//   these bounds. (similar to TransactionAuthenticator)

/// Marker trait for RNGs that can be shared across threads and used by the client.
pub trait ClientFeltRng: FeltRng + Send + Sync {}
impl<T> ClientFeltRng for T where T: FeltRng + Send + Sync {}

/// Boxed RNG trait object used by the client.
pub type ClientRngBox = Box<dyn ClientFeltRng>;

/// A wrapper around a [`FeltRng`] that implements the [`RngCore`] trait.
/// This allows the user to pass their own generic RNG so that it's used by the client.
pub struct ClientRng(ClientRngBox);

impl ClientRng {
    pub fn new(rng: ClientRngBox) -> Self {
        Self(rng)
    }

    pub fn inner_mut(&mut self) -> &mut ClientRngBox {
        &mut self.0
    }
}

impl RngCore for ClientRng {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }
}

impl FeltRng for ClientRng {
    fn draw_element(&mut self) -> Felt {
        self.0.draw_element()
    }

    fn draw_word(&mut self) -> Word {
        self.0.draw_word()
    }
}

/// Indicates whether the client is operating in debug mode.
#[derive(Debug, Clone, Copy)]
pub enum DebugMode {
    Enabled,
    Disabled,
}

impl From<DebugMode> for bool {
    fn from(debug_mode: DebugMode) -> Self {
        match debug_mode {
            DebugMode::Enabled => true,
            DebugMode::Disabled => false,
        }
    }
}

impl From<bool> for DebugMode {
    fn from(debug_mode: bool) -> DebugMode {
        if debug_mode {
            DebugMode::Enabled
        } else {
            DebugMode::Disabled
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn client_is_send_sync() {
        assert_send_sync::<Client<()>>();
    }
}
