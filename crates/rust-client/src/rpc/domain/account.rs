use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Display, Formatter};

use miden_objects::Word;
use miden_objects::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountStorageHeader,
    StorageSlotType,
};
use miden_objects::asset::Asset;
use miden_objects::block::{AccountWitness, BlockNumber};
use miden_objects::crypto::merkle::SparseMerklePath;
use miden_tx::utils::{Deserializable, Serializable, ToHex};
use thiserror::Error;

use crate::alloc::borrow::ToOwned;
use crate::alloc::string::ToString;
use crate::rpc::RpcError;
use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated::{self as proto};

// FETCHED ACCOUNT
// ================================================================================================

/// Describes the possible responses from the `GetAccountDetails` endpoint for an account.
pub enum FetchedAccount {
    /// Private accounts are stored off-chain. Only a commitment to the state of the account is
    /// shared with the network. The full account state is to be tracked locally.
    Private(AccountId, AccountUpdateSummary),
    /// Public accounts are recorded on-chain. As such, its state is shared with the network and
    /// can always be retrieved through the appropriate RPC method.
    Public(Box<Account>, AccountUpdateSummary),
}

impl FetchedAccount {
    /// Creates a [`FetchedAccount`] corresponding to a private account tracked only by its ID and
    /// update summary.
    pub fn new_private(account_id: AccountId, summary: AccountUpdateSummary) -> Self {
        Self::Private(account_id, summary)
    }

    /// Creates a [`FetchedAccount`] for a public account with its full [`Account`] state.
    pub fn new_public(account: Account, summary: AccountUpdateSummary) -> Self {
        Self::Public(Box::new(account), summary)
    }

    /// Returns the account ID.
    pub fn account_id(&self) -> AccountId {
        match self {
            Self::Private(account_id, _) => *account_id,
            Self::Public(account, _) => account.id(),
        }
    }

    // Returns the account update summary commitment
    pub fn commitment(&self) -> Word {
        match self {
            Self::Private(_, summary) | Self::Public(_, summary) => summary.commitment,
        }
    }

    // Returns the associated account if the account is public, otherwise none
    pub fn account(&self) -> Option<&Account> {
        match self {
            Self::Private(..) => None,
            Self::Public(account, _) => Some(account.as_ref()),
        }
    }
}

impl From<FetchedAccount> for Option<Account> {
    fn from(acc: FetchedAccount) -> Self {
        match acc {
            FetchedAccount::Private(..) => None,
            FetchedAccount::Public(account, _) => Some(*account),
        }
    }
}

// ACCOUNT UPDATE SUMMARY
// ================================================================================================

/// Contains public updated information about the account requested.
pub struct AccountUpdateSummary {
    /// Commitment of the account, that represents a commitment to its updated state.
    pub commitment: Word,
    /// Block number of last account update.
    pub last_block_num: u32,
}

impl AccountUpdateSummary {
    /// Creates a new [`AccountUpdateSummary`].
    pub fn new(commitment: Word, last_block_num: u32) -> Self {
        Self { commitment, last_block_num }
    }
}

// ACCOUNT ID
// ================================================================================================

impl Display for proto::account::AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{}", self.id.to_hex()))
    }
}

impl Debug for proto::account::AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

// INTO PROTO ACCOUNT ID
// ================================================================================================

impl From<AccountId> for proto::account::AccountId {
    fn from(account_id: AccountId) -> Self {
        Self { id: account_id.to_bytes() }
    }
}

// FROM PROTO ACCOUNT ID
// ================================================================================================

impl TryFrom<proto::account::AccountId> for AccountId {
    type Error = RpcConversionError;

    fn try_from(account_id: proto::account::AccountId) -> Result<Self, Self::Error> {
        AccountId::read_from_bytes(&account_id.id).map_err(|_| RpcConversionError::NotAValidFelt)
    }
}

// ACCOUNT HEADER
// ================================================================================================

pub(crate) struct SlotTypeProto(pub u32);

impl TryInto<StorageSlotType> for SlotTypeProto {
    type Error = crate::rpc::RpcError;

