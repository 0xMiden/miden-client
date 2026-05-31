use alloc::string::String;

use miden_protocol::WordError;
use miden_protocol::errors::AccountIdError;
use thiserror::Error;

/// Errors from resolving, encoding, or decoding values against a `Package`'s debug sections.
#[derive(Debug, Error)]
pub enum PackageDebugInfoError {
    #[error("invalid account-id '{token}': {source}")]
    InvalidAccountId { token: String, source: AccountIdError },

    #[error("invalid word '{token}': {source}")]
    InvalidWord { token: String, source: WordError },

    #[error("invalid bool '{0}' (expected true/false/0/1)")]
    InvalidBool(String),

    #[error("invalid u64 '{0}'")]
    InvalidU64(String),

    #[error("invalid hex '{0}'")]
    InvalidHex(String),

    #[error("value '{0}' is out of range for a field element")]
    FeltOutOfRange(String),

    #[error("expected {expected} argument(s), got {got}")]
    WrongArgCount { expected: usize, got: usize },

    #[error("not enough arguments")]
    NotEnoughArgs,

    #[error("too many arguments")]
    TooManyArgs,

    #[error("missing type at index {0} in DebugTypesSection")]
    MissingType(u32),

    #[error("type with shape '{0}' cannot be encoded as a CLI argument")]
    UnsupportedType(&'static str),
}
