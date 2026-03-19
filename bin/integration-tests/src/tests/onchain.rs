use anyhow::{Context, Result};
use miden_client::EMPTY_WORD;
use miden_client::account::{AccountStorageMode, build_wallet_id};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::RPO_FALCON_SCHEME_ID;
use miden_client::keystore::Keystore;
use miden_client::note::{NoteFile, NoteType};
use miden_client::rpc::{AcceptHeaderError, RpcError};
use miden_client::store::{InputNoteState, NoteFilter};
use miden_client::testing::common::*;
use miden_client::transaction::{
    InputNote,
    PaymentNoteDescription,
    TransactionRequestBuilder,
    TransactionStatus,
};
use rand::RngCore;
use tracing::info;

use crate::tests::config::ClientConfig;

// TESTS
// ================================================================================================

pub async fn test_onchain_notes_flow(client_config: ClientConfig) -> Result<()> {
    // Client 1 is an private faucet which will mint an onchain note for client 2
    let (mut client_1, keystore_1) = client_config.clone().into_client().await?;
    // Client 2 is an private account which will consume the note that it will sync from the node
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    // Client 3 will be transferred part of the assets by client 2's account
    let (mut client_3, keystore_3) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    wait_for_node(&mut client_3).await;

    // Create faucet account
    let (faucet_account, _) = insert_new_fungible_faucet(
        &mut client_1,
        AccountStorageMode::Private,
        &keystore_1,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    // Create regular accounts
    let (basic_wallet_1, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Private,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    // Create regular accounts
    let (basic_wallet_2, ..) = insert_new_wallet(
        &mut client_3,
        AccountStorageMode::Private,
        &keystore_3,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;
    client_1.sync_state().await?;
    client_2.sync_state().await?;

    let tx_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        FungibleAsset::new(faucet_account.id(), MINT_AMOUNT)?,
        basic_wallet_1.id(),
        NoteType::Public,
        client_1.rng(),
    )?;
    let note = tx_request
        .expected_output_own_notes()
        .pop()
        .with_context(|| "no expected output notes found in onchain transaction from faucet")?
        .clone();
    execute_tx_and_sync(&mut client_1, faucet_account.id(), tx_request).await?;

    // Client 2's account should receive the note here:
    client_2.sync_state().await?;

    // Assert that the note is the same
    let received_note: InputNote = client_2
        .get_input_note(note.id())
        .await?
        .with_context(|| format!("Note {} not found in client_2", note.id()))?
        .try_into()?;
    assert_eq!(received_note.note().commitment(), note.commitment());

    // TODO: revisit this.
    // The received note has the uri of the note stored in the node, so it may not match with the
    // original note.
    // assert_eq!(received_note.note(), &note);

    // consume the note
    let tx_id =
        consume_notes(&mut client_2, basic_wallet_1.id(), &[received_note.note().clone()]).await;
    wait_for_tx(&mut client_2, tx_id).await?;
    assert_account_has_single_asset(
        &client_2,
        basic_wallet_1.id(),
        faucet_account.id(),
        MINT_AMOUNT,
    )
    .await;

    let p2id_asset = FungibleAsset::new(faucet_account.id(), TRANSFER_AMOUNT)?;
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(
            vec![p2id_asset.into()],
            basic_wallet_1.id(),
            basic_wallet_2.id(),
        ),
        NoteType::Public,
        client_2.rng(),
    )?;
    execute_tx_and_sync(&mut client_2, basic_wallet_1.id(), tx_request).await?;

    // Create a note for client 3 that is already consumed before syncing
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(
            vec![p2id_asset.into()],
            basic_wallet_1.id(),
            basic_wallet_2.id(),
        )
        .with_reclaim_height(1.into()),
        NoteType::Public,
        client_2.rng(),
    )?;
    let note = tx_request
        .expected_output_own_notes()
        .pop()
        .with_context(|| "no expected output notes found in onchain transaction from basic wallet")?
        .clone();
    execute_tx_and_sync(&mut client_2, basic_wallet_1.id(), tx_request).await?;

    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.clone()])?;
    execute_tx_and_sync(&mut client_2, basic_wallet_1.id(), tx_request).await?;

    // sync client 3 (basic account 2)
    client_3.sync_state().await?;

    // client 3 should have two notes, the one directed to them and the one consumed by client 2
    // (which should come from the tag added)
    assert_eq!(client_3.get_input_notes(NoteFilter::Committed).await?.len(), 1);
    assert_eq!(client_3.get_input_notes(NoteFilter::Consumed).await?.len(), 1);

    let note = client_3
        .get_input_notes(NoteFilter::Committed)
        .await?
        .first()
        .with_context(|| "no committed input notes found")?
        .clone()
        .try_into()?;

    let tx_id = consume_notes(&mut client_3, basic_wallet_2.id(), &[note]).await;
    wait_for_tx(&mut client_3, tx_id).await?;
    assert_account_has_single_asset(
        &client_3,
        basic_wallet_2.id(),
        faucet_account.id(),
        TRANSFER_AMOUNT,
    )
    .await;
    Ok(())
}

