use anyhow::{Context, Result};
use miden_client::account::{Account, AccountStorageMode};
use miden_client::asset::{Asset, FungibleAsset};
use miden_client::note::{Note, NoteDetails, NoteFile, NoteType, build_swap_tag};
use miden_client::testing::common::*;
use miden_client::transaction::{SwapTransactionData, TransactionRequestBuilder};

use crate::tests::config::ClientConfig;

// SWAP FULLY ONCHAIN
// ================================================================================================

pub async fn test_swap_fully_onchain(client_config: ClientConfig) -> Result<()> {
    const OFFERED_ASSET_AMOUNT: u64 = 1;
    const REQUESTED_ASSET_AMOUNT: u64 = 25;
    let (mut client1, authenticator_1) = client_config.clone().into_client().await?;
    wait_for_node(&mut client1).await;
    let (mut client2, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    client1.sync_state().await?;
    client2.sync_state().await?;

    // Create Client 1's basic wallet (We'll call it accountA)
    let (account_a, ..) =
        insert_new_wallet(&mut client1, AccountStorageMode::Private, &authenticator_1).await?;

    // Create Client 2's basic wallet (We'll call it accountB)
    let (account_b, ..) =
        insert_new_wallet(&mut client2, AccountStorageMode::Private, &authenticator_2).await?;

    // Create client with faucets BTC faucet (note: it's not real BTC)
    let (btc_faucet_account, ..) =
        insert_new_fungible_faucet(&mut client1, AccountStorageMode::Private, &authenticator_1)
            .await?;

    // Create client with faucets ETH faucet (note: it's not real ETH)
    let (eth_faucet_account, ..) =
        insert_new_fungible_faucet(&mut client2, AccountStorageMode::Private, &authenticator_2)
            .await?;

    // mint 1000 BTC for accountA
    println!("minting 1000 btc for account A");

    let tx_id =
        mint_and_consume(&mut client1, account_a.id(), btc_faucet_account.id(), NoteType::Public)
            .await;
    wait_for_tx(&mut client1, tx_id).await?;

    // mint 1000 ETH for accountB
    println!("minting 1000 eth for account B");

    let tx_id =
        mint_and_consume(&mut client2, account_b.id(), eth_faucet_account.id(), NoteType::Public)
            .await;
    wait_for_tx(&mut client2, tx_id).await?;

    // Create ONCHAIN swap note (clientA offers 1 BTC in exchange of 25 ETH)
    // check that account now has 1 less BTC
    println!("creating swap note with accountA");
    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_ASSET_AMOUNT)?;
    let requested_asset = FungibleAsset::new(eth_faucet_account.id(), REQUESTED_ASSET_AMOUNT)?;

    println!("Running SWAP tx...");
    let tx_request = TransactionRequestBuilder::new().build_swap(
        &SwapTransactionData::new(
            account_a.id(),
            Asset::Fungible(offered_asset),
            Asset::Fungible(requested_asset),
        ),
        NoteType::Public,
        NoteType::Private,
        client1.rng(),
    )?;

    let expected_output_notes: Vec<Note> = tx_request.expected_output_own_notes();
    let expected_payback_note_details: Vec<NoteDetails> =
        tx_request.expected_future_notes().cloned().map(|(n, _)| n).collect();
    assert_eq!(expected_output_notes.len(), 1);
    assert_eq!(expected_payback_note_details.len(), 1);

    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await?;

    let swap_note_tag = build_swap_tag(
        NoteType::Public,
        &Asset::Fungible(offered_asset),
        &Asset::Fungible(requested_asset),
    )?;

    // add swap note's tag to client2
    // we could technically avoid this step, but for the first iteration of swap notes we'll
    // require to manually add tags
    println!("Adding swap tag");
    client2.add_note_tag(swap_note_tag).await?;

    // sync on client 2, we should get the swap note
    // consume swap note with accountB, and check that the vault changed appropriately
    client2.sync_state().await?;
    println!("Consuming swap note on second client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_output_notes[0].id()])?;
    execute_tx_and_sync(&mut client2, account_b.id(), tx_request).await?;

    // sync on client 1, we should get the missing payback note details.
    // try consuming the received note with accountA, it should now have 25 ETH
    client1.sync_state().await?;
    println!("Consuming swap payback note on first client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_payback_note_details[0].id()])?;
    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await?;

    // At the end we should end up with
    //
    // - accountA: 999 BTC, 25 ETH
    // - accountB: 1 BTC, 975 ETH

    // first reload the account
    let account_a: Account = client1
        .get_account(account_a.id())
        .await?
        .context("failed to find account A after swap transaction")?
        .into();
    let account_a_assets = account_a.vault().assets();
    assert_eq!(account_a_assets.count(), 2);
    let mut account_a_assets = account_a.vault().assets();

    let asset_1 = account_a_assets
        .next()
        .context("expected first asset in account A;s vault after swap")?;
    let asset_2 = account_a_assets
        .next()
        .context("expected second asset in account A's vault after swap")?;

    match (asset_1, asset_2) {
        (Asset::Fungible(btc_asset), Asset::Fungible(eth_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 999);
            assert_eq!(eth_asset.amount(), 25);
        },
        (Asset::Fungible(eth_asset), Asset::Fungible(btc_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 999);
            assert_eq!(eth_asset.amount(), 25);
        },
        _ => panic!("should only have fungible assets!"),
    }

    let account_b: Account = client2
        .get_account(account_b.id())
        .await?
        .context("failed to find account B after swap transaction")?
        .into();
    let account_b_assets = account_b.vault().assets();
    assert_eq!(account_b_assets.count(), 2);
    let mut account_b_assets = account_b.vault().assets();

    let asset_1 = account_b_assets
        .next()
        .context("expected first asset in account B's vault after swap")?;
    let asset_2 = account_b_assets
        .next()
        .context("expected second asset in account B's vault after swap")?;

    match (asset_1, asset_2) {
        (Asset::Fungible(btc_asset), Asset::Fungible(eth_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 1);
            assert_eq!(eth_asset.amount(), 975);
        },
        (Asset::Fungible(eth_asset), Asset::Fungible(btc_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 1);
            assert_eq!(eth_asset.amount(), 975);
        },
        _ => panic!("should only have fungible assets!"),
    }
    Ok(())
}

pub async fn test_swap_private(client_config: ClientConfig) -> Result<()> {
    const OFFERED_ASSET_AMOUNT: u64 = 1;
    const REQUESTED_ASSET_AMOUNT: u64 = 25;
    let (mut client1, authenticator_1) = client_config.clone().into_client().await?;
    wait_for_node(&mut client1).await;
    let (mut client2, authenticator_2) = ClientConfig::default()
        .with_rpc_endpoint(client_config.rpc_endpoint())
        .into_client()
        .await?;

    client1.sync_state().await?;
    client2.sync_state().await?;

    // Create Client 1's basic wallet (We'll call it accountA)
    let (account_a, ..) =
        insert_new_wallet(&mut client1, AccountStorageMode::Private, &authenticator_1).await?;

    // Create Client 2's basic wallet (We'll call it accountB)
    let (account_b, ..) =
        insert_new_wallet(&mut client2, AccountStorageMode::Private, &authenticator_2).await?;

    // Create client with faucets BTC faucet (note: it's not real BTC)
    let (btc_faucet_account, ..) =
        insert_new_fungible_faucet(&mut client1, AccountStorageMode::Private, &authenticator_1)
            .await?;
    // Create client with faucets ETH faucet (note: it's not real ETH)
    let (eth_faucet_account, ..) =
        insert_new_fungible_faucet(&mut client2, AccountStorageMode::Private, &authenticator_2)
            .await?;

    // mint 1000 BTC for accountA
    println!("minting 1000 btc for account A");
    let tx_id =
        mint_and_consume(&mut client1, account_a.id(), btc_faucet_account.id(), NoteType::Public)
            .await;
    wait_for_tx(&mut client1, tx_id).await?;

    // mint 1000 ETH for accountB
    println!("minting 1000 eth for account B");
    let tx_id =
        mint_and_consume(&mut client2, account_b.id(), eth_faucet_account.id(), NoteType::Public)
            .await;
    wait_for_tx(&mut client2, tx_id).await?;

    // Create ONCHAIN swap note (clientA offers 1 BTC in exchange of 25 ETH)
    // check that account now has 1 less BTC
    println!("creating swap note with accountA");
    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_ASSET_AMOUNT)?;
    let requested_asset = FungibleAsset::new(eth_faucet_account.id(), REQUESTED_ASSET_AMOUNT)?;

    println!("Running SWAP tx...");
    let tx_request = TransactionRequestBuilder::new().build_swap(
        &SwapTransactionData::new(
            account_a.id(),
            Asset::Fungible(offered_asset),
            Asset::Fungible(requested_asset),
        ),
        NoteType::Private,
        NoteType::Private,
        client1.rng(),
    )?;

    let expected_output_notes: Vec<Note> = tx_request.expected_output_own_notes();
    let expected_payback_note_details =
        tx_request.expected_future_notes().cloned().map(|(n, _)| n).collect::<Vec<_>>();
    assert_eq!(expected_output_notes.len(), 1);
    assert_eq!(expected_payback_note_details.len(), 1);

    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await?;

    // Export note from client 1 to client 2
    let output_note = client1
        .get_output_note(expected_output_notes[0].id())
        .await?
        .with_context(|| format!("Output note {} not found", expected_output_notes[0].id()))?;

    let tag = build_swap_tag(
        NoteType::Private,
        &Asset::Fungible(offered_asset),
        &Asset::Fungible(requested_asset),
    )?;
    client2.add_note_tag(tag).await?;
    client2
        .import_note(NoteFile::NoteDetails {
            details: output_note.try_into()?,
            after_block_num: client1.get_sync_height().await?,
            tag: Some(tag),
        })
        .await?;

    // Sync so we get the inclusion proof info
    client2.sync_state().await?;

    // consume swap note with accountB, and check that the vault changed appropriately
    println!("Consuming swap note on second client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_output_notes[0].id()])?;
    execute_tx_and_sync(&mut client2, account_b.id(), tx_request).await?;

    // sync on client 1, we should get the missing payback note details.
    // try consuming the received note with accountA, it should now have 25 ETH
    client1.sync_state().await?;
    println!("Consuming swap payback note on first client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_payback_note_details[0].id()])?;
    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await?;

    // At the end we should end up with
    //
    // - accountA: 999 BTC, 25 ETH
    // - accountB: 1 BTC, 975 ETH

    // first reload the account
    let account_a: Account = client1
        .get_account(account_a.id())
        .await?
        .context("failed to find account A after private swap transaction")?
        .into();
    let account_a_assets = account_a.vault().assets();
    assert_eq!(account_a_assets.count(), 2);
    let mut account_a_assets = account_a.vault().assets();

    let asset_1 = account_a_assets
        .next()
        .context("expected first asset in account A's vault after private swap")?;
    let asset_2 = account_a_assets
        .next()
        .context("expected second asset in account A's vault after private swap")?;

    match (asset_1, asset_2) {
        (Asset::Fungible(btc_asset), Asset::Fungible(eth_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 999);
            assert_eq!(eth_asset.amount(), 25);
        },
        (Asset::Fungible(eth_asset), Asset::Fungible(btc_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 999);
            assert_eq!(eth_asset.amount(), 25);
        },
        _ => panic!("should only have fungible assets!"),
    }

    let account_b: Account = client2
        .get_account(account_b.id())
        .await?
        .context("failed to find account B after swap transaction")?
        .into();
    let account_b_assets = account_b.vault().assets();
    assert_eq!(account_b_assets.count(), 2);
    let mut account_b_assets = account_b.vault().assets();

    let asset_1 = account_b_assets
        .next()
        .context("expected first asset in account B's vault after swap")?;
    let asset_2 = account_b_assets
        .next()
        .context("expected second asset in account B's vault after swap")?;

    match (asset_1, asset_2) {
        (Asset::Fungible(btc_asset), Asset::Fungible(eth_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 1);
            assert_eq!(eth_asset.amount(), 975);
        },
        (Asset::Fungible(eth_asset), Asset::Fungible(btc_asset))
            if btc_asset.faucet_id() == btc_faucet_account.id()
                && eth_asset.faucet_id() == eth_faucet_account.id() =>
        {
            assert_eq!(btc_asset.amount(), 1);
            assert_eq!(eth_asset.amount(), 975);
        },
        _ => panic!("should only have fungible assets!"),
    }
    Ok(())
}
