//! The `account` module provides types and client APIs for managing accounts within the Miden
//! network.
//!
//! Accounts are foundational entities of the Miden protocol. They store assets and define
//! rules for manipulating them. Once an account is registered with the client, its state will
//! be updated accordingly, and validated against the network state on every sync.
//!
//! # Example
//!
//! To add a new account to the client's store, you might use the [`Client::add_account`] method as
//! follows:
//!
//! ```rust
//! # use miden_client::{
//! #   account::{Account, AccountBuilder, AccountType, component::BasicWallet},
//! #   crypto::FeltRng
//! # };
//! # use miden_objects::account::AccountStorageMode;
//! # async fn add_new_account_example<AUTH>(
//! #     client: &mut miden_client::Client<AUTH>
//! # ) -> Result<(), miden_client::ClientError> {
//! #   let random_seed = Default::default();
//! let account = AccountBuilder::new(random_seed)
//!     .account_type(AccountType::RegularAccountImmutableCode)
//!     .storage_mode(AccountStorageMode::Private)
//!     .with_component(BasicWallet)
//!     .build()?;
//!
//! // Add the account to the client. The account already embeds its seed information.
//! client.add_account(&account, false).await?;
//! #   Ok(())
//! # }
//! ```
//!
//! For more details on accounts, refer to the [Account] documentation.

use alloc::vec::Vec;

use miden_lib::account::auth::AuthRpoFalcon512;
use miden_lib::account::wallets::BasicWallet;
use miden_objects::crypto::dsa::rpo_falcon512::PublicKey;
use miden_objects::note::NoteTag;
// RE-EXPORTS
// ================================================================================================
pub use miden_objects::{
    AccountIdError,
    AddressError,
    NetworkIdError,
    account::{
        Account,
        AccountBuilder,
        AccountCode,
        AccountDelta,
        AccountFile,
        AccountHeader,
        AccountId,
        AccountIdPrefix,
        AccountStorage,
        AccountStorageMode,
        AccountType,
        PartialAccount,
        PartialStorage,
        PartialStorageMap,
        StorageMap,
        StorageSlot,
        StorageSlotType,
    },
    address::{AccountIdAddress, Address, AddressInterface, AddressType, NetworkId},
};

use super::Client;
use crate::errors::ClientError;
use crate::rpc::domain::account::FetchedAccount;
use crate::store::{AccountRecord, AccountStatus};
use crate::sync::NoteTagRecord;

pub mod component {
    pub const MIDEN_PACKAGE_EXTENSION: &str = "masp";

    pub use miden_lib::account::auth::*;
    pub use miden_lib::account::components::{
        basic_fungible_faucet_library,
        basic_wallet_library,
        rpo_falcon_512_library,
    };
    pub use miden_lib::account::faucets::{BasicFungibleFaucet, FungibleFaucetExt};
    pub use miden_lib::account::wallets::BasicWallet;
    pub use miden_objects::account::{
        AccountComponent,
        AccountComponentMetadata,
        FeltRepresentation,
        InitStorageData,
        StorageEntry,
        StorageValueName,
        TemplateType,
        WordRepresentation,
    };
}

// CLIENT METHODS
// ================================================================================================

/// This section of the [Client] contains methods for:
///
/// - **Account creation:** Use the [`AccountBuilder`] to construct new accounts, specifying account
///   type, storage mode (public/private), and attaching necessary components (e.g., basic wallet or
///   fungible faucet). After creation, they can be added to the client.
///
/// - **Account tracking:** Accounts added via the client are persisted to the local store, where
///   their state (including nonce, balance, and metadata) is updated upon every synchronization
///   with the network.
///
/// - **Data retrieval:** The module also provides methods to fetch account-related data.
impl<AUTH> Client<AUTH> {
    // ACCOUNT CREATION
    // --------------------------------------------------------------------------------------------