    fn try_into(self) -> Result<StorageSlotType, Self::Error> {
        match self.0 {
            0 => Ok(StorageSlotType::Map),
            1 => Ok(StorageSlotType::Value),
            _ => Err(RpcError::InvalidResponse("Invalid storage slot type".into())),
        }
    }
}

impl TryInto<AccountHeader> for proto::account::AccountHeader {
    type Error = crate::rpc::RpcError;

    fn try_into(self) -> Result<AccountHeader, Self::Error> {
        use miden_objects::Felt;

        use crate::rpc::domain::MissingFieldHelper;

        let proto::account::AccountHeader {
            account_id,
            nonce,
            vault_root,
            storage_commitment,
            code_commitment,
        } = self;

        let account_id: AccountId = account_id
            .ok_or(proto::account::AccountHeader::missing_field(stringify!(account_id)))?
            .try_into()?;
        let vault_root = vault_root
            .ok_or(proto::account::AccountHeader::missing_field(stringify!(vault_root)))?
            .try_into()?;
        let storage_commitment = storage_commitment
            .ok_or(proto::account::AccountHeader::missing_field(stringify!(storage_commitment)))?
            .try_into()?;
        let code_commitment = code_commitment
            .ok_or(proto::account::AccountHeader::missing_field(stringify!(code_commitment)))?
            .try_into()?;

        Ok(AccountHeader::new(
            account_id,
            Felt::new(nonce),
            vault_root,
            storage_commitment,
            code_commitment,
        ))
    }
}

// ACCOUNT STORAGE HEADER
// ================================================================================================

impl TryInto<AccountStorageHeader> for proto::account::AccountStorageHeader {
    type Error = crate::rpc::RpcError;

    fn try_into(self) -> Result<AccountStorageHeader, Self::Error> {
        use crate::rpc::domain::MissingFieldHelper;

        let mut header_slots: Vec<(StorageSlotType, Word)> = Vec::with_capacity(self.slots.len());

        for slot in self.slots {
            let commitment: Word = slot
                .commitment
                .ok_or(proto::account::account_storage_header::StorageSlot::missing_field(
                    stringify!(commitment),
                ))?
                .try_into()?;

            let slot_type: StorageSlotType = SlotTypeProto(slot.slot_type).try_into()?;

            header_slots.push((slot_type, commitment));
        }

        Ok(AccountStorageHeader::new(header_slots))
    }
}

// FROM PROTO ACCOUNT HEADERS
// ================================================================================================

#[cfg(feature = "tonic")]
impl proto::rpc_store::account_proof_response::AccountDetails {
    /// Converts the RPC response into `StateHeaders`.
    ///
    /// The RPC response may omit unchanged account codes. If so, this function uses
    /// `known_account_codes` to fill in the missing code. If a required code cannot be found in
    /// the response or `known_account_codes`, an error is returned.
    ///
    /// # Errors
    /// - If account code is missing both on `self` and `known_account_codes`
    /// - If data cannot be correctly deserialized
    pub fn into_domain(
        self,
        known_account_codes: &BTreeMap<Word, AccountCode>,
        storage_requirements: &AccountStorageRequirements,
    ) -> Result<AccountDetails, crate::rpc::RpcError> {
        use crate::rpc::RpcError;
        use crate::rpc::domain::MissingFieldHelper;

        let proto::rpc_store::account_proof_response::AccountDetails {
            header,
            storage_details,
            code,
            vault_details,
        } = self;
        let header: AccountHeader = header
            .ok_or(proto::rpc_store::account_proof_response::AccountDetails::missing_field(
                stringify!(header),
            ))?
            .try_into()?;

        let storage_details = storage_details
            .ok_or(proto::rpc_store::account_proof_response::AccountDetails::missing_field(
                stringify!(storage_details),
            ))?
            .try_into()?;

        // If an account code was received, it means the previously known account code is no longer
        // valid. If it was not, it means we sent a code commitment that matched and so our code
        // is still valid
        let code = {
            let received_code = code.map(|c| AccountCode::read_from_bytes(&c)).transpose()?;
            match received_code {
                Some(code) => code,
                None => known_account_codes
                    .get(&header.code_commitment())
                    .ok_or(RpcError::InvalidResponse(
                        "Account code was not provided, but the response did not contain it either"
                            .into(),
                    ))?
                    .clone(),
            }
        };

        let vault_details = vault_details
            .ok_or(proto::rpc_store::AccountVaultDetails::missing_field(stringify!(vault_details)))?
            .try_into()?;

        Ok(AccountDetails {
            header,
            storage_details,
            code,
            vault_details,
        })
    }
}

