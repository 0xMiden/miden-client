// RPC LIMITS
// ================================================================================================

use crate::rpc::generated::rpc as proto;

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

impl RpcLimits {
    /// Creates a new `RpcLimits` from the proto `RpcLimits` response.
    ///
    /// This method extracts the relevant limits from the proto response and falls back to
    /// default values if limits are not present (e.g., when connecting to an older node).
    pub fn from_proto(proto_limits: &proto::RpcLimits) -> Self {
        let mut limits = Self::default();

        // Extract note_id limit from GetNotesById endpoint
        if let Some(endpoint) = proto_limits.endpoints.get("GetNotesById")
            && let Some(&limit) = endpoint.parameters.get("note_id")
        {
            limits.note_ids_limit = limit as usize;
        }

        // Extract nullifier limit from CheckNullifiers or SyncNullifiers endpoint
        // Both should have the same limit, so we check CheckNullifiers first
        if let Some(endpoint) = proto_limits.endpoints.get("CheckNullifiers")
            && let Some(&limit) = endpoint.parameters.get("nullifier")
        {
            limits.nullifiers_limit = limit as usize;
        } else if let Some(endpoint) = proto_limits.endpoints.get("SyncNullifiers")
            && let Some(&limit) = endpoint.parameters.get("nullifier")
        {
            limits.nullifiers_limit = limit as usize;
        }

        // Extract account_id limit from SyncState endpoint
        if let Some(endpoint) = proto_limits.endpoints.get("SyncState")
            && let Some(&limit) = endpoint.parameters.get("account_id")
        {
            limits.account_ids_limit = limit as usize;
        }

        // Extract note_tag limit from SyncState or SyncNotes endpoint
        // Both should have the same limit, so we check SyncState first
        if let Some(endpoint) = proto_limits.endpoints.get("SyncState")
            && let Some(&limit) = endpoint.parameters.get("note_tag")
        {
            limits.note_tags_limit = limit as usize;
        } else if let Some(endpoint) = proto_limits.endpoints.get("SyncNotes")
            && let Some(&limit) = endpoint.parameters.get("note_tag")
        {
            limits.note_tags_limit = limit as usize;
        }

        limits
    }
}
