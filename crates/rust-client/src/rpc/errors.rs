use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::error::Error;
use core::num::TryFromIntError;

use miden_objects::NoteError;
use miden_objects::account::AccountId;
use miden_objects::crypto::merkle::MerkleError;
use miden_objects::note::NoteId;
use miden_objects::utils::DeserializationError;
use thiserror::Error;

// RPC ERROR
// ================================================================================================

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("rpc api response contained an update for a private account: {0}")]
    AccountUpdateForPrivateAccountReceived(AccountId),
    #[error("failed to connect to the api server: {0}")]
    ConnectionError(#[source] Box<dyn Error + Send + Sync + 'static>),
    #[error("failed to deserialize rpc data: {0}")]
    DeserializationError(String),
    #[error("rpc api response missing an expected field: {0}")]
    ExpectedDataMissing(String),
    #[error("rpc api response is invalid: {0}")]
    InvalidResponse(String),
    #[error("note with id {0} was not found")]
    NoteNotFound(NoteId),
    #[error("rpc request failed for {0}: {1}")]
    RequestError(String, String),
    #[error("merkle proof is not contained")]
    MerkleError(#[from] MerkleError),
    #[error("slot index out of bounds")]
    SlotOutOfBounds(#[source] TryFromIntError),
}

impl From<DeserializationError> for RpcError {
    fn from(err: DeserializationError) -> Self {
        Self::DeserializationError(err.to_string())
    }
}

impl From<NoteError> for RpcError {
    fn from(err: NoteError) -> Self {
        Self::DeserializationError(err.to_string())
    }
}

impl From<RpcConversionError> for RpcError {
    fn from(err: RpcConversionError) -> Self {
        Self::DeserializationError(err.to_string())
    }
}

// RPC CONVERSION ERROR
// ================================================================================================

#[derive(Debug, Error)]
pub enum RpcConversionError {
    #[error("failed to deserialize: {0}")]
    DeserializationError(#[from] DeserializationError),
    #[error("value is not in the range 0..modulus")]
    NotAValidFelt,
    #[error("note error")]
    NoteTypeError(#[from] NoteError),
    #[error("merkle error")]
    MerkleError(#[from] MerkleError),
    #[error("failed to convert rpc data: {0}")]
    InvalidField(String),
    #[error("failed to convert int")]
    InvalidInt(#[from] TryFromIntError),
    #[error("field `{field_name}` expected to be present in protobuf representation of {entity}")]
    MissingFieldInProtobufRepresentation {
        entity: &'static str,
        field_name: &'static str,
    },
}
