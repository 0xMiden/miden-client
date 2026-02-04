// RPC LIMITS
// ================================================================================================

use core::convert::TryFrom;

use miden_tx::utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated::rpc as proto;

/// Key used to store RPC limits in the settings table.
pub const RPC_LIMITS_STORE_SETTING: &str = "rpc_limits";

const DEFAULT_NOTE_IDS_LIMIT: usize = 100;
const DEFAULT_NULLIFIERS_LIMIT: usize = 1000;
const DEFAULT_ACCOUNT_IDS_LIMIT: usize = 1000;
const DEFAULT_NOTE_TAGS_LIMIT: usize = 1000;

/// Domain type representing RPC endpoint limits.
///
/// These limits define the maximum number of items that can be sent in a single RPC request.
/// Exceeding these limits will result in the request being rejected by the node.
#[derive(Debug, Clone, Copy)]
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
        // Use u64 for portability (usize varies by platform)
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

impl TryFrom<proto::RpcLimits> for RpcLimits {
    type Error = RpcConversionError;

    fn try_from(proto_limits: proto::RpcLimits) -> Result<Self, Self::Error> {
        // Extract note_id limit from GetNotesById endpoint
        let endpoint = proto_limits.endpoints.get("GetNotesById").ok_or_else(|| {
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "RpcLimits",
                field_name: "note_id",
            }
        })?;
        let limit = endpoint.parameters.get("note_id").ok_or_else(|| {
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "RpcLimits",
                field_name: "note_id",
            }
        })?;
        let note_ids_limit = *limit as usize;

        // Extract nullifier limit from CheckNullifiers or SyncNullifiers endpoint
        // Both should have the same limit, so we check CheckNullifiers first
        let nullifiers_limit = if let Some(endpoint) = proto_limits.endpoints.get("CheckNullifiers")
        {
            endpoint.parameters.get("nullifier").ok_or_else(|| {
                RpcConversionError::MissingFieldInProtobufRepresentation {
                    entity: "RpcLimits",
                    field_name: "nullifier",
                }
            })?
        } else if let Some(endpoint) = proto_limits.endpoints.get("SyncNullifiers") {
            endpoint.parameters.get("nullifier").ok_or_else(|| {
                RpcConversionError::MissingFieldInProtobufRepresentation {
                    entity: "RpcLimits",
                    field_name: "nullifier",
                }
            })?
        } else {
            return Err(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "RpcLimits",
                field_name: "nullifier",
            });
        };
        let nullifiers_limit = *nullifiers_limit as usize;

        // Extract account_id limit from SyncState endpoint
        let endpoint = proto_limits.endpoints.get("SyncState").ok_or_else(|| {
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "RpcLimits",
                field_name: "SyncState",
            }
        })?;
        let limit = endpoint.parameters.get("account_id").ok_or_else(|| {
            RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "RpcLimits",
                field_name: "account_id",
            }
        })?;
        let account_ids_limit = *limit as usize;

        // Extract note_tag limit from SyncState or SyncNotes endpoint
        // Both should have the same limit, so we check SyncState first
        let note_tags_limit = if let Some(endpoint) = proto_limits.endpoints.get("SyncState") {
            endpoint.parameters.get("note_tag").ok_or_else(|| {
                RpcConversionError::MissingFieldInProtobufRepresentation {
                    entity: "RpcLimits",
                    field_name: "note_tag",
                }
            })?
        } else if let Some(endpoint) = proto_limits.endpoints.get("SyncNotes") {
            endpoint.parameters.get("note_tag").ok_or_else(|| {
                RpcConversionError::MissingFieldInProtobufRepresentation {
                    entity: "RpcLimits",
                    field_name: "note_tag",
                }
            })?
        } else {
            return Err(RpcConversionError::MissingFieldInProtobufRepresentation {
                entity: "RpcLimits",
                field_name: "note_tag",
            });
        };
        let note_tags_limit = *note_tags_limit as usize;

        Ok(Self {
            note_ids_limit,
            nullifiers_limit,
            account_ids_limit,
            note_tags_limit,
        })
    }
}
