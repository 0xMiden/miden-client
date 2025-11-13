use anyhow::{Context, Result};
use miden_client::EMPTY_WORD;
use miden_client::account::{AccountStorageMode, build_wallet_id};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::auth::AuthSecretKey;
use miden_client::note::{NoteFile, NoteType};
use miden_client::rpc::{AcceptHeaderError, RpcError};
use miden_client::store::{InputNoteState, NoteFilter};
use miden_client::testing::common::*;
use miden_client::transaction::{InputNote, PaymentNoteDescription, TransactionRequestBuilder};
use rand::RngCore;

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
    let (faucet_account, _) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Private, &keystore_1).await?;

    // Create regular accounts
    let (basic_wallet_1, ..) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &keystore_2).await?;

    // Create regular accounts
    let (basic_wallet_2, ..) =
        insert_new_wallet(&mut client_3, AccountStorageMode::Private, &keystore_3).await?;

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

    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![note.id()])?;
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

    let (faucet_account_header, secret_key) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Public, &keystore_1).await?;

    let (first_regular_account, ..) =
        insert_new_wallet(&mut client_1, AccountStorageMode::Private, &keystore_1).await?;

    let (second_client_first_regular_account, ..) =
        insert_new_wallet(&mut client_2, AccountStorageMode::Private, &keystore_2).await?;

    let target_account_id = first_regular_account.id();
    let second_client_target_account_id = second_client_first_regular_account.id();
    let faucet_account_id = faucet_account_header.id();

    keystore_2.add_key(&secret_key)?;
    client_2.add_account(&faucet_account_header, false).await?;

    // First Mint necessary token
    println!("First client consuming note");
    client_1.sync_state().await?;
    let (tx_id, note) =
        mint_note(&mut client_1, target_account_id, faucet_account_id, NoteType::Private).await;
    wait_for_tx(&mut client_1, tx_id).await?;

    // Update the state in the other client and ensure the onchain faucet commitment is consistent
    // between clients
    client_2.sync_state().await?;

    let (client_1_faucet, _) = client_1
        .get_account_header_by_id(faucet_account_header.id())
        .await?
        .context("failed to find faucet account in client 1 after sync")?;
    let (client_2_faucet, _) = client_2
        .get_account_header_by_id(faucet_account_header.id())
        .await?
        .context("failed to find faucet account in client 2 after sync")?;

    assert_eq!(client_1_faucet.commitment(), client_2_faucet.commitment());

    // Now use the faucet in the second client to mint to its own account
    println!("Second client consuming note");
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

    println!("About to consume");
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

    let (client_1_faucet, _) = client_1
        .get_account_header_by_id(faucet_account_header.id())
        .await?
        .context("failed to find faucet account in client 1 after consume transactions")?;
    let (client_2_faucet, _) = client_2
        .get_account_header_by_id(faucet_account_header.id())
        .await?
        .context("failed to find faucet account in client 2 after consume transactions")?;

    assert_eq!(client_1_faucet.commitment(), client_2_faucet.commitment());

    // Now we'll try to do a p2id transfer from an account of one client to the other one
    let from_account_id = target_account_id;
    let to_account_id = second_client_target_account_id;

    // get initial balances
    let from_account_balance = client_1
        .get_account(from_account_id)
        .await?
        .context("failed to find from account for balance check")?
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let to_account_balance = client_2
        .get_account(to_account_id)
        .await?
        .context("failed to find to account for balance check")?
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);

    let asset = FungibleAsset::new(faucet_account_id, TRANSFER_AMOUNT)?;

    println!("Running P2ID tx...");
    let tx_request = TransactionRequestBuilder::new().build_pay_to_id(
        PaymentNoteDescription::new(vec![Asset::Fungible(asset)], from_account_id, to_account_id),
        NoteType::Public,
        client_1.rng(),
    )?;
    execute_tx_and_sync(&mut client_1, from_account_id, tx_request).await?;

    // sync on second client until we receive the note
    println!("Syncing on second client...");
    client_2.sync_state().await?;
    let notes = client_2.get_input_notes(NoteFilter::Committed).await?;

    //Import the note on the first client so that we can later check its consumer account
    client_1.import_note(NoteFile::NoteId(notes[0].id())).await?;

    // Consume the note
    println!("Consuming note on second client...");
    let tx_request = TransactionRequestBuilder::new().build_consume_notes(vec![notes[0].id()])?;
    execute_tx_and_sync(&mut client_2, to_account_id, tx_request).await?;

    // sync on first client
    println!("Syncing on first client...");
    client_1.sync_state().await?;

    // Check that the client doesn't know who consumed the note
    let input_note = client_1
        .get_input_note(notes[0].id())
        .await?
        .with_context(|| format!("input note {} not found", notes[0].id()))?;
    assert!(matches!(input_note.state(), InputNoteState::ConsumedExternal { .. }));

    let new_from_account_balance = client_1
        .get_account(from_account_id)
        .await?
        .context("failed to find from account after transfer")?
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);
    let new_to_account_balance = client_2
        .get_account(to_account_id)
        .await?
        .context("failed to find to account after transfer")?
        .account()
        .vault()
        .get_balance(faucet_account_id)
        .unwrap_or(0);

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

    let (faucet_account_header, _) =
        insert_new_fungible_faucet(&mut client_1, AccountStorageMode::Public, &keystore_1).await?;

    let (first_regular_account, secret_key) = insert_new_wallet_with_seed(
        &mut client_1,
        AccountStorageMode::Public,
        &keystore_1,
        user_seed,
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
        build_wallet_id(user_seed, secret_key.public_key(), AccountStorageMode::Public, false)?;
    assert_eq!(built_wallet_id, first_regular_account.id());
    client_2.import_account_by_id(built_wallet_id).await?;
    keystore_2.add_key(&AuthSecretKey::RpoFalcon512(secret_key))?;

    let original_account =
        client_1.get_account(first_regular_account.id()).await?.with_context(|| {
            format!("Original account {} not found in client_1", first_regular_account.id())
        })?;
    let imported_account =
        client_2.get_account(first_regular_account.id()).await?.with_context(|| {
            format!("Imported account {} not found in client_2", first_regular_account.id())
        })?;
    assert_eq!(imported_account.account().commitment(), original_account.account().commitment());

    // Now use the wallet in the second client to consume the generated note
    println!("Second client consuming note");
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
        Err(RpcError::AcceptHeaderError(AcceptHeaderError::NoSupportedMediaRange)) => Ok(()),
        Ok(_) => anyhow::bail!("grpc request was unexpectedly successful"),
        _ => anyhow::bail!("expected accept header error"),
    }
}