// ACCOUNT PROOF
// ================================================================================================

/// Contains a block number, and a list of account proofs at that block.
pub type AccountProofs = (BlockNumber, Vec<AccountProof>);

// ACCOUNT DETAILS
// ================================================================================================

/// An account details.
#[derive(Clone, Debug)]
pub struct AccountDetails {
    pub header: AccountHeader,
    pub storage_details: AccountStorageDetails,
    pub code: AccountCode,
    pub vault_details: AccountVaultDetails,
}

// ACCOUNT STORAGE DETAILS
// ================================================================================================

/// Account storage details for AccountProofResponse
#[derive(Clone, Debug)]
pub struct AccountStorageDetails {
    /// Account storage header (storage slot info for up to 256 slots)
    pub header: AccountStorageHeader,
    /// Additional data for the requested storage maps
    pub map_details: Vec<AccountStorageMapDetails>,
}

impl TryFrom<proto::rpc_store::AccountStorageDetails> for AccountStorageDetails {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::AccountStorageDetails) -> Result<Self, Self::Error> {
        let header = value
            .header
            .ok_or(proto::account::AccountStorageHeader::missing_field(stringify!(header)))?
            .try_into()?;
        let map_details = value
            .map_details
            .iter()
            .map(|entry| entry.to_owned().try_into())
            .collect::<Result<Vec<AccountStorageMapDetails>, RpcError>>()?;

        Ok(Self { header, map_details })
    }
}

// ACCOUNT MAP DETAILS
// ================================================================================================

#[derive(Clone, Debug)]
pub struct AccountStorageMapDetails {
    /// slot index of the storage map
    pub slot_index: u32,
    /// A flag that is set to `true` if the number of to-be-returned entries in the
    /// storage map would exceed a threshold. This indicates to the user that `SyncStorageMaps`
    /// endpoint should be used to get all storage map data.
    pub too_many_entries: bool,
    /// By default we provide all storage entries.
    pub entries: Vec<StorageMapEntry>,
}

impl TryFrom<proto::rpc_store::account_storage_details::AccountStorageMapDetails>
    for AccountStorageMapDetails
{
    type Error = RpcError;

    fn try_from(
        value: proto::rpc_store::account_storage_details::AccountStorageMapDetails,
    ) -> Result<Self, Self::Error> {
        let slot_index = value.slot_index;
        let too_many_entries = value.too_many_entries;
        let entries = value
            .entries
            .ok_or(
                proto::rpc_store::account_storage_details::AccountStorageMapDetails::missing_field(
                    stringify!(entries),
                ),
            )?
            .entries
            .iter_mut()
            .map(|entry| entry.to_owned().try_into())
            .collect::<Result<Vec<StorageMapEntry>, RpcError>>()?;

        Ok(Self { slot_index, too_many_entries, entries })
    }
}

// STORAGE MAP ENTRY
// ================================================================================================

#[derive(Clone, Debug)]
pub struct StorageMapEntry {
    pub key: Word,
    pub value: Word,
}

impl TryFrom<proto::rpc_store::account_storage_details::account_storage_map_details::map_entries::StorageMapEntry>
    for StorageMapEntry
{
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::account_storage_details::account_storage_map_details::map_entries::StorageMapEntry) -> Result<Self, Self::Error> {
        let key = value.key.ok_or(
            proto::rpc_store::account_storage_details::account_storage_map_details::map_entries::StorageMapEntry::missing_field(
                stringify!(key),
            ))?.try_into()?;

        let value = value.value.ok_or(
            proto::rpc_store::account_storage_details::account_storage_map_details::map_entries::StorageMapEntry::missing_field(
                stringify!(value),
            ))?.try_into()?;

        Ok(Self {
            key,
            value
        })
    }
}

