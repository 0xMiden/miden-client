use alloc::string::ToString;
use alloc::vec::Vec;

use miden_protocol::account::{Account, AccountId};
use miden_protocol::note::{NoteDetailsCommitment, NoteId, NoteTag};
use miden_tx::utils::serde::{
    ByteReader,
    ByteWriter,
    Deserializable,
    DeserializationError,
    Serializable,
};
use tracing::warn;

use crate::Client;
use crate::errors::ClientError;
use crate::store::{InputNoteRecord, NoteRecordError};

/// Tag management methods
impl<AUTH> Client<AUTH> {
    /// Returns the list of note tags tracked by the client along with their source.
    ///
    /// When syncing the state with the node, these tags will be added to the sync request and
    /// note-related information will be retrieved for notes that have matching tags.
    ///  The source of the tag indicates its origin. It helps distinguish between:
    ///  - Tags added manually by the user.
    ///  - Tags automatically added by the client to track notes.
    ///  - Tags added for accounts tracked by the client.
    ///
    /// Note: Tags for accounts that are being tracked by the client are managed automatically by
    /// the client and don't need to be added here. That is, notes for managed accounts will be
    /// retrieved automatically by the client when syncing.
    pub async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, ClientError> {
        self.store.get_note_tags().await.map_err(Into::into)
    }

    /// Adds a note tag for the client to track. This tag's source will be marked as `User`.
    pub async fn add_note_tag(&mut self, tag: NoteTag) -> Result<(), ClientError> {
        let added = self
            .store
            .add_note_tag(NoteTagRecord { tag, source: NoteTagSource::User })
            .await?;
        if !added {
            warn!("Tag {} is already being tracked", tag);
        }
        Ok(())
    }

    /// Removes a note tag for the client to track. Only tags added by the user can be removed.
    pub async fn remove_note_tag(&mut self, tag: NoteTag) -> Result<(), ClientError> {
        if self
            .store
            .remove_note_tag(NoteTagRecord { tag, source: NoteTagSource::User })
            .await?
            == 0
        {
            warn!("Tag {} wasn't being tracked", tag);
        }

        Ok(())
    }
}

/// Represents a note tag of which the Store can keep track and retrieve.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NoteTagRecord {
    pub tag: NoteTag,
    pub source: NoteTagSource,
}

/// Represents the source of the tag. This is used to differentiate between tags that are added by
/// the user and tags that are added automatically by the client to track notes .
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NoteTagSource {
    /// Tag for notes directed to a tracked account.
    Account(AccountId),
    /// Tag for tracked expected notes, identified by the note's details commitment.
    Note(NoteDetailsCommitment),
    /// Tag manually added by the user.
    User,
    /// Subscription tag anchored to a [`NoteId`].
    Subscription(NoteId),
}

impl NoteTagRecord {
    pub fn with_note_source(tag: NoteTag, details_commitment: NoteDetailsCommitment) -> Self {
        Self {
            tag,
            source: NoteTagSource::Note(details_commitment),
        }
    }

    pub fn with_account_source(tag: NoteTag, account_id: AccountId) -> Self {
        Self {
            tag,
            source: NoteTagSource::Account(account_id),
        }
    }
}

impl Serializable for NoteTagRecord {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        self.tag.write_into(target);
        self.source.write_into(target);
    }
}

impl Deserializable for NoteTagRecord {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let tag = NoteTag::read_from(source)?;
        let source = NoteTagSource::read_from(source)?;
        Ok(Self { tag, source })
    }
}

impl Serializable for NoteTagSource {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        match self {
            NoteTagSource::Account(account_id) => {
                target.write_u8(0);
                account_id.write_into(target);
            },
            NoteTagSource::Note(details_commitment) => {
                target.write_u8(1);
                details_commitment.write_into(target);
            },
            NoteTagSource::User => target.write_u8(2),
            // Discriminant 3 must remain stable for pre-Subscription row compatibility.
            NoteTagSource::Subscription(key) => {
                target.write_u8(3);
                key.write_into(target);
            },
        }
    }
}

