// RPC LIMITS
// ================================================================================================

use core::convert::TryFrom;

use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated::rpc as proto;

/// Key used to store RPC limits in the settings table.
pub(crate) const RPC_LIMITS_STORE_SETTING: &str = "rpc_limits";

const DEFAULT_NOTE_IDS_LIMIT: usize = 100;
const DEFAULT_NULLIFIERS_LIMIT: usize = 1000;
const DEFAULT_ACCOUNT_IDS_LIMIT: usize = 1000;
const DEFAULT_NOTE_TAGS_LIMIT: usize = 1000;

/// Domain type representing RPC endpoint limits.
///
/// These limits define the maximum number of items that can be sent in a single RPC request.
/// Exceeding these limits will result in the request being rejected by the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpcLimits {
    /// Maximum number of note IDs that can be sent in a single `GetNotesById` request.
    pub note_ids_limit: usize,
    /// Maximum number of nullifier prefixes that can be sent in `CheckNullifiers` or
    /// `SyncNullifiers` requests.
    pub nullifiers_limit: usize,
    /// Maximum number of account IDs that can be sent in a single `SyncState` request.
    pub account_ids_limit: usize,
    /// Maximum number of note tags that can be sent in `SyncState` or `SyncNotes` requests.
    pub note_tags_limit: usize,
}

impl Default for RpcLimits {
    fn default() -> Self {
        Self {
            note_ids_limit: DEFAULT_NOTE_IDS_LIMIT,
            nullifiers_limit: DEFAULT_NULLIFIERS_LIMIT,
            account_ids_limit: DEFAULT_ACCOUNT_IDS_LIMIT,
            note_tags_limit: DEFAULT_NOTE_TAGS_LIMIT,
        }
    }
}

impl Serializable for RpcLimits {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        (self.note_ids_limit as u64).write_into(target);
        (self.nullifiers_limit as u64).write_into(target);
        (self.account_ids_limit as u64).write_into(target);
        (self.note_tags_limit as u64).write_into(target);
    }
}

impl Deserializable for RpcLimits {
    #[allow(clippy::cast_possible_truncation)]
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        Ok(Self {
            note_ids_limit: u64::read_from(source)? as usize,
            nullifiers_limit: u64::read_from(source)? as usize,
            account_ids_limit: u64::read_from(source)? as usize,
            note_tags_limit: u64::read_from(source)? as usize,
        })
    }
}

/// Extracts a parameter limit from the proto response for a given endpoint and parameter name.
fn get_param(
    proto: &proto::RpcLimits,
    endpoint: &str,
    param: &'static str,
) -> Result<usize, RpcConversionError> {
    let ep = proto.endpoints.get(endpoint).ok_or(
        RpcConversionError::MissingFieldInProtobufRepresentation {
            entity: "RpcLimits",
            field_name: param,
        },
    )?;
    let limit = ep.parameters.get(param).ok_or(
        RpcConversionError::MissingFieldInProtobufRepresentation {
            entity: "RpcLimits",
            field_name: param,
        },
    )?;
    Ok(*limit as usize)
}

impl TryFrom<proto::RpcLimits> for RpcLimits {
    type Error = RpcConversionError;

    fn try_from(proto: proto::RpcLimits) -> Result<Self, Self::Error> {
        Ok(Self {
            note_ids_limit: get_param(&proto, "GetNotesById", "note_id")?,
            nullifiers_limit: get_param(&proto, "CheckNullifiers", "nullifier")
                .or_else(|_| get_param(&proto, "SyncNullifiers", "nullifier"))?,
            account_ids_limit: get_param(&proto, "SyncState", "account_id")?,
            note_tags_limit: get_param(&proto, "SyncState", "note_tag")
                .or_else(|_| get_param(&proto, "SyncNotes", "note_tag"))?,
        })
    }
}