// ACCOUNT VAULT DETAILS
// ================================================================================================

#[derive(Clone, Debug)]
pub struct AccountVaultDetails {
    /// A flag that is set to true if the account contains too many assets. This indicates
    /// to the user that `SyncAccountVault` endpoint should be used to retrieve the
    /// account's assets
    pub too_many_assets: bool,
    /// When too_many_assets == false, this will contain the list of assets in the
    /// account's vault
    pub assets: Vec<Asset>,
}

impl TryFrom<proto::rpc_store::AccountVaultDetails> for AccountVaultDetails {
    type Error = RpcError;

    fn try_from(value: proto::rpc_store::AccountVaultDetails) -> Result<Self, Self::Error> {
        let too_many_assets = value.too_many_assets;
        let assets = value
            .assets
            .into_iter()
            .map(|asset| {
                let native_digest: Word = asset
                    .asset
                    .ok_or(proto::rpc_store::AccountVaultDetails::missing_field(stringify!(
                        assets
                    )))?
                    .try_into()?;
                native_digest
                    .try_into()
                    .map_err(|_| RpcError::DeserializationError("asset".to_string()))
            })
            .collect::<Result<Vec<Asset>, RpcError>>()?;

        Ok(Self { too_many_assets, assets })
    }
}

// ACCOUNT PROOF
// ================================================================================================

/// Represents a proof of existence of an account's state at a specific block number.
#[derive(Clone, Debug)]
pub struct AccountProof {
    /// Account witness.
    account_witness: AccountWitness,
    /// State headers of public accounts.
    state_headers: Option<AccountDetails>,
}

impl AccountProof {
    /// Creates a new [`AccountProof`].
    pub fn new(
        account_witness: AccountWitness,
        account_details: Option<AccountDetails>,
    ) -> Result<Self, AccountProofError> {
        if let Some(AccountDetails {
            header: account_header,
            storage_details: _,
            code,
            ..
        }) = &account_details
        {
            if account_header.commitment() != account_witness.state_commitment() {
                return Err(AccountProofError::InconsistentAccountCommitment);
            }
            if account_header.id() != account_witness.id() {
                return Err(AccountProofError::InconsistentAccountId);
            }
            if code.commitment() != account_header.code_commitment() {
                return Err(AccountProofError::InconsistentCodeCommitment);
            }
        }

        Ok(Self {
            account_witness,
            state_headers: account_details,
        })
    }

    /// Returns the account ID related to the account proof.
    pub fn account_id(&self) -> AccountId {
        self.account_witness.id()
    }

    /// Returns the account header, if present.
    pub fn account_header(&self) -> Option<&AccountHeader> {
        self.state_headers.as_ref().map(|account_details| &account_details.header)
    }

    /// Returns the storage header, if present.
    pub fn storage_header(&self) -> Option<&AccountStorageHeader> {
        self.state_headers
            .as_ref()
            .map(|account_details| &account_details.storage_details.header)
    }

    /// Returns the account code, if present.
    pub fn account_code(&self) -> Option<&AccountCode> {
        self.state_headers.as_ref().map(|headers| &headers.code)
    }

    /// Returns the code commitment, if account code is present in the state headers.
    pub fn code_commitment(&self) -> Option<Word> {
        self.account_code().map(AccountCode::commitment)
    }

    /// Returns the current state commitment of the account.
    pub fn account_commitment(&self) -> Word {
        self.account_witness.state_commitment()
    }

    pub fn account_witness(&self) -> &AccountWitness {
        &self.account_witness
    }

    /// Returns the proof of the account's inclusion.
    pub fn merkle_proof(&self) -> &SparseMerklePath {
        self.account_witness.path()
    }

    /// Deconstructs `AccountProof` into its individual parts.
    pub fn into_parts(self) -> (AccountWitness, Option<AccountDetails>) {
        (self.account_witness, self.state_headers)
    }
}

// ACCOUNT WITNESS
// ================================================================================================

