use miden_crypto::utils::Deserializable;
use miden_mast_package::Package;
use miden_protocol::account::AccountId;
use miden_protocol::asset::{Asset, FungibleAsset};
use miden_protocol::crypto::rand::FeltRng;
use miden_protocol::errors::NoteError;
use miden_protocol::note::{
    Note,
    NoteAssets,
    NoteAttachment,
    NoteAttachmentScheme,
    NoteInputs,
    NoteMetadata,
    NoteRecipient,
    NoteScript,
    NoteTag,
    NoteType,
};
use miden_protocol::utils::sync::LazyLock;
use miden_protocol::{Felt, Word, ZERO};
use miden_standards::note::utils;

const PSWAP_NOTE_SCRIPT_BYTES: &[u8] =
    include_bytes!("../../../contracts/swapp-note/swapp_note.masp");

// NOTE SCRIPT
// ================================================================================================

// Initialize the SWAPP note script only once by loading the embedded package
static PSWAP_SCRIPT: LazyLock<NoteScript> = LazyLock::new(|| {
    // Deserialize the package
    let package = Package::read_from_bytes(PSWAP_NOTE_SCRIPT_BYTES)
        .expect("Failed to deserialize swapp-note package");

    // Extract the note script from the package
    let note_program = package.unwrap_program();
    NoteScript::from_parts(note_program.mast_forest().clone(), note_program.entrypoint())
});

// PSWAP NOTE
// ================================================================================================

/// Partial swap (pswap) note for decentralized asset exchange.
///
/// This note implements a partially-fillable swap mechanism where:
/// - Creator offers an asset and requests another asset
/// - Note can be partially or fully filled by consumers
/// - Unfilled portions create remainder notes
/// - Creator receives requested assets via P2ID notes
pub struct PswapNote;

impl PswapNote {
    // CONSTANTS
    // --------------------------------------------------------------------------------------------

    /// Expected number of input items for the PSWAP note.
    ///
    /// Layout (8 Felts):
    /// - [0-3]: Requested asset (faucet_id_prefix, faucet_id_suffix, padding, amount)
    /// - [4-5]: Creator account ID (prefix, suffix)
    /// - [6]: Note type
    /// - [7]: P2ID routing tag
    pub const NUM_INPUT_ITEMS: usize = 8;

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the script of the PSWAP note.
    pub fn script() -> NoteScript {
        PSWAP_SCRIPT.clone()
    }

    /// Returns the PSWAP note script root.
    pub fn script_root() -> Word {
        PSWAP_SCRIPT.root()
    }

    // BUILDERS
    // --------------------------------------------------------------------------------------------

    /// Creates a PSWAP note offering one asset in exchange for another.
    ///
    /// # Arguments
    ///
    /// * `creator_account_id` - The account creating the swap offer
    /// * `offered_asset` - The asset being offered (will be locked in the note)
    /// * `requested_asset` - The asset being requested in exchange
    /// * `note_type` - Whether the note is public or private
    /// * `note_attachment` - Optional attachment data
    /// * `rng` - Random number generator for serial number
    ///
    /// # Returns
    ///
    /// Returns a `Note` that can be consumed by anyone willing to provide the requested asset.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Assets are invalid or have the same faucet ID
    /// - Note construction fails
    pub fn create<R: FeltRng>(
        creator_account_id: AccountId,
        offered_asset: Asset,
        requested_asset: Asset,
        note_type: NoteType,
        note_attachment: NoteAttachment,
        rng: &mut R,
    ) -> Result<Note, NoteError> {
        // Validate that offered and requested assets are different
        if offered_asset.faucet_id_prefix() == requested_asset.faucet_id_prefix() {
            return Err(NoteError::other("Offered and requested assets must be different"));
        }

        let note_script = Self::script();

        let (faucet_prefix, faucet_suffix, amount) = match &requested_asset {
            Asset::Fungible(fa) => (
                fa.faucet_id().prefix().as_felt(),
                fa.faucet_id().suffix(),
                Felt::new(fa.amount()),
            ),
            Asset::NonFungible(_nfa) => {
                return Err(NoteError::other("Non-fungible assets not yet supported"));
            },
        };

        // Build note inputs (8 Felts)
        let p2id_tag_felt = Self::compute_p2id_tag_felt(creator_account_id);

        let inputs = vec![
            faucet_prefix, // requested_asset.faucet_id().prefix()
            faucet_suffix, // requested_asset.faucet_id().suffix()
            Felt::new(0),  // padding
            amount,        // requested_asset.amount()
            creator_account_id.prefix().as_felt(),
            creator_account_id.suffix(),
            note_type.into(),
            p2id_tag_felt,
        ];

        let note_inputs = NoteInputs::new(inputs)?;

        // Build the tag for the PSWAP use case
        let tag = Self::build_tag(note_type, &offered_asset, &requested_asset);

        // Generate serial number
        let serial_num = rng.draw_word();

        // Build the outgoing note
        let metadata =
            NoteMetadata::new(creator_account_id, note_type, tag).with_attachment(note_attachment);

        let assets = NoteAssets::new(vec![offered_asset])?;
        let recipient = NoteRecipient::new(serial_num, note_script, note_inputs);
        let note = Note::new(assets, metadata, recipient);

        Ok(note)
    }