impl Deserializable for NoteTagSource {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        match source.read_u8()? {
            0 => Ok(NoteTagSource::Account(AccountId::read_from(source)?)),
            1 => Ok(NoteTagSource::Note(NoteDetailsCommitment::read_from(source)?)),
            2 => Ok(NoteTagSource::User),
            3 => Ok(NoteTagSource::Subscription(NoteId::read_from(source)?)),
            val => Err(DeserializationError::InvalidValue(format!("Invalid tag source: {val}"))),
        }
    }
}

impl PartialEq<NoteTag> for NoteTagRecord {
    fn eq(&self, other: &NoteTag) -> bool {
        self.tag == *other
    }
}

impl From<&Account> for NoteTagRecord {
    fn from(account: &Account) -> Self {
        NoteTagRecord::with_account_source(NoteTag::with_account_target(account.id()), account.id())
    }
}

impl TryInto<NoteTagRecord> for &InputNoteRecord {
    type Error = NoteRecordError;

    fn try_into(self) -> Result<NoteTagRecord, Self::Error> {
        match self.metadata() {
            Some(metadata) => {
                Ok(NoteTagRecord::with_note_source(metadata.tag(), self.details_commitment()))
            },
            None => Err(NoteRecordError::ConversionError(
                "Input Note Record does not contain tag".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tag_source_tests {
    use miden_protocol::Word;
    use miden_protocol::account::AccountId;
    use miden_protocol::note::{NoteDetailsCommitment, NoteId};
    use miden_protocol::testing::account_id::ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET;

    use super::{Deserializable, NoteTagSource, Serializable};

    /// Helper: builds a deterministic `NoteId` from a single u64.
    fn note_id_from_u64(value: u64) -> NoteId {
        let f = miden_protocol::Felt::new(value).unwrap();
        NoteId::from_raw(Word::from([f, f, f, f]))
    }

    /// `NoteTagSource` is serialised into the on-disk `tags.source` BLOB
    /// column. The wire encoding starts with a `u8` discriminant —
    /// stability of those values is part of the persisted-format
    /// contract. This test pins the discriminants explicitly so a
    /// renumber would fail it before any user upgrades a wallet to a
    /// version with shifted bytes.
    #[test]
    fn note_tag_source_discriminants_are_stable() {
        let cases = [
            (NoteTagSource::User, 2u8),
            (NoteTagSource::Subscription(note_id_from_u64(42)), 3u8),
        ];
        for (variant, expected_disc) in cases {
            let bytes = variant.to_bytes();
            assert_eq!(
                bytes[0], expected_disc,
                "variant {variant:?} expected discriminant {expected_disc}, got {}",
                bytes[0],
            );
        }
    }

    /// Round-trip every variant. Confirms the new `Subscription` discriminant
    /// (3) coexists with the existing `Account` / `Note` / `User` variants
    /// without disturbing their on-disk encoding.
    #[test]
    fn note_tag_source_round_trip_every_variant() {
        let account_id = AccountId::try_from(ACCOUNT_ID_PUBLIC_FUNGIBLE_FAUCET).unwrap();
        let details_commitment =
            NoteDetailsCommitment::from_raw_commitments(Word::empty(), Word::empty());
        let subscription_key = note_id_from_u64(0xdead_beef_dead_beef);

        let variants = [
            NoteTagSource::Account(account_id),
            NoteTagSource::Note(details_commitment),
            NoteTagSource::User,
            NoteTagSource::Subscription(subscription_key),
        ];

        for v in variants {
            let bytes = v.to_bytes();
            let decoded = NoteTagSource::read_from_bytes(&bytes).unwrap();
            assert_eq!(decoded, v, "round-trip failed for {v:?}");
        }
    }

    /// Deserialising an unknown discriminant must error rather than
    /// silently mapping to a known variant — defends against a future
    /// version writing a byte we don't understand.
    #[test]
    fn note_tag_source_unknown_discriminant_errors() {
        let bogus = [99u8];
        assert!(NoteTagSource::read_from_bytes(&bogus).is_err());
    }
}
