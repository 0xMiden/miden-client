//! Response verification helpers shared by the [`NodeRpcClient`](super::NodeRpcClient) trait's
//! default methods, which check that a node's reply matches what was requested and return
//! [`RpcError::InvalidResponse`] on a mismatch.

use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec::Vec;

use miden_protocol::Word;
use miden_protocol::block::BlockNumber;
use miden_protocol::note::{NoteId, NoteScript, NoteTag};

use super::RpcError;
use super::domain::nullifier::NullifierUpdate;

/// Returns [`RpcError::InvalidResponse`] if `requested` is `Some` and `returned` does not equal it.
pub(super) fn verify_block_num(
    requested: Option<BlockNumber>,
    returned: BlockNumber,
) -> Result<(), RpcError> {
    if let Some(requested) = requested
        && returned != requested
    {
        return Err(RpcError::InvalidResponse(format!(
            "node returned block {returned} but block {requested} was requested"
        )));
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if any returned note ID was not in `requested`.
pub(super) fn verify_note_ids(
    requested: &BTreeSet<NoteId>,
    returned: impl IntoIterator<Item = NoteId>,
) -> Result<(), RpcError> {
    for id in returned {
        if !requested.contains(&id) {
            let list = requested.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            return Err(RpcError::InvalidResponse(format!(
                "node returned note {id} but [{list}] were requested"
            )));
        }
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if any returned note tag was not in `requested`.
pub(super) fn verify_note_tags(
    requested: &BTreeSet<NoteTag>,
    returned: impl IntoIterator<Item = NoteTag>,
) -> Result<(), RpcError> {
    for tag in returned {
        if !requested.contains(&tag) {
            let list = requested.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            return Err(RpcError::InvalidResponse(format!(
                "node returned note with tag {tag} but [{list}] were requested"
            )));
        }
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if any update carries a nullifier whose prefix was not in
/// `requested_prefixes`.
pub(super) fn verify_nullifier_prefixes(
    requested_prefixes: &BTreeSet<u16>,
    batch: &[NullifierUpdate],
) -> Result<(), RpcError> {
    for update in batch {
        let prefix = update.nullifier.prefix();
        if !requested_prefixes.contains(&prefix) {
            let requested = requested_prefixes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(RpcError::InvalidResponse(format!(
                "node returned nullifier with prefix {prefix} but [{requested}] were requested"
            )));
        }
    }
    Ok(())
}

/// Returns [`RpcError::InvalidResponse`] if `script`'s root does not equal the `requested` root.
pub(super) fn verify_note_script_root(
    requested: Word,
    script: &NoteScript,
) -> Result<(), RpcError> {
    let fetched_root = script.root();
    if Word::from(fetched_root) != requested {
        return Err(RpcError::InvalidResponse(format!(
            "node returned note script with root {fetched_root} for requested root {requested}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use core::slice;
    use std::collections::BTreeSet;

    use miden_protocol::note::{NoteId, NoteTag, Nullifier};
    use miden_protocol::{Felt, Word};

    use super::{NullifierUpdate, verify_note_ids, verify_note_tags, verify_nullifier_prefixes};
    use crate::rpc::RpcError;

    fn nullifier_with_prefix(prefix: u16) -> Nullifier {
        Nullifier::from_raw(Word::new([
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::new_unchecked(u64::from(prefix) << 48),
        ]))
    }

    #[test]
    fn verify_nullifier_prefixes_rejects_unrequested() {
        let requested = NullifierUpdate {
            nullifier: nullifier_with_prefix(0x1234),
            block_num: 1u32.into(),
        };
        let unrequested = NullifierUpdate {
            nullifier: nullifier_with_prefix(0xabcd),
            block_num: 2u32.into(),
        };

        let requested_prefixes: BTreeSet<u16> = BTreeSet::from([0x1234]);

        verify_nullifier_prefixes(&requested_prefixes, slice::from_ref(&requested))
            .expect("requested prefix must be accepted");

        let err = verify_nullifier_prefixes(&requested_prefixes, &[requested, unrequested])
            .expect_err("unrequested prefix must be rejected");
        assert!(matches!(err, RpcError::InvalidResponse(_)));
    }

    #[test]
    fn verify_note_tags_rejects_unrequested() {
        let requested = NoteTag::new(1);
        let other = NoteTag::new(2);
        let requested_set = BTreeSet::from([requested]);

        verify_note_tags(&requested_set, [requested]).expect("requested tag must be accepted");

        let err = verify_note_tags(&requested_set, [other])
            .expect_err("unrequested tag must be rejected");
        assert!(matches!(err, RpcError::InvalidResponse(_)));
    }

    fn note_id(n: u32) -> NoteId {
        NoteId::from_raw(Word::from([n, 0, 0, 0]))
    }

    #[test]
    fn verify_note_ids_rejects_unrequested() {
        let requested = note_id(1);
        let other = note_id(2);
        let requested_set = BTreeSet::from([requested]);

        verify_note_ids(&requested_set, [requested]).expect("requested note id must be accepted");

        let err = verify_note_ids(&requested_set, [other])
            .expect_err("unrequested note id must be rejected");
        assert!(matches!(err, RpcError::InvalidResponse(_)));
    }
}
