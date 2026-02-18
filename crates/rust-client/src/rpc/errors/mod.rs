use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::error::Error;
use core::fmt;
use core::num::TryFromIntError;

use miden_protocol::account::AccountId;
use miden_protocol::crypto::merkle::MerkleError;
use miden_protocol::errors::NoteError;
use miden_protocol::note::NoteId;
use miden_protocol::utils::DeserializationError;
use thiserror::Error;

use super::RpcEndpoint;

pub mod node;
pub use node::EndpointError;

// RPC ERROR
// ================================================================================================

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("accept header validation failed: {0}")]
    AcceptHeaderError(#[from] AcceptHeaderError),
    #[error("rpc api response contained an update for a private account: {0}")]
    AccountUpdateForPrivateAccountReceived(AccountId),
    #[error("failed to connect to the api server: {0}")]
    ConnectionError(#[source] Box<dyn Error + Send + Sync + 'static>),
    #[error("failed to deserialize rpc data: {0}")]
    DeserializationError(String),
    #[error("rpc api response missing an expected field: {0}")]
    ExpectedDataMissing(String),
    #[error("rpc pagination error: {0}")]
    PaginationError(String),
    #[error("rpc api response is invalid: {0}")]
    InvalidResponse(String),
    #[error("grpc request failed for {endpoint}: {error_kind}{}",
        endpoint_error.as_ref().map_or(String::new(), |e| format!(" ({e})")))]
    RequestError {
        endpoint: RpcEndpoint,
        error_kind: GrpcError,
        endpoint_error: Option<EndpointError>,
        #[source]
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
    },
    #[error("note with id {0} was not found")]
    NoteNotFound(NoteId),
    #[error("invalid node endpoint: {0}")]
    InvalidNodeEndpoint(String),
}

impl RpcError {
    /// Returns the typed endpoint error if this is a request error, or `None` otherwise.
    pub fn endpoint_error(&self) -> Option<&EndpointError> {
        match self {
            Self::RequestError { endpoint_error, .. } => endpoint_error.as_ref(),
            _ => None,
        }
    }
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

// GRPC ERROR KIND
// ================================================================================================

/// Categorizes gRPC errors based on their status codes and common patterns
#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("resource not found")]
    NotFound,
    #[error("invalid request parameters")]
    InvalidArgument,
    #[error("permission denied")]
    PermissionDenied,
    #[error("resource already exists")]
    AlreadyExists,
    #[error("resource exhausted or rate limited")]
    ResourceExhausted,
    #[error("precondition failed")]
    FailedPrecondition,
    #[error("operation was cancelled")]
    Cancelled,
    #[error("deadline exceeded")]
    DeadlineExceeded,
    #[error("service unavailable")]
    Unavailable,
    #[error("internal server error")]
    Internal,
    #[error("unimplemented method")]
    Unimplemented,
    #[error("unauthenticated request")]
    Unauthenticated,
    #[error("operation was aborted")]
    Aborted,
    #[error("operation was attempted past the valid range")]
    OutOfRange,
    #[error("unrecoverable data loss or corruption")]
    DataLoss,
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl GrpcError {
    /// Creates a `GrpcError` from a gRPC status code following the official specification
    /// <https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc>
    pub fn from_code(code: i32, message: Option<String>) -> Self {
        match code {
            1 => Self::Cancelled,
            2 => Self::Unknown(message.unwrap_or_default()),
            3 => Self::InvalidArgument,
            4 => Self::DeadlineExceeded,
            5 => Self::NotFound,
            6 => Self::AlreadyExists,
            7 => Self::PermissionDenied,
            8 => Self::ResourceExhausted,
            9 => Self::FailedPrecondition,
            10 => Self::Aborted,
            11 => Self::OutOfRange,
            12 => Self::Unimplemented,
            13 => Self::Internal,
            14 => Self::Unavailable,
            15 => Self::DataLoss,
            16 => Self::Unauthenticated,
            _ => Self::Unknown(
                message.unwrap_or_else(|| format!("Unknown gRPC status code: {code}")),
            ),
        }
    }
}

// ACCEPT HEADER ERROR
// ================================================================================================

// TODO: Accept header errors are still parsed from message strings, which is fragile.
// Ideally the node would return structured error codes for these too. See #1129.

/// Errors that can occur during accept header validation.
#[derive(Debug, Error)]
pub enum AcceptHeaderError {
    #[error("server rejected request - please check your version and network settings ({0})")]
    NoSupportedMediaRange(AcceptHeaderContext),
    #[error("server rejected request - parsing error: {0}")]
    ParsingError(String),
}

/// Extra context attached to Accept header negotiation failures.
#[derive(Debug, Clone)]
pub struct AcceptHeaderContext {
    pub client_version: String,
    pub genesis_commitment: String,
}

impl AcceptHeaderContext {
    pub fn unknown() -> Self {
        Self {
            client_version: "unknown".to_string(),
            genesis_commitment: "unknown".to_string(),
        }
    }
}

impl fmt::Display for AcceptHeaderContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "client version: {}, genesis commitment: {}",
            self.client_version, self.genesis_commitment
        )
    }
}

impl AcceptHeaderError {
    /// Try to parse an accept header error from a message string, adding context.
    pub fn try_from_message_with_context(
        message: &str,
        context: AcceptHeaderContext,
    ) -> Option<Self> {
        // Check for the main compatibility error message
        if message.contains(
            "server does not support any of the specified application/vnd.miden content types",
        ) {
            return Some(Self::NoSupportedMediaRange(context));
        }
        if message.contains("genesis value failed to parse")
            || message.contains("version value failed to parse")
        {
            return Some(Self::ParsingError(message.to_string()));
        }
        None
    }
}