    /// Creates output notes when consuming a swap note (P2ID + optional remainder).
    ///
    /// This is the main function to call when consuming/filling a swap note. It handles both
    /// full and partial fills:
    /// - **Full fill**: Returns P2ID note with full requested amount, no remainder
    /// - **Partial fill**: Returns P2ID note with partial amount + remainder swap note
    ///
    /// # Arguments
    ///
    /// * `original_swap_note` - The original swap note being consumed
    /// * `consumer_account_id` - The account consuming the swap note (sender of P2ID)
    /// * `fill_amount` - The amount of requested asset being provided (e.g., ETH amount)
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(p2id_note, Option<remainder_note>)`:
    /// - `p2id_note`: Always created, contains the fill amount of requested asset
    /// - `remainder_note`: Only created for partial fills, contains remaining offered assets
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Alice created a swap: 100 USDC for 50 ETH
    /// // Bob provides 25 ETH (partial fill)
    /// let (p2id_note, remainder) = PswapNote::create_output_notes(
    ///     &swap_note,
    ///     bob_account_id,
    ///     25, // input_amount: 25 ETH
    ///     0,  // inflight_amount: 0
    /// )?;
    /// // p2id_note: 25 ETH sent to Alice
    /// // remainder: Some(new swap note: 50 USDC for 25 ETH)
    /// ```
    pub fn create_output_notes(
        original_swap_note: &Note,
        consumer_account_id: AccountId,
        input_amount: u64,
        inflight_amount: u64,
    ) -> Result<(Note, Option<Note>), NoteError> {
        // Parse original note to extract creator and swap details
        let inputs = original_swap_note.recipient().inputs();
        let (requested_asset_word, _creator_account_id, note_type, p2id_tag) =
            Self::parse_inputs(inputs.values())?;

        // Use input_amount as the fill amount for this call
        let fill_amount = input_amount + inflight_amount;

        // Reconstruct requested asset from note input components:
        // [0]=faucet_prefix, [1]=faucet_suffix, [2]=padding(0), [3]=amount
        let requested_faucet_id =
            AccountId::try_from([requested_asset_word[0], requested_asset_word[1]]).map_err(
                |e| NoteError::other(format!("Failed to parse requested faucet ID: {}", e)),
            )?;
        let total_requested_amount = requested_asset_word[3].as_int();

        // Ensure offered asset exists and is fungible
        let offered_assets = original_swap_note.assets();
        if offered_assets.num_assets() != 1 {
            return Err(NoteError::other("Swap note must have exactly 1 offered asset"));
        }
        let offered_asset =
            offered_assets.iter().next().ok_or(NoteError::other("No offered asset found"))?;
        let (offered_faucet_id, total_offered_amount) = match offered_asset {
            Asset::Fungible(fa) => (fa.faucet_id(), fa.amount()),
            _ => return Err(NoteError::other("Non-fungible offered asset not supported")),
        };

        // Validate fill amount
        if fill_amount == 0 {
            return Err(NoteError::other("Fill amount must be greater than 0"));
        }
        if fill_amount > total_requested_amount {
            return Err(NoteError::other(format!(
                "Fill amount {} exceeds requested amount {}",
                fill_amount, total_requested_amount
            )));
        }

        // Calculate proportional offered amount for this fill using helper
        let offered_amount_for_fill = Self::calculate_output_amount(
            total_offered_amount,
            total_requested_amount,
            fill_amount,
        );

        // Build the payback (P2ID) asset that will be sent to the creator
        let payback_asset = Asset::Fungible(
            FungibleAsset::new(requested_faucet_id, fill_amount)
                .map_err(|e| NoteError::other(format!("Failed to create P2ID asset: {}", e)))?,
        );

        // Build aux word: [fill_amount, 0, 0, 0] matching on-chain contract layout
        let aux_word = Word::from([Felt::new(fill_amount), ZERO, ZERO, ZERO]);

        // Create P2ID note using helper (pass aux word)
        let p2id_note = Self::create_p2id_payback_note(
            original_swap_note,
            consumer_account_id,
            payback_asset,
            note_type,
            p2id_tag,
            aux_word,
        )?;

        // Create remainder note if partial fill
        let remainder_note = if fill_amount < total_requested_amount {
            let remaining_offered = total_offered_amount - offered_amount_for_fill;
            let remaining_requested = total_requested_amount - fill_amount;

            let remaining_offered_asset =
                Asset::Fungible(FungibleAsset::new(offered_faucet_id, remaining_offered).map_err(
                    |e| NoteError::other(format!("Failed to create remainder asset: {}", e)),
                )?);

            Some(Self::create_remainder_note(
                original_swap_note,
                consumer_account_id,
                remaining_offered_asset,
                remaining_requested,
                offered_amount_for_fill,
            )?)
        } else {
            None
        };

        Ok((p2id_note, remainder_note))
    }

    /// Creates a P2ID (Pay-to-ID) note for the swap creator as payback.
    ///
    /// This is called when a swap note is consumed. The P2ID note contains the
    /// requested asset that the consumer is providing.
    ///
    /// # Arguments
    ///
    /// * `original_swap_note` - The original swap note being consumed
    /// * `consumer_account_id` - The account consuming the swap note (note sender)
    /// * `payback_asset` - The asset being sent to the creator (from requested asset pool)
    /// * `note_type` - The note type for the P2ID note (from swap note inputs)
    /// * `p2id_tag` - The P2ID routing tag (from swap note inputs)
    /// * `aux_word` - The aux Word to attach as auxiliary data (layout: [fill_amount, 0, 0, 0])
    ///
    /// # Returns
    ///
    /// Returns a P2ID `Note` that will be sent to the swap creator.
    pub fn create_p2id_payback_note(
        original_swap_note: &Note,
        consumer_account_id: AccountId,
        payback_asset: Asset,
        note_type: NoteType,
        p2id_tag: NoteTag,
        aux_word: Word,
    ) -> Result<Note, NoteError> {
        // Parse original note inputs to get creator (P2ID target)
        let inputs = original_swap_note.recipient().inputs();
        let (_, creator_account_id, ..) = Self::parse_inputs(inputs.values())?;

        // Generate serial number (typically: original serial_num + 1 for each element)
        let original_serial = original_swap_note.recipient().serial_num();
        let p2id_serial_num = Word::from([
            original_serial[0] + Felt::new(1),
            original_serial[1] + Felt::new(1),
            original_serial[2] + Felt::new(1),
            original_serial[3] + Felt::new(1),
        ]);

        // P2ID recipient is the creator (who receives the payback)
        let recipient = utils::build_p2id_recipient(creator_account_id, p2id_serial_num)?;

        // Attach aux value (amount) to the P2ID note
        let attachment = NoteAttachment::new_word(NoteAttachmentScheme::none(), aux_word);

        // Build P2ID note
        let p2id_assets = NoteAssets::new(vec![payback_asset])?;
        let p2id_metadata =
            NoteMetadata::new(consumer_account_id, note_type, p2id_tag).with_attachment(attachment);

        let p2id_note = Note::new(p2id_assets, p2id_metadata, recipient);

        Ok(p2id_note)
    }

    /// Creates a remainder note for partial fills.
    ///
    /// When a swap is partially filled, a remainder note is created containing:
    /// - The remaining offered assets
    /// - Updated note inputs reflecting the new amounts
    ///
    /// # Arguments
    ///
    /// * `original_swap_note` - The original swap note being consumed
    /// * `consumer_account_id` - The account consuming the swap note (note sender)
    /// * `remaining_offered_asset` - The remaining offered asset after partial fill
    /// * `remaining_requested_amount` - The remaining requested amount
    /// * `offered_amount_for_fill` - The proportional offered amount used for this fill (attached
    ///   as aux)
    ///
    /// # Returns
    ///
    /// Returns a new swap `Note` with the remaining amounts.
    pub fn create_remainder_note(
        original_swap_note: &Note,
        consumer_account_id: AccountId,
        remaining_offered_asset: Asset,
        remaining_requested_amount: u64,
        offered_amount_for_fill: u64,
    ) -> Result<Note, NoteError> {
        // Parse original note inputs
        let original_inputs = original_swap_note.recipient().inputs();
        let (requested_asset_word, creator_account_id, note_type, p2id_tag) =
            Self::parse_inputs(original_inputs.values())?;

        // Extract faucet prefix/suffix directly from note input components
        let faucet_prefix = requested_asset_word[0];
        let faucet_suffix = requested_asset_word[1];

        // Build new inputs with updated remaining amounts
        let p2id_tag_felt = Felt::new(u32::from(p2id_tag) as u64);

        let inputs = vec![
            faucet_prefix,
            faucet_suffix,
            ZERO,
            Felt::new(remaining_requested_amount), // Updated requested amount
            creator_account_id.prefix().as_felt(),
            creator_account_id.suffix(),
            note_type.into(),
            p2id_tag_felt,
        ];

        let note_inputs = NoteInputs::new(inputs)?;

        let original_serial: [Felt; 4] = original_swap_note.recipient().serial_num().into();

        // Build remainder note with same script
        let note_script = Self::script();
        let remainder_serial_num: [Felt; 4] =
            miden_core::crypto::hash::Rpo256::hash_elements(&original_serial).into();
        let remainder_serial_num = Word::from(remainder_serial_num);

        let recipient = NoteRecipient::new(remainder_serial_num, note_script, note_inputs);

        // Reconstruct requested faucet ID for tag building
        let requested_faucet_id =
            AccountId::try_from([faucet_prefix, faucet_suffix]).map_err(|e| {
                NoteError::other(format!("Failed to reconstruct requested faucet ID: {}", e))
            })?;
        let requested_asset_for_tag = Asset::Fungible(
            FungibleAsset::new(requested_faucet_id, remaining_requested_amount).map_err(|e| {
                NoteError::other(format!("Failed to create requested asset for tag: {}", e))
            })?,
        );

        // Build tag for the remainder note
        let tag = Self::build_tag(note_type, &remaining_offered_asset, &requested_asset_for_tag);

        // Attach offered_out as aux value: [offered_out, 0, 0, 0]
        let aux_word = Word::from([Felt::new(offered_amount_for_fill), ZERO, ZERO, ZERO]);
        let attachment = NoteAttachment::new_word(NoteAttachmentScheme::none(), aux_word);

        // Sender is the consumer (who executes the transaction)
        let metadata =
            NoteMetadata::new(consumer_account_id, note_type, tag).with_attachment(attachment);

        let assets = NoteAssets::new(vec![remaining_offered_asset])?;
        let remainder_note = Note::new(assets, metadata, recipient);

        Ok(remainder_note)
    }