    /// Adds the provided [Account] in the store so it can start being tracked by the client.
    ///
    /// If the account is already being tracked and `overwrite` is set to `true`, the account will
    /// be overwritten. Newly created accounts must embed their seed (`account.seed()` must return
    /// `Some(_)`).
    ///
    /// # Errors
    ///
    /// - If the account is new but it does not contain the seed.
    /// - If the account is already tracked and `overwrite` is set to `false`.
    /// - If `overwrite` is set to `true` and the `account_data` nonce is lower than the one already
    ///   being tracked.
    /// - If `overwrite` is set to `true` and the `account_data` commitment doesn't match the
    ///   network's account commitment.
    pub async fn add_account(
        &mut self,
        account: &Account,
        overwrite: bool,
    ) -> Result<(), ClientError> {
        if account.is_new() {
            if account.seed().is_none() {
                return Err(ClientError::AddNewAccountWithoutSeed);
            }
        } else {
            // Ignore the seed since it's not a new account

            // TODO: The alternative approach to this is to store the seed anyway, but
            // ignore it at the point of executing against this transaction, but that
            // approach seems a little bit more incorrect
            if account.seed().is_some() {
                tracing::warn!(
                    "Added an existing account and still provided a seed when it is not needed. It's possible that the account's file was incorrectly generated. The seed will be ignored."
                );
            }
        }

        let tracked_account = self.store.get_account(account.id()).await?;

        match tracked_account {
            None => {
                let default_address = Address::AccountId(AccountIdAddress::new(
                    account.id(),
                    AddressInterface::Unspecified,
                ));

                // If the account is not being tracked, insert it into the store regardless of the
                // `overwrite` flag
                let default_address_note_tag = default_address.to_note_tag();
                let note_tag_record =
                    NoteTagRecord::with_account_source(default_address_note_tag, account.id());
                self.store.add_note_tag(note_tag_record).await?;

                self.store
                    .insert_account(account, default_address)
                    .await
                    .map_err(ClientError::StoreError)
            },
            Some(tracked_account) => {
                if !overwrite {
                    // Only overwrite the account if the flag is set to `true`
                    return Err(ClientError::AccountAlreadyTracked(account.id()));
                }

                if tracked_account.account().nonce().as_int() > account.nonce().as_int() {
                    // If the new account is older than the one being tracked, return an error
                    return Err(ClientError::AccountNonceTooLow);
                }

                if tracked_account.is_locked() {
                    // If the tracked account is locked, check that the account commitment matches
                    // the one in the network
                    let network_account_commitment =
                        self.rpc_api.get_account_details(account.id()).await?.commitment();
                    if network_account_commitment != account.commitment() {
                        return Err(ClientError::AccountCommitmentMismatch(
                            network_account_commitment,
                        ));
                    }
                }

                self.store.update_account(account).await.map_err(ClientError::StoreError)
            },
        }
    }

    /// Imports an account from the network to the client's store. The account needs to be public
    /// and be tracked by the network, it will be fetched by its ID. If the account was already
    /// being tracked by the client, it's state will be overwritten.
    ///
    /// # Errors
    /// - If the account is not found on the network.
    /// - If the account is private.
    /// - There was an error sending the request to the network.
    pub async fn import_account_by_id(&mut self, account_id: AccountId) -> Result<(), ClientError> {
        let fetched_account = self.rpc_api.get_account_details(account_id).await?;

        let account = match fetched_account {
            FetchedAccount::Private(..) => {
                return Err(ClientError::AccountIsPrivate(account_id));
            },
            FetchedAccount::Public(account, ..) => *account,
        };

        self.add_account(&account, true).await
    }

    /// Adds an [`Address`] to the associated [`AccountId`], alongside its derived [`NoteTag`].
    ///
    /// # Errors
    /// - If the account is not found on the network.
    /// - If the address is already being tracked.
    pub async fn add_address(
        &mut self,
        address: Address,
        account_id: AccountId,
    ) -> Result<(), ClientError> {
        let network_id = self.rpc_api.get_network_id().await?;
        let address_bench32 = address.to_bech32(network_id);
        if self.store.get_addresses_by_account_id(account_id).await?.contains(&address) {
            return Err(ClientError::AddressAlreadyTracked(address_bench32));
        }

        let tracked_account = self.store.get_account(account_id).await?;
        match tracked_account {
            None => Err(ClientError::AccountDataNotFound(account_id)),
            Some(_tracked_account) => {
                // Check that the Address is not already tracked
                let derived_note_tag: NoteTag = address.to_note_tag();
                let note_tag_record =
                    NoteTagRecord::with_account_source(derived_note_tag, account_id);
                if self.store.get_note_tags().await?.contains(&note_tag_record) {
                    return Err(ClientError::NoteTagDerivedAddressAlreadyTracked(
                        address_bench32,
                        derived_note_tag,
                    ));
                }

                self.store.insert_address(address, account_id).await?;
                Ok(())
            },
        }
    }

