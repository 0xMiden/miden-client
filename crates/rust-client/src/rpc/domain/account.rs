use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Display, Formatter};

use miden_protocol::Word;
use miden_protocol::account::{
    Account,
    AccountCode,
    AccountHeader,
    AccountId,
    AccountStorageHeader,
    StorageMapWitness,
    StorageSlotHeader,
    StorageSlotName,
    StorageSlotType,
};
use miden_protocol::asset::Asset;
use miden_protocol::block::BlockNumber;
use miden_protocol::block::account_tree::AccountWitness;
use miden_protocol::crypto::merkle::SparseMerklePath;
use miden_protocol::crypto::merkle::smt::SmtProof;
use miden_tx::utils::{Deserializable, Serializable, ToHex};
use thiserror::Error;

use crate::alloc::borrow::ToOwned;
use crate::alloc::string::ToString;
use crate::rpc::RpcError;
use crate::rpc::domain::MissingFieldHelper;
use crate::rpc::errors::RpcConversionError;
use crate::rpc::generated::rpc::account_request::account_detail_request;
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
    pub last_block_num: BlockNumber,
}

impl AccountUpdateSummary {
    /// Creates a new [`AccountUpdateSummary`].
    pub fn new(commitment: Word, last_block_num: BlockNumber) -> Self {
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

impl TryInto<AccountHeader> for proto::account::AccountHeader {
    type Error = crate::rpc::RpcError;

    fn try_into(self) -> Result<AccountHeader, Self::Error> {
        use miden_protocol::Felt;

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
        use crate::rpc::RpcError;
        use crate::rpc::domain::MissingFieldHelper;

        let mut header_slots: Vec<StorageSlotHeader> = Vec::with_capacity(self.slots.len());

        for slot in self.slots {
            let slot_value: Word = slot
                .commitment
                .ok_or(proto::account::account_storage_header::StorageSlot::missing_field(
                    stringify!(commitment),
                ))?
                .try_into()?;

            let slot_type = u8::try_from(slot.slot_type)
                .map_err(|e| RpcError::InvalidResponse(e.to_string()))
                .and_then(|v| {
                    StorageSlotType::try_from(v)
                        .map_err(|e| RpcError::InvalidResponse(e.to_string()))
                })?;
            let slot_name = StorageSlotName::new(slot.slot_name)
                .map_err(|err| RpcError::InvalidResponse(err.to_string()))?;

            header_slots.push(StorageSlotHeader::new(slot_name, slot_type, slot_value));
        }

        header_slots.sort_by_key(StorageSlotHeader::id);
        AccountStorageHeader::new(header_slots)
            .map_err(|err| RpcError::InvalidResponse(err.to_string()))
    }
}

// FROM PROTO ACCOUNT HEADERS
// ================================================================================================

#[cfg(feature = "tonic")]
impl proto::rpc::account_response::AccountDetails {
    /// Converts the RPC response into `AccountDetails`.
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
    ) -> Result<AccountDetails, crate::rpc::RpcError> {
        use crate::rpc::RpcError;
        use crate::rpc::domain::MissingFieldHelper;

        let proto::rpc::account_response::AccountDetails {
            header,
            storage_details,
            code,
            vault_details,
        } = self;
        let header: AccountHeader = header
            .ok_or(proto::rpc::account_response::AccountDetails::missing_field(stringify!(header)))?
            .try_into()?;

        let storage_details = storage_details
            .ok_or(proto::rpc::account_response::AccountDetails::missing_field(stringify!(
                storage_details
            )))?
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
            .ok_or(proto::rpc::AccountVaultDetails::missing_field(stringify!(vault_details)))?
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

/// Account storage details for `AccountResponse`
#[derive(Clone, Debug)]
pub struct AccountStorageDetails {
    /// Account storage header (storage slot info for up to 256 slots)
    pub header: AccountStorageHeader,
    /// Additional data for the requested storage maps
    pub map_details: Vec<AccountStorageMapDetails>,
}

impl AccountStorageDetails {
    /// Find the matching details for a map, given its storage slot name.
    //  This linear search should be good enough since there can be
    //  only up to 256 slots, so locality probably wins here.
    pub fn find_map_details(&self, target: &StorageSlotName) -> Option<&AccountStorageMapDetails> {
        self.map_details.iter().find(|map_detail| map_detail.slot_name == *target)
    }
}

impl TryFrom<proto::rpc::AccountStorageDetails> for AccountStorageDetails {
    type Error = RpcError;

    fn try_from(value: proto::rpc::AccountStorageDetails) -> Result<Self, Self::Error> {
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
    /// Storage slot name of the storage map.
    pub slot_name: StorageSlotName,
    /// A flag that is set to `true` if the number of to-be-returned entries in the
    /// storage map would exceed a threshold. This indicates to the user that `SyncStorageMaps`
    /// endpoint should be used to get all storage map data.
    pub too_many_entries: bool,
    /// Storage map entries - either all entries (for small/full maps) or entries with proofs
    /// (for partial maps).
    pub entries: StorageMapEntries,
}

impl TryFrom<proto::rpc::account_storage_details::AccountStorageMapDetails>
    for AccountStorageMapDetails
{
    type Error = RpcError;

    fn try_from(
        value: proto::rpc::account_storage_details::AccountStorageMapDetails,
    ) -> Result<Self, Self::Error> {
        use proto::rpc::account_storage_details::account_storage_map_details::Entries;

        let slot_name = StorageSlotName::new(value.slot_name)
            .map_err(|err| RpcError::ExpectedDataMissing(err.to_string()))?;
        let too_many_entries = value.too_many_entries;

        let entries = match value.entries {
            Some(Entries::AllEntries(all_entries)) => {
                let entries = all_entries
                    .entries
                    .into_iter()
                    .map(core::convert::TryInto::try_into)
                    .collect::<Result<Vec<StorageMapEntry>, RpcError>>()?;
                StorageMapEntries::AllEntries(entries)
            },
            Some(Entries::EntriesWithProofs(entries_with_proofs)) => {
                let witnesses = entries_with_proofs
                    .entries
                    .into_iter()
                    .map(|entry| {
                        let key: Word = entry
                            .key
                            .ok_or(RpcError::ExpectedDataMissing("key".into()))?
                            .try_into()?;
                        let proof: SmtProof = entry
                            .proof
                            .ok_or(RpcError::ExpectedDataMissing("proof".into()))?
                            .try_into()?;
                        StorageMapWitness::new(proof, [key])
                            .map_err(|e| RpcError::InvalidResponse(e.to_string()))
                    })
                    .collect::<Result<Vec<StorageMapWitness>, RpcError>>()?;
                StorageMapEntries::EntriesWithProofs(witnesses)
            },
            None => StorageMapEntries::AllEntries(Vec::new()),
        };

        Ok(Self { slot_name, too_many_entries, entries })
    }
}

// STORAGE MAP ENTRY
// ================================================================================================

/// A storage map entry containing a key-value pair.
#[derive(Clone, Debug)]
pub struct StorageMapEntry {
    pub key: Word,
    pub value: Word,
}

impl TryFrom<proto::rpc::account_storage_details::account_storage_map_details::all_map_entries::StorageMapEntry>
    for StorageMapEntry
{
    type Error = RpcError;

    fn try_from(value: proto::rpc::account_storage_details::account_storage_map_details::all_map_entries::StorageMapEntry) -> Result<Self, Self::Error> {
        let key = value.key.ok_or(RpcError::ExpectedDataMissing("key".into()))?.try_into()?;
        let value = value.value.ok_or(RpcError::ExpectedDataMissing("value".into()))?.try_into()?;
        Ok(Self { key, value })
    }
}

// STORAGE MAP ENTRIES
// ================================================================================================

/// Storage map entries, either all entries (for small/full maps) or entries with proofs
/// (for partial maps).
#[derive(Clone, Debug)]
pub enum StorageMapEntries {
    /// All entries in the storage map (no proofs needed as the full map is available).
    AllEntries(Vec<StorageMapEntry>),
    /// Specific entries with their SMT proofs (for partial maps).
    EntriesWithProofs(Vec<StorageMapWitness>),
}

// ACCOUNT VAULT DETAILS
// ================================================================================================

#[derive(Clone, Debug)]
pub struct AccountVaultDetails {
    /// A flag that is set to true if the account contains too many assets. This indicates
    /// to the user that `SyncAccountVault` endpoint should be used to retrieve the
    /// account's assets
    pub too_many_assets: bool,
    /// When `too_many_assets` == false, this will contain the list of assets in the
    /// account's vault
    pub assets: Vec<Asset>,
}

impl TryFrom<proto::rpc::AccountVaultDetails> for AccountVaultDetails {
    type Error = RpcError;

    fn try_from(value: proto::rpc::AccountVaultDetails) -> Result<Self, Self::Error> {
        let too_many_assets = value.too_many_assets;
        let assets = value
            .assets
            .into_iter()
            .map(|asset| {
                let native_digest: Word = asset
                    .asset
                    .ok_or(proto::rpc::AccountVaultDetails::missing_field(stringify!(assets)))?
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

#[cfg(feature = "tonic")]
impl TryFrom<proto::rpc::AccountResponse> for AccountProof {
    type Error = RpcError;
    fn try_from(account_proof: proto::rpc::AccountResponse) -> Result<Self, Self::Error> {
        let Some(witness) = account_proof.witness else {
            return Err(RpcError::ExpectedDataMissing(
                "GetAccountProof returned an account without witness".to_owned(),
            ));
        };

        let details: Option<AccountDetails> = {
            match account_proof.details {
                None => None,
                Some(details) => Some(details.into_domain(&BTreeMap::new())?),
            }
        };
        AccountProof::new(witness.try_into()?, details)
            .map_err(|err| RpcError::InvalidResponse(format!("{err}")))
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

pub type StorageMapKey = Word;

/// Describes storage slots indices to be requested, as well as a list of keys for each of those
/// slots.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccountStorageRequirements(BTreeMap<StorageSlotName, Vec<StorageMapKey>>);

impl AccountStorageRequirements {
    pub fn new<'a>(
        slots_and_keys: impl IntoIterator<
            Item = (StorageSlotName, impl IntoIterator<Item = &'a StorageMapKey>),
        >,
    ) -> Self {
        let map = slots_and_keys
            .into_iter()
            .map(|(slot_name, keys_iter)| {
                let keys_vec: Vec<StorageMapKey> = keys_iter.into_iter().copied().collect();
                (slot_name, keys_vec)
            })
            .collect();

        AccountStorageRequirements(map)
    }

    pub fn inner(&self) -> &BTreeMap<StorageSlotName, Vec<StorageMapKey>> {
        &self.0
    }
}

impl From<AccountStorageRequirements> for Vec<account_detail_request::StorageMapDetailRequest> {
    fn from(
        value: AccountStorageRequirements,
    ) -> Vec<account_detail_request::StorageMapDetailRequest> {
        use account_detail_request;
        use account_detail_request::storage_map_detail_request;
        let request_map = value.0;
        let mut requests = Vec::with_capacity(request_map.len());
        for (slot_name, _map_keys) in request_map {
            let slot_data = Some(storage_map_detail_request::SlotData::AllEntries(true));
            requests.push(account_detail_request::StorageMapDetailRequest {
                slot_name: slot_name.to_string(),
                slot_data,
            });
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