    // TAG CONSTRUCTION
    // --------------------------------------------------------------------------------------------

    /// Returns a note tag for a pswap note with the specified parameters.
    ///
    /// The tag is laid out as follows:
    ///
    /// ```text
    /// [
    ///   note_type (2 bits) | script_root (14 bits)
    ///   | offered_asset_faucet_id (8 bits) | requested_asset_faucet_id (8 bits)
    /// ]
    /// ```
    ///
    /// The script root serves as the use case identifier of the PSWAP tag.
    pub fn build_tag(
        note_type: NoteType,
        offered_asset: &Asset,
        requested_asset: &Asset,
    ) -> NoteTag {
        let pswap_root_bytes = Self::script().root().as_bytes();

        // Construct the pswap use case ID from the 14 most significant bits of the script root
        // This leaves the two most significant bits zero
        let mut pswap_use_case_id = (pswap_root_bytes[0] as u16) << 6;
        pswap_use_case_id |= (pswap_root_bytes[1] >> 2) as u16;

        // Get bits 0..8 from the faucet IDs of both assets which will form the tag payload
        let offered_asset_id: u64 = offered_asset.faucet_id_prefix().into();
        let offered_asset_tag = (offered_asset_id >> 56) as u8;

        let requested_asset_id: u64 = requested_asset.faucet_id_prefix().into();
        let requested_asset_tag = (requested_asset_id >> 56) as u8;

        let asset_pair = ((offered_asset_tag as u16) << 8) | (requested_asset_tag as u16);

        let tag = ((note_type as u8 as u32) << 30)
            | ((pswap_use_case_id as u32) << 16)
            | asset_pair as u32;

        NoteTag::new(tag)
    }

    // HELPER FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Computes the P2ID tag for routing payback notes to the creator.
    fn compute_p2id_tag_felt(account_id: AccountId) -> Felt {
        let p2id_tag = NoteTag::with_account_target(account_id);
        Felt::new(u32::from(p2id_tag) as u64)
    }

    // PARSING FUNCTIONS
    // --------------------------------------------------------------------------------------------

    /// Parses note inputs to extract swap parameters.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The note inputs (must be exactly 8 Felts)
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - `requested_asset_word`: The requested asset as a Word
    /// - `creator_account_id`: The account ID of the swap creator
    /// - `note_type`: The note type for payback notes
    /// - `p2id_tag`: The tag for routing payback notes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input length is not 8
    /// - Account ID construction fails
    pub fn parse_inputs(
        inputs: &[Felt],
    ) -> Result<(Word, AccountId, NoteType, NoteTag), NoteError> {
        if inputs.len() != Self::NUM_INPUT_ITEMS {
            return Err(NoteError::other(format!(
                "PSWAP note should have {} inputs, but {} were provided",
                Self::NUM_INPUT_ITEMS,
                inputs.len()
            )));
        }

        // Extract requested asset word
        let requested_asset_word = Word::from([
            inputs[0], // faucet_id_prefix
            inputs[1], // faucet_id_suffix
            inputs[2], // padding (should be 0)
            inputs[3], // amount
        ]);

        // Extract creator account ID
        let creator_prefix = inputs[4];
        let creator_suffix = inputs[5];
        let creator_account_id = AccountId::try_from([creator_prefix, creator_suffix])
            .map_err(|e| NoteError::other(format!("Failed to parse creator account ID: {}", e)))?;

        // Extract note type and tag
        let note_type = NoteType::try_from(inputs[6].as_int() as u8)
            .map_err(|e| NoteError::other(format!("Failed to parse note type: {}", e)))?;

        let p2id_tag = NoteTag::new(inputs[7].as_int() as u32);

        Ok((requested_asset_word, creator_account_id, note_type, p2id_tag))
    }

    /// Extracts the requested asset from note inputs.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The note inputs
    ///
    /// # Returns
    ///
    /// Returns the requested `Asset`.
    pub fn get_requested_asset(inputs: &[Felt]) -> Result<Asset, NoteError> {
        let (requested_asset_word, ..) = Self::parse_inputs(inputs)?;
        // Reconstruct from components: [0]=prefix, [1]=suffix, [2]=0, [3]=amount
        let faucet_id = AccountId::try_from([requested_asset_word[0], requested_asset_word[1]])
            .map_err(|e| NoteError::other(format!("Failed to parse faucet ID: {}", e)))?;
        let amount = requested_asset_word[3].as_int();
        Ok(Asset::Fungible(
            FungibleAsset::new(faucet_id, amount)
                .map_err(|e| NoteError::other(format!("Failed to create asset: {}", e)))?,
        ))
    }

    /// Extracts the creator account ID from note inputs.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The note inputs
    ///
    /// # Returns
    ///
    /// Returns the creator's `AccountId`.
    pub fn get_creator_account_id(inputs: &[Felt]) -> Result<AccountId, NoteError> {
        let (_, creator_account_id, ..) = Self::parse_inputs(inputs)?;
        Ok(creator_account_id)
    }

    /// Checks if the given account is the creator of this swap note.
    ///
    /// # Arguments
    ///
    /// * `inputs` - The note inputs
    /// * `account_id` - The account ID to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the account is the creator, `false` otherwise.
    pub fn is_creator(inputs: &[Felt], account_id: AccountId) -> Result<bool, NoteError> {
        let creator_id = Self::get_creator_account_id(inputs)?;
        Ok(creator_id == account_id)
    }