    /// Removes an [`Address`] from the associated [`AccountId`], alongside its derived [`NoteTag`].
    ///
    /// # Errors
    /// - If the account is not found on the network.
    /// - If the address is not being tracked.
    pub async fn remove_address(
        &mut self,
        address: Address,
        account_id: AccountId,
    ) -> Result<(), ClientError> {
        self.store.remove_address(address, account_id).await?;
        Ok(())
    }

    // ACCOUNT DATA RETRIEVAL
    // --------------------------------------------------------------------------------------------

    /// Returns a list of [`AccountHeader`] of all accounts stored in the database along with their
    /// statuses.
    ///
    /// Said accounts' state is the state after the last performed sync.
    pub async fn get_account_headers(
        &self,
    ) -> Result<Vec<(AccountHeader, AccountStatus)>, ClientError> {
        self.store.get_account_headers().await.map_err(Into::into)
    }

    /// Retrieves a full [`AccountRecord`] object for the specified `account_id`. This result
    /// represents data for the latest state known to the client, alongside its status. Returns
    /// `None` if the account ID is not found.
    pub async fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Option<AccountRecord>, ClientError> {
        self.store.get_account(account_id).await.map_err(Into::into)
    }

    /// Retrieves an [`AccountHeader`] object for the specified [`AccountId`] along with its status.
    /// Returns `None` if the account ID is not found.
    ///
    /// Said account's state is the state according to the last sync performed.
    pub async fn get_account_header_by_id(
        &self,
        account_id: AccountId,
    ) -> Result<Option<(AccountHeader, AccountStatus)>, ClientError> {
        self.store.get_account_header(account_id).await.map_err(Into::into)
    }

    /// Attempts to retrieve an [`AccountRecord`] by its [`AccountId`].
    ///
    /// # Errors
    ///
    /// - If the account record is not found.
    /// - If the underlying store operation fails.
    pub async fn try_get_account(
        &self,
        account_id: AccountId,
    ) -> Result<AccountRecord, ClientError> {
        self.get_account(account_id)
            .await?
            .ok_or(ClientError::AccountDataNotFound(account_id))
    }

    /// Attempts to retrieve an [`AccountHeader`] by its [`AccountId`].
    ///
    /// # Errors
    ///
    /// - If the account header is not found.
    /// - If the underlying store operation fails.
    pub async fn try_get_account_header(
        &self,
        account_id: AccountId,
    ) -> Result<(AccountHeader, AccountStatus), ClientError> {
        self.get_account_header_by_id(account_id)
            .await?
            .ok_or(ClientError::AccountDataNotFound(account_id))
    }
}

// UTILITY FUNCTIONS
// ================================================================================================

/// Builds an regular account ID from the provided parameters. The ID may be used along
/// `Client::import_account_by_id` to import a public account from the network (provided that the
/// used seed is known).
///
/// This function will only work for accounts with the [`BasicWallet`] and [`AuthRpoFalcon512`]
/// components.
///
/// # Arguments
/// - `init_seed`: Initial seed used to create the account. This is the seed passed to
///   [`AccountBuilder::new`].
/// - `public_key`: Public key of the account used in the [`AuthRpoFalcon512`] component.
/// - `storage_mode`: Storage mode of the account.
/// - `is_mutable`: Whether the account is mutable or not.
///
/// # Errors
/// - If the account cannot be built.
pub fn build_wallet_id(
    init_seed: [u8; 32],
    public_key: PublicKey,
    storage_mode: AccountStorageMode,
    is_mutable: bool,
) -> Result<AccountId, ClientError> {
    let account_type = if is_mutable {
        AccountType::RegularAccountUpdatableCode
    } else {
        AccountType::RegularAccountImmutableCode
    };

    let account = AccountBuilder::new(init_seed)
        .account_type(account_type)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512::new(public_key.into()))
        .with_component(BasicWallet)
        .build()?;

    Ok(account.id())
}