pub async fn test_onchain_accounts(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, keystore_1) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    wait_for_node(&mut client_2).await;

    let (faucet_account_header, secret_key) = insert_new_fungible_faucet(
        &mut client_1,
        AccountStorageMode::Public,
        &keystore_1,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (first_regular_account, ..) = insert_new_wallet(
        &mut client_1,
        AccountStorageMode::Private,
        &keystore_1,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (second_client_first_regular_account, ..) = insert_new_wallet(
        &mut client_2,
        AccountStorageMode::Private,
        &keystore_2,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let target_account_id = first_regular_account.id();
    let second_client_target_account_id = second_client_first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    keystore_2.add_key(&secret_key, faucet_account_id).await?;
    client_2.add_account(&faucet_account_header, false).await?;

    // First Mint necessary token
    info!(account_id = %target_account_id, faucet_id = %faucet_account_id, "First client minting note");
    client_1.sync_state().await?;
    let (tx_id, note) =
        mint_note(&mut client_1, target_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client_1, tx_id).await?;

    // Update the state in the other client and ensure the onchain faucet commitment is consistent
    // between clients
    client_2.sync_state().await?;

    let (client_1_faucet, _) = client_1
        .account_reader(faucet_account_header.id())
        .header()
        .await
        .context("failed to find faucet account in client 1 after sync")?;
    let (client_2_faucet, _) = client_2
        .account_reader(faucet_account_header.id())
        .header()
        .await
        .context("failed to find faucet account in client 2 after sync")?;

    assert_eq!(client_1_faucet.to_commitment(), client_2_faucet.to_commitment());

    // Now use the faucet in the second client to mint to its own account
    info!(account_id = %second_client_target_account_id, faucet_id = %faucet_account_id, "Second client minting note");
    let (tx_id, second_client_note) = mint_note(
        &mut client_2,
        second_client_target_account_id,
        faucet_account_id,
        NoteType::Private,
    )
    .await;
    wait_for_tx(&mut client_2, tx_id).await?;

    // Update the state in the other client and ensure the onchain faucet commitment is consistent
    // between clients
    client_1.sync_state().await?;

    info!(account_id = %target_account_id, "Consuming note on first client");
    let tx_id = consume_notes(&mut client_1, target_account_id, &[note]).await;
    wait_for_tx(&mut client_1, tx_id).await?;
    assert_account_has_single_asset(&client_1, target_account_id, faucet_account_id, MINT_AMOUNT)
        .await;
    let tx_id =
        consume_notes(&mut client_2, second_client_target_account_id, &[second_client_note]).await;
    wait_for_tx(&mut client_2, tx_id).await?;
    assert_account_has_single_asset(
        &client_2,
        second_client_target_account_id,
        faucet_account_id,
        MINT_AMOUNT,
    )
    .await;

    let (client_1_faucet, _) =
        client_1
            .account_reader(faucet_account_header.id())
            .header()
            .await
            .context("failed to find faucet account in client 1 after consume transactions")?;
    let (client_2_faucet, _) =
        client_2
            .account_reader(faucet_account_header.id())
            .header()
            .await
            .context("failed to find faucet account in client 2 after consume transactions")?;

    assert_eq!(client_1_faucet.to_commitment(), client_2_faucet.to_commitment());

    // Now we'll try to do a p2id transfer from an account of one client to the other one
    let from_account_id = target_account_id;
    let to_account_id = second_client_target_account_id;

    // get initial balances
    let from_account_balance = client_1
        .account_reader(from_account_id)
        .get_balance(faucet_account_id)
        .await
        .context("failed to find from account for balance check")?;
    let to_account_balance = client_2
        .account_reader(to_account_id)
        .get_balance(faucet_account_id)
        .await
        .context("failed to find to account for balance check")?;

    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT)?;

    info!(from = %from_account_id, to = %to_account_id, amount = TRANSFER_AMOUNT, "Running P2ID transaction");
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id),
        NoteType::Public,
        client_1.rng(),
    )?;
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;

    // sync on second client until we receive the note
    info!("Syncing state on second client");
    client_2.sync_state().await?;
    let notes = client_2.get_input_notes(NoteFilter::Committed).await?;

    //Import the note on the first client so that we can later check its consumer account
    client_1.import_notes(&[NoteFile::NoteId(notes[0].id())]).await?;

    // Consume the note
    info!(note_id = %notes[0].id(), account_id = %to_account_id, "Consuming note on second client");
    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![notes[0].clone().try_into().unwrap()])?;
    execute_tx_and_sync(&mut client_2, to_account_id, tx_request).await?;

    // sync on first client
    info!("Syncing state on first client");
    client_1.sync_state().await?;

    // Check that the client doesn't know who consumed the note
    let input_note = client_1
        .get_input_note(notes[0].id())
        .await?
        .with_context(|| format!("input note {} not found", notes[0].id()))?;
    assert!(matches!(input_note.state(), InputNoteState::ConsumedExternal { .. }));

    let new_from_account_balance = client_1
        .account_reader(from_account_id)
        .get_balance(faucet_account_id)
        .await
        .context("failed to find from account after transfer")?;
    let new_to_account_balance = client_2
        .account_reader(to_account_id)
        .get_balance(faucet_account_id)
        .await
        .context("failed to find to account after transfer")?;

    assert_eq!(new_from_account_balance, from_account_balance - TRANSFER_AMOUNT);
    assert_eq!(new_to_account_balance, to_account_balance + TRANSFER_AMOUNT);
    Ok(())
}