    /// Calculates the output amount for a partial fill.
    ///
    /// This uses the same proportional calculation as the on-chain script.
    ///
    /// # Arguments
    ///
    /// * `offered_total` - Total offered asset amount in the note
    /// * `requested_total` - Total requested asset amount
    /// * `input_amount` - Amount of requested asset being provided
    ///
    /// # Returns
    ///
    /// Returns the proportional amount of offered asset to receive.
    pub fn calculate_output_amount(
        offered_total: u64,
        requested_total: u64,
        input_amount: u64,
    ) -> u64 {
        const PRECISION_FACTOR: u64 = 100_000;

        if offered_total > requested_total {
            // Case 1: offered_total > requested_total
            // Calculate ratio = (offered_total * factor) / requested_total
            // Then output = (input_amount * ratio) / factor
            let ratio = (offered_total * PRECISION_FACTOR) / requested_total;
            (input_amount * ratio) / PRECISION_FACTOR
        } else {
            // Case 2: offered_total <= requested_total
            // Direct calculation with precision
            let ratio = (requested_total * PRECISION_FACTOR) / offered_total;
            (input_amount * PRECISION_FACTOR) / ratio
        }
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;
    use std::{println, vec};

    use miden_crypto::FieldElement;
    use miden_protocol::account::{
        AccountBuilder,
        AccountId,
        AccountIdVersion,
        AccountStorageMode,
        AccountType,
    };
    use miden_protocol::asset::FungibleAsset;
    use miden_protocol::transaction::OutputNote;

    use super::*;

    #[test]
    fn test_pswap_note_creation_and_script() {
        // Test that the LazyLock PSWAP_SCRIPT initializes correctly and note creation works

        // Create test faucet IDs
        let mut offered_faucet_bytes = [0; 15];
        offered_faucet_bytes[0] = 0xaa;

        let mut requested_faucet_bytes = [0; 15];
        requested_faucet_bytes[0] = 0xbb;

        let offered_faucet_id = AccountId::dummy(
            offered_faucet_bytes,
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        let requested_faucet_id = AccountId::dummy(
            requested_faucet_bytes,
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        // Create creator account
        let creator_id = AccountId::dummy(
            [1; 15],
            AccountIdVersion::Version0,
            AccountType::RegularAccountImmutableCode,
            AccountStorageMode::Public,
        );

        // Create assets
        let offered_asset = Asset::Fungible(FungibleAsset::new(offered_faucet_id, 1000).unwrap());
        let requested_asset =
            Asset::Fungible(FungibleAsset::new(requested_faucet_id, 500).unwrap());

        // Create RNG
        use miden_crypto::rand::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(Word::default());

        // Test that the script can be accessed (this will trigger LazyLock initialization)
        let script = PswapNote::script();
        assert!(script.root() != Word::default(), "Script root should not be zero");
        println!("Script loaded successfully with root: {:?}", script.root());

        // Create a PSWAP note
        let note = PswapNote::create(
            creator_id,
            offered_asset,
            requested_asset,
            NoteType::Public,
            NoteAttachment::default(),
            &mut rng,
        );

        assert!(note.is_ok(), "Note creation should succeed");
        let note = note.unwrap();

        // Verify note properties
        assert_eq!(note.metadata().sender(), creator_id, "Note sender should match creator");
        assert_eq!(note.metadata().note_type(), NoteType::Public, "Note type should be Public");
        assert_eq!(note.assets().num_assets(), 1, "Note should have 1 asset");

        // Verify the note has the correct script
        assert_eq!(
            note.recipient().script().root(),
            script.root(),
            "Note script should match PSWAP script"
        );

        println!("PSWAP note created successfully with ID: {:?}", note.id());
    }

    #[test]
    fn test_pswap_tag() {
        // Construct test faucet IDs
        let mut offered_faucet_bytes = [0; 15];
        offered_faucet_bytes[0] = 0xcd;
        offered_faucet_bytes[1] = 0xb1;

        let mut requested_faucet_bytes = [0; 15];
        requested_faucet_bytes[0] = 0xab;
        requested_faucet_bytes[1] = 0xec;

        let offered_asset = Asset::Fungible(
            FungibleAsset::new(
                AccountId::dummy(
                    offered_faucet_bytes,
                    AccountIdVersion::Version0,
                    AccountType::FungibleFaucet,
                    AccountStorageMode::Public,
                ),
                2500,
            )
            .unwrap(),
        );

        let requested_asset = Asset::Fungible(
            FungibleAsset::new(
                AccountId::dummy(
                    requested_faucet_bytes,
                    AccountIdVersion::Version0,
                    AccountType::FungibleFaucet,
                    AccountStorageMode::Public,
                ),
                5000,
            )
            .unwrap(),
        );

        let expected_asset_pair = 0xcdab;

        let note_type = NoteType::Public;
        let actual_tag = PswapNote::build_tag(note_type, &offered_asset, &requested_asset);

        assert_eq!(actual_tag.as_u32() as u16, expected_asset_pair, "asset pair should match");
        assert_eq!((actual_tag.as_u32() >> 30) as u8, note_type as u8, "note type should match");
    }

    #[test]
    fn test_calculate_output_amount() {
        // Test 1: offered > requested (e.g., 1000 USDT offered for 500 ETH)
        let output = PswapNote::calculate_output_amount(1000, 500, 250);
        assert_eq!(output, 500); // Should get 500 USDT for 250 ETH

        // Test 2: offered < requested (e.g., 500 ETH offered for 1000 USDT)
        let output = PswapNote::calculate_output_amount(500, 1000, 500);
        assert_eq!(output, 250); // Should get 250 ETH for 500 USDT

        // Test 3: offered == requested (1:1 ratio)
        let output = PswapNote::calculate_output_amount(1000, 1000, 500);
        assert_eq!(output, 500); // Should get 500 for 500

        // Test 4: Partial fill
        let output = PswapNote::calculate_output_amount(10000, 5000, 1000);
        assert_eq!(output, 2000); // Should get 2000 for 1000
    }

    #[test]
    fn test_parse_inputs() {
        // Create test inputs
        let faucet_id = AccountId::dummy(
            [0; 15],
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        let creator_id = AccountId::dummy(
            [1; 15],
            AccountIdVersion::Version0,
            AccountType::RegularAccountImmutableCode,
            AccountStorageMode::Public,
        );

        let requested_asset = Asset::Fungible(FungibleAsset::new(faucet_id, 5000).unwrap());

        let requested_asset_word: Word = requested_asset.into();

        let note_type = NoteType::Public;
        let p2id_tag = NoteTag::with_account_target(creator_id);
        let p2id_tag_felt = Felt::new(u32::from(p2id_tag) as u64);

        let inputs = vec![
            requested_asset_word[0],
            requested_asset_word[1],
            ZERO,
            requested_asset_word[3],
            creator_id.prefix().as_felt(),
            creator_id.suffix(),
            note_type.into(),
            p2id_tag_felt,
        ];

        // Parse and verify
        let (parsed_asset_word, parsed_creator, parsed_note_type, parsed_tag) =
            PswapNote::parse_inputs(&inputs).unwrap();

        assert_eq!(parsed_asset_word, requested_asset_word);
        assert_eq!(parsed_creator, creator_id);
        assert_eq!(parsed_note_type, note_type);
        assert_eq!(parsed_tag, p2id_tag);
    }

    #[test]
    fn test_create_pswap_output_notes_full_fill() {
        // Test the simplified API: full fill scenario
        // Alice offers 100 USDC for 50 ETH
        // Bob provides 50 ETH (full fill)

        println!("=== Test: Full Fill using create_swap_output_notes ===");

        // Create Alice (creator)
        let alice_id = AccountId::dummy(
            [1; 15],
            AccountIdVersion::Version0,
            AccountType::RegularAccountImmutableCode,
            AccountStorageMode::Public,
        );

        // Create Bob (consumer)
        let bob_id = AccountId::dummy(
            [2; 15],
            AccountIdVersion::Version0,
            AccountType::RegularAccountImmutableCode,
            AccountStorageMode::Public,
        );

        // Create faucets
        let mut usdc_bytes = [0; 15];
        usdc_bytes[0] = 0xaa;
        let usdc_faucet = AccountId::dummy(
            usdc_bytes,
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        let mut eth_bytes = [0; 15];
        eth_bytes[0] = 0xbb;
        let eth_faucet = AccountId::dummy(
            eth_bytes,
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        // Create swap note: 100 USDC for 50 ETH
        let offered_asset = Asset::Fungible(FungibleAsset::new(usdc_faucet, 100).unwrap());
        let requested_asset = Asset::Fungible(FungibleAsset::new(eth_faucet, 50).unwrap());

        use miden_crypto::rand::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(Word::default());

        let swap_note = PswapNote::create(
            alice_id,
            offered_asset,
            requested_asset,
            NoteType::Public,
            NoteAttachment::default(),
            &mut rng,
        )
        .unwrap();

        println!("Created swap note: 100 USDC for 50 ETH");

        // NOW THE MAGIC: Bob consumes with just 2 parameters!
        let (p2id_note, remainder_note) =
            PswapNote::create_output_notes(&swap_note, bob_id, 50, 0).unwrap();

        println!("Created output notes with simple API");

        // Verify P2ID note (sender is Bob, recipient/target is Alice)
        assert_eq!(p2id_note.metadata().sender(), bob_id, "P2ID sender should be Bob");
        assert_eq!(p2id_note.metadata().note_type(), NoteType::Public, "P2ID should be Public");

        let p2id_assets = p2id_note.assets();
        assert_eq!(p2id_assets.num_assets(), 1);
        match p2id_assets.iter().next().unwrap() {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), eth_faucet, "P2ID should contain ETH");
                assert_eq!(fa.amount(), 50, "P2ID should contain 50 ETH");
            },
            _ => panic!("Expected fungible asset"),
        }

        // Verify no remainder for full fill
        assert!(remainder_note.is_none(), "No remainder for full fill");

        println!("Full fill test passed!");
        println!("  - P2ID note: 50 ETH to Alice");
        println!("  - No remainder (full fill)");
    }

    #[test]
    fn test_create_pswap_output_notes_partial_fill() {
        // Test the simplified API: partial fill scenario
        // Alice offers 100 USDC for 50 ETH
        // Bob provides 25 ETH (partial fill - 50%)

        println!("=== Test: Partial Fill using create_swap_output_notes ===");

        // Create Alice and Bob
        let alice_id = AccountId::dummy(
            [1; 15],
            AccountIdVersion::Version0,
            AccountType::RegularAccountImmutableCode,
            AccountStorageMode::Public,
        );

        let bob_id = AccountId::dummy(
            [2; 15],
            AccountIdVersion::Version0,
            AccountType::RegularAccountImmutableCode,
            AccountStorageMode::Public,
        );

        // Create faucets
        let mut usdc_bytes = [0; 15];
        usdc_bytes[0] = 0xaa;
        let usdc_faucet = AccountId::dummy(
            usdc_bytes,
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        let mut eth_bytes = [0; 15];
        eth_bytes[0] = 0xbb;
        let eth_faucet = AccountId::dummy(
            eth_bytes,
            AccountIdVersion::Version0,
            AccountType::FungibleFaucet,
            AccountStorageMode::Public,
        );

        // Create swap note: 100 USDC for 50 ETH
        let offered_asset = Asset::Fungible(FungibleAsset::new(usdc_faucet, 100).unwrap());
        let requested_asset = Asset::Fungible(FungibleAsset::new(eth_faucet, 50).unwrap());

        use miden_crypto::rand::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(Word::default());

        let swap_note = PswapNote::create(
            alice_id,
            offered_asset,
            requested_asset,
            NoteType::Public,
            NoteAttachment::default(),
            &mut rng,
        )
        .unwrap();

        println!("Created swap note: 100 USDC for 50 ETH");

        // Bob provides 25 ETH (50% fill) - SIMPLE API!
        let (p2id_note, remainder_note) =
            PswapNote::create_output_notes(&swap_note, bob_id, 25, 0).unwrap();

        println!("Created output notes for 50% fill");

        // Verify P2ID note (25 ETH to Alice)
        assert_eq!(p2id_note.metadata().sender(), bob_id);
        let p2id_assets = p2id_note.assets();
        match p2id_assets.iter().next().unwrap() {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), eth_faucet, "P2ID should contain ETH");
                assert_eq!(fa.amount(), 25, "P2ID should contain 25 ETH");
            },
            _ => panic!("Expected fungible asset"),
        }

        // Verify remainder note (50 USDC for 25 ETH)
        assert!(remainder_note.is_some(), "Remainder should exist for partial fill");
        let remainder = remainder_note.unwrap();

        assert_eq!(
            remainder.metadata().sender(),
            bob_id,
            "Remainder sender should be Bob (consumer)"
        );

        // Check remainder assets (50 USDC)
        let remainder_assets = remainder.assets();
        match remainder_assets.iter().next().unwrap() {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), usdc_faucet, "Remainder should contain USDC");
                assert_eq!(fa.amount(), 50, "Remainder should contain 50 USDC");
            },
            _ => panic!("Expected fungible asset"),
        }

        // Check remainder inputs (should request 25 ETH)
        let remainder_inputs = remainder.recipient().inputs();
        let remainder_requested =
            PswapNote::get_requested_asset(remainder_inputs.values()).unwrap();
        match remainder_requested {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), eth_faucet, "Remainder should request ETH");
                assert_eq!(fa.amount(), 25, "Remainder should request 25 ETH");
            },
            _ => panic!("Expected fungible asset"),
        }

        println!("Partial fill test passed!");
        println!("  - P2ID note: 25 ETH to Alice");
        println!("  - Remainder: 50 USDC for 25 ETH");
        println!("  - Proportional amounts calculated automatically!");
    }

    /// Helper to create test accounts and faucets used across validation tests.
    struct TestFixture {
        alice_id: AccountId,
        bob_id: AccountId,
        usdc_faucet: AccountId,
        eth_faucet: AccountId,
    }

    impl TestFixture {
        fn new() -> Self {
            let alice_id = AccountId::dummy(
                [1; 15],
                AccountIdVersion::Version0,
                AccountType::RegularAccountImmutableCode,
                AccountStorageMode::Public,
            );
            let bob_id = AccountId::dummy(
                [2; 15],
                AccountIdVersion::Version0,
                AccountType::RegularAccountImmutableCode,
                AccountStorageMode::Public,
            );
            let mut usdc_bytes = [0; 15];
            usdc_bytes[0] = 0xaa;
            let usdc_faucet = AccountId::dummy(
                usdc_bytes,
                AccountIdVersion::Version0,
                AccountType::FungibleFaucet,
                AccountStorageMode::Public,
            );
            let mut eth_bytes = [0; 15];
            eth_bytes[0] = 0xbb;
            let eth_faucet = AccountId::dummy(
                eth_bytes,
                AccountIdVersion::Version0,
                AccountType::FungibleFaucet,
                AccountStorageMode::Public,
            );
            Self {
                alice_id,
                bob_id,
                usdc_faucet,
                eth_faucet,
            }
        }

        /// Creates a swap note: Alice offers `offered_amt` USDC for `requested_amt` ETH.
        fn create_swap_note(&self, offered_amt: u64, requested_amt: u64) -> Note {
            use miden_crypto::rand::RpoRandomCoin;
            let mut rng = RpoRandomCoin::new(Word::default());

            let offered =
                Asset::Fungible(FungibleAsset::new(self.usdc_faucet, offered_amt).unwrap());
            let requested =
                Asset::Fungible(FungibleAsset::new(self.eth_faucet, requested_amt).unwrap());

            PswapNote::create(
                self.alice_id,
                offered,
                requested,
                NoteType::Public,
                NoteAttachment::default(),
                &mut rng,
            )
            .unwrap()
        }
    }

    #[test]
    fn test_p2id_recipient_targets_creator_not_consumer() {
        // Validates that build_p2id_recipient uses creator (Alice), not consumer (Bob).
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let (p2id_note, _) = PswapNote::create_output_notes(&swap_note, f.bob_id, 25, 0).unwrap();

        // P2ID metadata sender is the consumer (Bob) — he created the output note.
        assert_eq!(
            p2id_note.metadata().sender(),
            f.bob_id,
            "P2ID metadata sender should be Bob (consumer)"
        );

        // The P2ID recipient digest must match one built for Alice (creator).
        let original_serial = swap_note.recipient().serial_num();
        let p2id_serial = Word::from([
            original_serial[0] + Felt::new(1),
            original_serial[1] + Felt::new(1),
            original_serial[2] + Felt::new(1),
            original_serial[3] + Felt::new(1),
        ]);
        let expected_recipient = utils::build_p2id_recipient(f.alice_id, p2id_serial).unwrap();

        assert_eq!(
            p2id_note.recipient().digest(),
            expected_recipient.digest(),
            "P2ID recipient digest must match build_p2id_recipient(creator, ...)"
        );
        println!("P2ID recipient correctly targets creator (Alice)");
    }

    #[test]
    fn test_aux_word_layout_matches_contract() {
        // Validates that aux word layout is [fill_amount, 0, 0, 0].
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        // Full fill: input_amount=25, inflight=0 -> fill_amount=25
        let (p2id_note, _) = PswapNote::create_output_notes(&swap_note, f.bob_id, 25, 0).unwrap();

        let expected_aux = Word::from([Felt::new(25), ZERO, ZERO, ZERO]);
        let attachment = p2id_note.metadata().attachment();
        assert_eq!(
            attachment.content(),
            NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_aux).content(),
            "P2ID aux word should be [fill_amount, 0, 0, 0]"
        );

        // Inflight fill: input_amount=0, inflight=25 -> fill_amount=25
        let (p2id_note_inflight, _) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 0, 25).unwrap();

        let attachment_inflight = p2id_note_inflight.metadata().attachment();
        assert_eq!(
            attachment_inflight.content(),
            NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_aux).content(),
            "Inflight aux word should also be [fill_amount, 0, 0, 0]"
        );

        // Mixed: input_amount=10, inflight=5 -> fill_amount=15
        let swap_note_big = f.create_swap_note(100, 50);
        let (p2id_mixed, _) =
            PswapNote::create_output_notes(&swap_note_big, f.bob_id, 10, 5).unwrap();

        let expected_mixed_aux = Word::from([Felt::new(15), ZERO, ZERO, ZERO]);
        let attachment_mixed = p2id_mixed.metadata().attachment();
        assert_eq!(
            attachment_mixed.content(),
            NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_mixed_aux).content(),
            "Mixed aux word should be [15, 0, 0, 0]"
        );

        println!("Aux word layout [fill_amount, 0, 0, 0] verified for all cases");
    }

    #[test]
    fn test_remainder_note_sender_is_consumer() {
        // Validates that remainder note sender is consumer (Bob), not creator (Alice).
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        // Partial fill: 15 out of 25 ETH
        let (_, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 15, 0).unwrap();

        let remainder = remainder_note.expect("Partial fill should produce remainder");

        assert_eq!(
            remainder.metadata().sender(),
            f.bob_id,
            "Remainder sender should be Bob (consumer), not Alice (creator)"
        );
        println!("Remainder note sender is consumer (Bob)");
    }

    #[test]
    fn test_remainder_note_attachment_has_offered_out() {
        // Validates that remainder attachment is [offered_out, 0, 0, 0],
        // not a copy of the original swap note's attachment.
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        // Partial fill: 15 out of 25 -> offered_out = (50 * 15) / 25 = 30
        let (_, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 15, 0).unwrap();

        let remainder = remainder_note.expect("Partial fill should produce remainder");

        let expected_offered_out = PswapNote::calculate_output_amount(50, 25, 15);
        assert_eq!(expected_offered_out, 30, "Sanity check: offered_out should be 30");

        let expected_aux = Word::from([Felt::new(30), ZERO, ZERO, ZERO]);
        let attachment = remainder.metadata().attachment();
        assert_eq!(
            attachment.content(),
            NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_aux).content(),
            "Remainder attachment should be [offered_out, 0, 0, 0]"
        );

        // Also verify it's NOT the same as the original note's attachment
        assert_ne!(
            attachment.content(),
            NoteAttachment::default().content(),
            "Remainder attachment should NOT be a copy of original default attachment"
        );

        println!("Remainder note attachment correctly contains offered_out");
    }

    #[test]
    fn test_remainder_note_preserves_creator_in_inputs() {
        // Ensure the remainder note's inputs still reference Alice as creator,
        // even though the metadata sender is Bob.
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let (_, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 15, 0).unwrap();

        let remainder = remainder_note.unwrap();
        let (_, creator_in_remainder, note_type, _) =
            PswapNote::parse_inputs(remainder.recipient().inputs().values()).unwrap();

        assert_eq!(
            creator_in_remainder, f.alice_id,
            "Remainder inputs should preserve Alice as the creator"
        );
        assert_eq!(note_type, NoteType::Public, "Remainder should preserve original note type");

        println!("Remainder note inputs preserve creator (Alice) and note type");
    }

    #[test]
    fn test_remainder_note_amounts_match_integration_test() {
        // End-to-end partial fill matching the integration test scenario:
        // Alice offers 50 USDC for 25 ETH, Bob provides 15 ETH (60% fill).
        // Expected: P2ID = 15 ETH, remainder = 20 USDC for 10 ETH.
        // (swapp_test.rs: partial fill test lines 706-893)
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let (p2id_note, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 15, 0).unwrap();

        // Verify P2ID: 15 ETH
        let p2id_asset = p2id_note.assets().iter().next().unwrap();
        match p2id_asset {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), f.eth_faucet);
                assert_eq!(fa.amount(), 15, "P2ID should contain 15 ETH");
            },
            _ => panic!("Expected fungible asset"),
        }

        // Verify remainder: 20 USDC offered, requesting 10 ETH
        let remainder = remainder_note.expect("Should have remainder for partial fill");

        // Check remainder offered asset
        match remainder.assets().iter().next().unwrap() {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), f.usdc_faucet);
                assert_eq!(fa.amount(), 20, "Remainder should contain 20 USDC");
            },
            _ => panic!("Expected fungible asset"),
        }

        // Check remainder requested amount from inputs
        let remainder_requested =
            PswapNote::get_requested_asset(remainder.recipient().inputs().values()).unwrap();
        match remainder_requested {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), f.eth_faucet);
                assert_eq!(fa.amount(), 10, "Remainder should request 10 ETH");
            },
            _ => panic!("Expected fungible asset"),
        }

        println!("Partial fill amounts match integration test expectations");
        println!("  - P2ID: 15 ETH to Alice");
        println!("  - Remainder: 20 USDC for 10 ETH");
    }

    #[test]
    fn test_remainder_serial_num_is_hash_of_original() {
        // Validates that the remainder serial number is derived by hashing
        // the original serial number.
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let (_, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 15, 0).unwrap();

        let remainder = remainder_note.unwrap();

        let original_serial: [Felt; 4] = swap_note.recipient().serial_num().into();
        let expected_serial: [Felt; 4] =
            miden_core::crypto::hash::Rpo256::hash_elements(&original_serial).into();
        let expected_serial = Word::from(expected_serial);

        assert_eq!(
            remainder.recipient().serial_num(),
            expected_serial,
            "Remainder serial num should be RPO hash of original serial"
        );

        println!("Remainder serial number derived correctly via RPO hash");
    }

    #[test]
    fn test_full_fill_no_remainder() {
        // Full fill should produce no remainder note.
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let (p2id_note, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 25, 0).unwrap();

        assert!(remainder_note.is_none(), "Full fill should produce no remainder");

        // P2ID should have full requested amount
        match p2id_note.assets().iter().next().unwrap() {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), f.eth_faucet);
                assert_eq!(fa.amount(), 25, "P2ID should contain full 25 ETH");
            },
            _ => panic!("Expected fungible asset"),
        }

        println!("Full fill: 25 ETH in P2ID, no remainder");
    }

    #[test]
    fn test_inflight_only_fill() {
        // Inflight-only fill: input_amount=0, inflight=25 -> fill_amount=25.
        // Matches swapp_test.rs inflight cross-swap test pattern.
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let (p2id_note, remainder_note) =
            PswapNote::create_output_notes(&swap_note, f.bob_id, 0, 25).unwrap();

        assert!(remainder_note.is_none(), "Full inflight fill should produce no remainder");

        match p2id_note.assets().iter().next().unwrap() {
            Asset::Fungible(fa) => {
                assert_eq!(fa.faucet_id(), f.eth_faucet);
                assert_eq!(fa.amount(), 25, "P2ID should contain 25 ETH (inflight)");
            },
            _ => panic!("Expected fungible asset"),
        }

        // Aux should be [25, 0, 0, 0]
        let expected_aux = Word::from([Felt::new(25), ZERO, ZERO, ZERO]);
        assert_eq!(
            p2id_note.metadata().attachment().content(),
            NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_aux).content(),
            "Inflight-only aux should be [25, 0, 0, 0]"
        );

        println!("Inflight-only fill works correctly");
    }

    #[test]
    fn test_overfill_rejected() {
        // fill_amount > requested should be rejected.
        // Matches swapp_test.rs invalid input test.
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let result = PswapNote::create_output_notes(&swap_note, f.bob_id, 30, 0);
        assert!(result.is_err(), "Overfill (30 > 25) should be rejected");

        let result = PswapNote::create_output_notes(&swap_note, f.bob_id, 20, 10);
        assert!(result.is_err(), "Combined overfill (20+10=30 > 25) should be rejected");

        println!("Overfill correctly rejected");
    }

    #[test]
    fn test_zero_fill_rejected() {
        let f = TestFixture::new();
        let swap_note = f.create_swap_note(50, 25);

        let result = PswapNote::create_output_notes(&swap_note, f.bob_id, 0, 0);
        assert!(result.is_err(), "Zero fill should be rejected");

        println!("Zero fill correctly rejected");
    }

    #[test]
    fn test_multiple_partial_fill_scenarios() {
        // Runs through multiple partial fill scenarios matching
        // swapp_test.rs:swapp_note_multiple_partial_fills_test
        let f = TestFixture::new();

        let scenarios: Vec<(u64, u64, u64, u64)> = vec![
            // (input_amount, expected_offered_out, expected_remaining_usdc,
            // expected_remaining_eth)
            (5, 10, 40, 20),
            (7, 14, 36, 18),
            (10, 20, 30, 15),
            (13, 26, 24, 12),
            (15, 30, 20, 10),
            (19, 38, 12, 6),
            (20, 40, 10, 5),
            (23, 46, 4, 2),
            (25, 50, 0, 0), // full fill
        ];

        for (input_amount, expected_offered_out, expected_remaining_usdc, expected_remaining_eth) in
            scenarios
        {
            let swap_note = f.create_swap_note(50, 25);

            let (p2id_note, remainder_note) =
                PswapNote::create_output_notes(&swap_note, f.bob_id, input_amount, 0).unwrap();

            // Verify P2ID asset amount
            match p2id_note.assets().iter().next().unwrap() {
                Asset::Fungible(fa) => {
                    assert_eq!(
                        fa.amount(),
                        input_amount,
                        "P2ID should contain {} ETH",
                        input_amount
                    );
                },
                _ => panic!("Expected fungible asset"),
            }

            // Verify P2ID aux
            let expected_aux = Word::from([Felt::new(input_amount), ZERO, ZERO, ZERO]);
            assert_eq!(
                p2id_note.metadata().attachment().content(),
                NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_aux).content(),
            );

            if input_amount < 25 {
                let remainder = remainder_note.expect("Partial fill should have remainder");

                // Verify remainder offered asset
                match remainder.assets().iter().next().unwrap() {
                    Asset::Fungible(fa) => {
                        assert_eq!(
                            fa.amount(),
                            expected_remaining_usdc,
                            "Remainder should contain {} USDC for input={}",
                            expected_remaining_usdc,
                            input_amount
                        );
                    },
                    _ => panic!("Expected fungible asset"),
                }

                // Verify remainder requested amount
                match PswapNote::get_requested_asset(remainder.recipient().inputs().values())
                    .unwrap()
                {
                    Asset::Fungible(fa) => {
                        assert_eq!(
                            fa.amount(),
                            expected_remaining_eth,
                            "Remainder should request {} ETH for input={}",
                            expected_remaining_eth,
                            input_amount
                        );
                    },
                    _ => panic!("Expected fungible asset"),
                }

                // Verify remainder sender is consumer
                assert_eq!(remainder.metadata().sender(), f.bob_id);

                // Verify remainder attachment is [offered_out, 0, 0, 0]
                let expected_remainder_aux =
                    Word::from([Felt::new(expected_offered_out), ZERO, ZERO, ZERO]);
                assert_eq!(
                    remainder.metadata().attachment().content(),
                    NoteAttachment::new_word(NoteAttachmentScheme::none(), expected_remainder_aux)
                        .content(),
                );
            } else {
                assert!(remainder_note.is_none(), "Full fill should have no remainder");
            }

            println!(
                "  input={} ETH -> offered_out={} USDC, remainder=({} USDC, {} ETH)",
                input_amount, expected_offered_out, expected_remaining_usdc, expected_remaining_eth
            );
        }

        println!("All multiple partial fill scenarios passed");
    }

    #[tokio::test]
    async fn pswap_note_partial_fill_test() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use miden_standards::account::auth::NoAuth;
        use miden_testing::{Auth, MockChain};

        use crate::BasicWallet;

        println!("=== Test: Partial Fill Swap (Using PswapNote helpers) ===");
        let mut builder = MockChain::builder();

        // STEP 1: Create faucets
        println!("Creating USDC and ETH faucets...");
        let usdc_faucet =
            builder.add_existing_basic_faucet(Auth::BasicAuth, "USDC", 1000, Some(150))?;
        println!("USDC Faucet: {:?}", usdc_faucet.id());

        let eth_faucet =
            builder.add_existing_basic_faucet(Auth::BasicAuth, "ETH", 1000, Some(50))?;
        println!("ETH Faucet: {:?}", eth_faucet.id());

        // STEP 2: Create wallets
        println!("\nCreating Alice and Bob wallets...");
        let alice = builder.add_existing_wallet_with_assets(
            Auth::BasicAuth,
            [FungibleAsset::new(usdc_faucet.id(), 50)?.into()],
        )?;
        println!("Alice: {:?} (has 50 USDC)", alice.id());

        // Create Bob's wallet using BasicWallet component
        println!("\nCreating Bob's basic-wallet account...");
        let assets = vec![FungibleAsset::new(eth_faucet.id(), 25)?.into()];

        let bob = AccountBuilder::new([3u8; 32])
            .account_type(AccountType::RegularAccountUpdatableCode)
            .storage_mode(AccountStorageMode::Public)
            .with_component(BasicWallet::component())
            .with_auth_component(NoAuth::new())
            .with_assets(assets)
            .build_existing()
            .expect("Failed to build basic-wallet account");

        println!("Bob account created: {:?}", bob.id());

        let _bob_account = builder.add_account(bob.clone());

        // STEP 3: Create swap note using PswapNote::create
        println!("\nCreating swap note (Alice offers 50 USDC for 25 ETH)...");

        let offered_asset = Asset::Fungible(FungibleAsset::new(usdc_faucet.id(), 50)?);
        let requested_asset = Asset::Fungible(FungibleAsset::new(eth_faucet.id(), 25)?);

        use miden_crypto::rand::RpoRandomCoin;
        let mut rng = RpoRandomCoin::new(Word::default());

        let swap_note = PswapNote::create(
            alice.id(),
            offered_asset,
            requested_asset,
            NoteType::Public,
            NoteAttachment::default(),
            &mut rng,
        )?;

        println!("Swap note created: {:?}", swap_note.id());
        println!("  Used PswapNote::create");

        // Add note to genesis
        builder.add_output_note(OutputNote::Full(swap_note.clone()));

        // STEP 4: Build MockChain
        println!("\nBuilding MockChain...");
        let mock_chain = builder.build()?;

        // STEP 5: Bob provides 15 ETH (60% partial fill)
        println!("\nBob consuming swap note (providing 15 ETH - partial fill)...");
        let note_args = Word::from([
            Felt::ZERO,
            Felt::ZERO,
            Felt::ZERO,
            Felt::new(15), // input_amount = 15 (partial fill)
        ]);

        let mut note_args_map = BTreeMap::new();
        note_args_map.insert(swap_note.id(), note_args);

        // STEP 6: Use PswapNote::create_output_notes to build both expected output notes
        println!("\nCreating expected output notes using PswapNote::create_swap_output_notes...");
        let (expected_p2id_note, expected_remainder) =
            PswapNote::create_output_notes(&swap_note, bob.id(), 15, 0)?;

        assert!(expected_remainder.is_some(), "Partial fill should produce a remainder note");
        let expected_remainder_note = expected_remainder.unwrap();

        println!("  Expected P2ID note: {:?}", expected_p2id_note.id());
        println!("  Expected remainder note: {:?}", expected_remainder_note.id());

        // Build transaction context with both expected output notes
        let tx_context = mock_chain
            .build_tx_context(bob.id(), &[swap_note.id()], &[])?
            .extend_note_args(note_args_map)
            .extend_expected_output_notes(vec![
                OutputNote::Full(expected_p2id_note),
                OutputNote::Full(expected_remainder_note),
            ])
            .build()?;

        let executed_transaction = tx_context.execute().await?;

        println!(
            "Transaction executed! Cycle count: {:?}",
            executed_transaction.measurements().note_execution
        );

        // STEP 7: Verify results
        println!("\n=== Verification ===");

        // Should have 2 output notes: P2ID + remainder
        let output_notes = executed_transaction.output_notes();
        println!("Output notes created: {}", output_notes.num_notes());
        assert_eq!(output_notes.num_notes(), 2, "Expected 2 notes: 1 P2ID + 1 remainder");

        // Find and verify the P2ID note and remainder note
        let mut p2id_found = false;
        let mut remainder_found = false;

        for idx in 0..output_notes.num_notes() {
            let note = output_notes.get_note(idx);
            let assets = note.assets().unwrap();

            if assets.num_assets() == 1 {
                let asset = assets.iter().next().unwrap();
                if let Asset::Fungible(f) = asset {
                    if f.faucet_id() == eth_faucet.id() {
                        // P2ID note: contains 15 ETH for Alice
                        assert_eq!(f.amount(), 15, "P2ID note should contain 15 ETH");
                        println!("- P2ID note verified: 15 ETH for Alice");
                        p2id_found = true;
                    } else if f.faucet_id() == usdc_faucet.id() {
                        // Remainder note: contains 20 USDC (50 - 30 = 20)
                        // offered_out = (50 * 15) / 25 = 30, remaining = 50 - 30 = 20
                        assert_eq!(f.amount(), 20, "Remainder note should contain 20 USDC");
                        println!("- Remainder note verified: 20 USDC (requesting 10 ETH)");
                        remainder_found = true;
                    }
                }
            }
        }

        assert!(p2id_found, "P2ID note not found in output");
        assert!(remainder_found, "Remainder swap note not found in output");

        // Verify Bob's vault delta
        let account_delta = executed_transaction.account_delta();
        let vault_delta = account_delta.vault();
        let added_assets: Vec<Asset> = vault_delta.added_assets().collect();

        assert_eq!(added_assets.len(), 1, "Bob should receive 1 asset");
        let usdc_received = match added_assets[0] {
            Asset::Fungible(f) => f,
            _ => panic!("Expected fungible USDC asset"),
        };
        assert_eq!(usdc_received.faucet_id(), usdc_faucet.id(), "Bob should receive USDC");
        assert_eq!(usdc_received.amount(), 30, "Bob should receive 30 USDC");
        println!("- Bob's vault delta verified: +30 USDC, -15 ETH");

        println!("\nPartial-fill swap test passed (using PswapNote helpers)!");
        println!("  - Bob provided 15 ETH (60% of requested 25)");
        println!("  - Bob received 30 USDC (60% of offered 50)");
        println!("  - P2ID note: 15 ETH for Alice");
        println!("  - Remainder note: 20 USDC still requesting 10 ETH");
        println!("  - Both output notes built via PswapNote::create_swap_output_notes");

        Ok(())
    }
}