impl TryFrom<proto::account::AccountWitness> for AccountWitness {
    type Error = RpcError;

    fn try_from(account_witness: proto::account::AccountWitness) -> Result<Self, Self::Error> {
        let state_commitment = account_witness
            .commitment
            .ok_or(proto::account::AccountWitness::missing_field(stringify!(state_commitment)))?
            .try_into()?;
        let merkle_path = account_witness
            .path
            .ok_or(proto::account::AccountWitness::missing_field(stringify!(merkle_path)))?
            .try_into()?;
        let account_id = account_witness
            .witness_id
            .ok_or(proto::account::AccountWitness::missing_field(stringify!(witness_id)))?
            .try_into()?;

        let witness = AccountWitness::new(account_id, state_commitment, merkle_path).unwrap();
        Ok(witness)
    }
}

// ACCOUNT STORAGE REQUEST
// ================================================================================================

pub type StorageSlotIndex = u8;
pub type StorageMapKey = Word;

/// Describes storage slots indices to be requested, as well as a list of keys for each of those
/// slots.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccountStorageRequirements(BTreeMap<StorageSlotIndex, Vec<StorageMapKey>>);

impl AccountStorageRequirements {
    pub fn new<'a>(
        slots_and_keys: impl IntoIterator<
            Item = (StorageSlotIndex, impl IntoIterator<Item = &'a StorageMapKey>),
        >,
    ) -> Self {
        let map = slots_and_keys
            .into_iter()
            .map(|(slot_index, keys_iter)| {
                let keys_vec: Vec<StorageMapKey> = keys_iter.into_iter().copied().collect();
                (slot_index, keys_vec)
            })
            .collect();

        AccountStorageRequirements(map)
    }

    pub fn inner(&self) -> &BTreeMap<StorageSlotIndex, Vec<StorageMapKey>> {
        &self.0
    }
}

impl From<AccountStorageRequirements>
    for Vec<
        proto::rpc_store::account_proof_request::account_detail_request::StorageMapDetailRequest,
    >
{
    fn from(
        value: AccountStorageRequirements,
    ) -> Vec<proto::rpc_store::account_proof_request::account_detail_request::StorageMapDetailRequest>
    {
        let mut requests = Vec::with_capacity(value.0.len());
        for (slot_index, map_keys) in value.0 {
            let slot_data = match map_keys.len() <= 1000 {
                true => Some(proto::rpc_store::account_proof_request::account_detail_request::storage_map_detail_request::SlotData::AllEntries(true)),
                false => {
                    let map_keys = proto::rpc_store::account_proof_request::account_detail_request::storage_map_detail_request::MapKeys {
                        map_keys: map_keys
                            .into_iter()
                            .map(crate::rpc::generated::primitives::Digest::from)
                            .collect()
                    };
                    Some(proto::rpc_store::account_proof_request::account_detail_request::storage_map_detail_request::SlotData::MapKeys(map_keys))
                }
            };
            requests.push(
                proto::rpc_store::account_proof_request::account_detail_request::StorageMapDetailRequest {
                    slot_index: u32::from(slot_index),
                    slot_data,
                },

            );
        }
        requests
    }
}

impl Serializable for AccountStorageRequirements {
    fn write_into<W: miden_tx::utils::ByteWriter>(&self, target: &mut W) {
        target.write(&self.0);
    }
}

impl Deserializable for AccountStorageRequirements {
    fn read_from<R: miden_tx::utils::ByteReader>(
        source: &mut R,
    ) -> Result<Self, miden_tx::utils::DeserializationError> {
        Ok(AccountStorageRequirements(source.read()?))
    }
}

// ERRORS
// ================================================================================================

#[derive(Debug, Error)]
pub enum AccountProofError {
    #[error(
        "the received account commitment doesn't match the received account header's commitment"
    )]
    InconsistentAccountCommitment,
    #[error("the received account id doesn't match the received account header's id")]
    InconsistentAccountId,
    #[error(
        "the received code commitment doesn't match the received account header's code commitment"
    )]
    InconsistentCodeCommitment,
}