pub async fn test_import_account_by_id(client_config: ClientConfig) -> Result<()> {
    let (mut client_1, keystore_1) = client_config.clone().into_client().await?;
    let (mut client_2, keystore_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;
    wait_for_node(&mut client_1).await;

    let mut user_seed = [0u8; 32];
    client_1.rng().fill_bytes(&mut user_seed);

    let (faucet_account_header, _) = insert_new_fungible_faucet(
        &mut client_1,
        AccountStorageMode::Public,
        &keystore_1,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (first_regular_account, secret_key) = insert_new_wallet_with_seed(
        &mut client_1,
        AccountStorageMode::Public,
        &keystore_1,
        user_seed,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let target_account_id = first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    // First mint and consume in the first client
    let tx_id =
        mint_and_consume(&mut client_1, target_account_id, faucet_account_id, NoteType::Public)
            .await;
    wait_for_tx(&mut client_1, tx_id).await?;

    // Mint a note for the second client
    let (tx_id, note) =
        mint_note(&mut client_1, target_account_id, faucet_account_id, NoteType::Public).await;
    wait_for_tx(&mut client_1, tx_id).await?;

    // Import the public account by id
    let built_wallet_id =
        build_wallet_id(user_seed, &secret_key.public_key(), AccountStorageMode::Public, false)?;
    assert_eq!(built_wallet_id, first_regular_account.id());
    client_2.import_account_by_id(built_wallet_id).await?;
    keystore_2.add_key(&secret_key, built_wallet_id).await?;

    let original_commitment = client_1
        .account_reader(first_regular_account.id())
        .commitment()
        .await
        .with_context(|| {
            format!("Original account {} not found in client_1", first_regular_account.id())
        })?;
    let imported_commitment = client_2
        .account_reader(first_regular_account.id())
        .commitment()
        .await
        .with_context(|| {
            format!("Imported account {} not found in client_2", first_regular_account.id())
        })?;
    assert_eq!(imported_commitment, original_commitment);

    // Now use the wallet in the second client to consume the generated note
    info!(account_id = %target_account_id, "Second client consuming note");
    client_2.sync_state().await?;
    let tx_id = consume_notes(&mut client_2, target_account_id, &[note]).await;
    wait_for_tx(&mut client_2, tx_id).await?;
    assert_account_has_single_asset(
        &client_2,
        target_account_id,
        faucet_account_id,
        MINT_AMOUNT * 2,
    )
    .await;
    Ok(())
}

pub async fn test_incorrect_genesis(client_config: ClientConfig) -> Result<()> {
    let (builder, _) = client_config.into_client_builder().await?;
    let mut client = builder.build().await?;

    // Set an incorrect genesis commitment
    client.test_rpc_api().set_genesis_commitment(EMPTY_WORD).await?;

    // This request would always be valid as it requests the chain tip. But it should fail
    // because the genesis commitment in the request header does not match the one in the node.
    let result = client.test_rpc_api().get_block_header_by_number(None, false).await;

    match result {
        Err(RpcError::AcceptHeaderError(AcceptHeaderError::NoSupportedMediaRange(_))) => Ok(()),
        Ok(_) => anyhow::bail!("grpc request was unexpectedly successful"),
        _ => anyhow::bail!("expected accept header error"),
    }
}

/// Tests that consumed notes are returned in the correct transaction order when multiple
/// consume transactions for the same account are included in the same block.
///
/// The test mints 3 notes, then submits 3 separate consume transactions rapidly so they
/// are likely included in the same block. After syncing, it verifies that the
/// `InputNoteReader` returns the notes ordered by their consumption order.
pub async fn test_consumed_note_ordering(client_config: ClientConfig) -> Result<()> {
    let (mut client, keystore) = client_config.clone().into_client().await?;
    wait_for_node(&mut client).await;

    // Create faucet and wallet
    let (faucet_account, _) = insert_new_fungible_faucet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    let (wallet_account, ..) = insert_new_wallet(
        &mut client,
        AccountStorageMode::Private,
        &keystore,
        RPO_FALCON_SCHEME_ID,
    )
    .await?;

    client.sync_state().await?;

    // Mint 3 notes, each in a separate transaction
    let mut minted_notes = Vec::new();
    for i in 0..3 {
        let (tx_id, note) =
            mint_note(&mut client, wallet_account.id(), faucet_account.id(), NoteType::Private)
                .await;
        info!(tx_id = %tx_id, note_id = %note.id(), index = i, "Minted note");
        wait_for_tx(&mut client, tx_id).await?;
        minted_notes.push(note);
    }

    // Sync to pick up the minted notes
    client.sync_state().await?;

    // Submit 3 separate consume transactions without waiting between them.
    // This makes it likely they will be included in the same block, which tests
    // the consumed_tx_order field within a single block.
    let mut consume_tx_ids = Vec::new();
    for (i, note) in minted_notes.iter().enumerate() {
        let tx_id =
            consume_notes(&mut client, wallet_account.id(), core::slice::from_ref(note)).await;
        info!(tx_id = %tx_id, note_id = %note.id(), index = i, "Submitted consume transaction");
        consume_tx_ids.push(tx_id);
    }

    // Wait for all consume transactions to be committed
    for tx_id in &consume_tx_ids {
        wait_for_tx(&mut client, *tx_id).await?;
    }

    // Sync to apply the state updates
    client.sync_state().await?;

    // Verify all notes are consumed
    let consumed_notes = client.get_input_notes(NoteFilter::Consumed).await?;
    assert!(
        consumed_notes.len() >= 3,
        "Expected at least 3 consumed notes, got {}",
        consumed_notes.len()
    );

    // Check if all consume transactions landed in the same block
    let tx_records = client.get_transactions(miden_client::store::TransactionFilter::All).await?;
    let consume_blocks: Vec<_> = consume_tx_ids
        .iter()
        .filter_map(|tx_id| {
            tx_records.iter().find(|t| t.id == *tx_id).and_then(|t| {
                if let TransactionStatus::Committed { block_number, .. } = t.status {
                    Some(block_number)
                } else {
                    None
                }
            })
        })
        .collect();

    info!(?consume_blocks, "Consume transaction block numbers");

    // Use InputNoteReader to iterate consumed notes for this wallet
    let mut reader = client.input_note_reader().for_consumer(wallet_account.id());
    let mut reader_notes = Vec::new();
    while let Some(note) = reader.next().await? {
        reader_notes.push(note);
    }

    assert!(
        reader_notes.len() >= 3,
        "Expected at least 3 notes from reader, got {}",
        reader_notes.len()
    );

    // Extract the nullifier block height from a consumed note state
    let consumed_block_height = |note: &miden_client::store::InputNoteRecord| -> Option<u32> {
        match note.state() {
            InputNoteState::ConsumedAuthenticatedLocal(s) => {
                Some(s.nullifier_block_height.as_u32())
            },
            InputNoteState::ConsumedUnauthenticatedLocal(s) => {
                Some(s.nullifier_block_height.as_u32())
            },
            InputNoteState::ConsumedExternal(s) => Some(s.nullifier_block_height.as_u32()),
            _ => None,
        }
    };

    // Verify the notes are ordered by block height, then by tx_order within a block
    for window in reader_notes.windows(2) {
        let a = &window[0];
        let b = &window[1];

        let a_block = consumed_block_height(a).expect("consumed note should have block height");
        let b_block = consumed_block_height(b).expect("consumed note should have block height");

        assert!(
            a_block <= b_block,
            "Notes should be ordered by block height: note {} at block {} came before note {} at block {}",
            a.id(),
            a_block,
            b.id(),
            b_block,
        );
    }

    // If all transactions landed in the same block, additionally verify the note IDs
    // match the order we submitted them in
    let all_same_block =
        consume_blocks.len() == 3 && consume_blocks.iter().all(|b| *b == consume_blocks[0]);

    if all_same_block {
        info!("All consume transactions in the same block - verifying tx_order");
        let reader_note_ids: Vec<_> = reader_notes.iter().map(|n| n.id()).collect();
        for (i, note) in minted_notes.iter().enumerate() {
            let pos = reader_note_ids
                .iter()
                .position(|id| *id == note.id())
                .with_context(|| format!("Minted note {} not found in reader output", note.id()))?;
            info!(note_id = %note.id(), expected_order = i, actual_pos = pos, "Note position");
        }

        // Verify that the relative order of our 3 notes matches submission order
        let positions: Vec<_> = minted_notes
            .iter()
            .filter_map(|note| reader_note_ids.iter().position(|id| *id == note.id()))
            .collect();

        assert_eq!(positions.len(), 3, "All 3 minted notes should be in the reader output");
        assert!(
            positions.windows(2).all(|w| w[0] < w[1]),
            "Notes should appear in submission order, but got positions: {:?}",
            positions
        );
    } else {
        info!(
            "Consume transactions spread across multiple blocks - order verified by block height"
        );
    }

    Ok(())
}
