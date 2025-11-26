use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use miden_client::Client;
use miden_client::account::AccountId;
use miden_client::asset::FungibleAsset;
use miden_client::utils::{base_units_to_tokens, tokens_to_base_units};
use serde::{Deserialize, Serialize};

use crate::errors::CliError;
use crate::utils::parse_account_id;

/// Stores the detail information of a faucet to be stored in the token symbol map file.
#[derive(Debug, Serialize, Deserialize)]
pub struct FaucetDetails {
    pub id: String,
    pub decimals: u8,
}
pub struct FaucetDetailsMap(BTreeMap<String, FaucetDetails>);

impl FaucetDetailsMap {
    /// Creates a new instance of the `FaucetDetailsMap` struct by loading the token symbol map file
    /// from the specified `token_symbol_map_filepath`. If the file doesn't exist, an empty map is
    /// created.
    pub fn new(token_symbol_map_filepath: PathBuf) -> Result<Self, CliError> {
        let token_symbol_map: BTreeMap<String, FaucetDetails> =
            match std::fs::read_to_string(token_symbol_map_filepath) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(token_symbol_map) => token_symbol_map,
                    Err(err) => {
                        return Err(CliError::Config(
                            Box::new(err),
                            "Failed to parse token_symbol_map file".to_string(),
                        ));
                    },
                },
                Err(err) => {
                    if err.kind() != std::io::ErrorKind::NotFound {
                        return Err(CliError::Config(
                            Box::new(err),
                            "Failed to read token_symbol_map file".to_string(),
                        ));
                    }
                    BTreeMap::new()
                },
            };

        let mut faucet_ids = BTreeSet::new();
        for faucet in token_symbol_map.values() {
            if !faucet_ids.insert(faucet.id.clone()) {
                return Err(CliError::Config(
                    format!(
                        "Faucet ID {} appears more than once in the token symbol map",
                        faucet.id.clone()
                    )
                    .into(),
                    "Failed to parse token_symbol_map file".to_string(),
                ));
            }
        }

        Ok(Self(token_symbol_map))
    }

    pub fn get_token_symbol(&self, faucet_id: &AccountId) -> Option<String> {
        self.0
            .iter()
            .find(|(_, faucet)| faucet.id == faucet_id.to_hex())
            .map(|(symbol, _)| symbol.clone())
    }

    pub fn get_token_symbol_or_default(&self, faucet_id: &AccountId) -> String {
        self.get_token_symbol(faucet_id).unwrap_or("Unknown".to_string())
    }

    /// Returns the faucet ID and decimals for a given token symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the token symbol is not found in the map.
    pub fn get_faucet_details(&self, symbol: &str) -> Result<(&str, u8), CliError> {
        let details = self.0.get(symbol).ok_or(CliError::Config(
            "Token symbol not found in the map file".to_string().into(),
            symbol.to_string(),
        ))?;
        Ok((details.id.as_str(), details.decimals))
    }

    /// Returns the faucet account ID for a given token symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The token symbol is not found in the map.
    /// - The faucet ID cannot be parsed.
    pub async fn get_faucet_id<AUTH>(
        &self,
        client: &Client<AUTH>,
        symbol: &str,
    ) -> Result<AccountId, CliError> {
        let (id, _) = self.get_faucet_details(symbol)?;
        parse_account_id(client, id).await
    }

    /// Parses a string representing a [`FungibleAsset`]. There are two accepted formats for the
    /// string:
    /// - `<AMOUNT>::<FAUCET_ID>` where `<AMOUNT>` is in the faucet base units and `<FAUCET_ID>` is
    ///   the faucet's account ID.
    /// - `<AMOUNT>::<FAUCET_ADDRESS>` where `<AMOUNT>` is in the faucet base units and
    ///   `<FAUCET_ADDRESS>` is the faucet address.
    /// - `<AMOUNT>::<TOKEN_SYMBOL>` where `<AMOUNT>` is a decimal number representing the quantity
    ///   of the token (specified to the precision allowed by the token's decimals), and
    ///   `<TOKEN_SYMBOL>` is a symbol tracked in the token symbol map file.
    ///
    /// Some examples of valid `arg` values are `100::mlcl1qru2e5yvx40ndgqqqzusrryr0ucyd0uj`,
    /// `100::0xabcdef0123456789` and `1.23::TST`.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - The provided `arg` doesn't match one of the expected formats.
    /// - A faucet ID was provided but the amount isn't in base units.
    /// - The amount has more than the allowed number of decimals.
    /// - The token symbol isn't present in the token symbol map file.
    pub async fn parse_fungible_asset<AUTH>(
        &self,
        client: &Client<AUTH>,
        arg: &str,
    ) -> Result<FungibleAsset, CliError> {
        let (amount, asset) = arg.split_once("::").ok_or(CliError::Parse(
            "separator `::` not found".into(),
            "Failed to parse amount and asset".to_string(),
        ))?;
        let (faucet_id, amount) = if let Ok(id) = parse_account_id(client, asset).await {
            let amount = amount
                .parse::<u64>()
                .map_err(|err| CliError::Parse(err.into(), "Failed to parse u64".to_string()))?;
            (id, amount)
        } else {
            let FaucetDetails { id, decimals: faucet_decimals } =
                self.0.get(asset).ok_or(CliError::Config(
                    "Token symbol not found in the map file".to_string().into(),
                    asset.to_string(),
                ))?;

            // Convert from decimal to integer.
            let amount = tokens_to_base_units(amount, *faucet_decimals).map_err(|err| {
                CliError::Parse(err.into(), "Failed to parse tokens to base units".to_string())
            })?;

            (parse_account_id(client, id).await?, amount)
        };

        FungibleAsset::new(faucet_id, amount).map_err(CliError::Asset)
    }

    /// Formats a [`FungibleAsset`] into a tuple containing the faucet and the amount. The returned
    /// values depend on whether the faucet is tracked by the token symbol map file or not:
    /// - If the faucet is tracked, the token symbol is returned along with the amount in the
    ///   token's decimals.
    /// - If the faucet isn't tracked, the faucet ID is returned along with the amount in base
    ///   units.
    pub fn format_fungible_asset(
        &self,
        asset: &FungibleAsset,
    ) -> Result<(String, String), CliError> {
        if let Some(token_symbol) = self.get_token_symbol(&asset.faucet_id()) {
            let decimals = self
                .0
                .get(&token_symbol)
                .ok_or(CliError::Config(
                    "Token symbol not found in the map file".to_string().into(),
                    token_symbol.clone(),
                ))?
                .decimals;
            let amount = base_units_to_tokens(asset.amount(), decimals);

            Ok((token_symbol, amount))
        } else {
            Ok((asset.faucet_id().to_hex(), asset.amount().to_string()))
        }
    }
}
