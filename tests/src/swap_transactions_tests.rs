use miden_client::{
    account::Account,
    note::{Note, build_swap_tag},
    testing::common::*,
    transaction::{SwapTransactionData, TransactionRequestBuilder},
};
use miden_objects::{
    account::AccountStorageMode,
    asset::{Asset, FungibleAsset},
    note::{NoteDetails, NoteFile, NoteType},
};

use std::fs;
use miden_objects::utils::Deserializable as _;

// SWAP FULLY ONCHAIN
// ================================================================================================

#[tokio::test]
async fn swap_fully_onchain() {
    const OFFERED_ASSET_AMOUNT: u64 = 1;
    const REQUESTED_ASSET_AMOUNT: u64 = 25;
    let (mut client1, authenticator_1) = create_test_client().await;
    wait_for_node(&mut client1).await;
    let (mut client2, authenticator_2) = create_test_client().await;

    client1.sync_state().await.unwrap();
    client2.sync_state().await.unwrap();

    // Import pre-funded accounts from genesis
    let btc_faucet_data = AccountFile::read_from_bytes(&fs::read("data/faucet_btc.mac").expect("file exists")).expect("valid file");
    let btc_faucet_id = btc_faucet_data.account.id();
    client1.add_account(&btc_faucet_data.account, btc_faucet_data.account_seed, false).await.unwrap();
    for key in btc_faucet_data.auth_secret_keys {
        authenticator_1.add_key(&key).unwrap();
    }
    let btc_faucet_account: Account = client1.get_account(btc_faucet_id).await.unwrap().unwrap().into();

    let eth_faucet_data = AccountFile::read_from_bytes(&fs::read("data/faucet_eth.mac").expect("file exists")).expect("valid file");
    let eth_faucet_id = eth_faucet_data.account.id();
    client2.add_account(&eth_faucet_data.account, eth_faucet_data.account_seed, false).await.unwrap();
    for key in eth_faucet_data.auth_secret_keys {
        authenticator_2.add_key(&key).unwrap();
    }
    let eth_faucet_account: Account = client2.get_account(eth_faucet_id).await.unwrap().unwrap().into();

    let account_a_data = AccountFile::read_from_bytes(&fs::read("data/wallet_0.mac").expect("file exists")).expect("valid file");
    let account_a_id = account_a_data.account.id();
    client1.add_account(&account_a_data.account, account_a_data.account_seed, false).await.unwrap();
    for key in account_a_data.auth_secret_keys {
        authenticator_1.add_key(&key).unwrap();
    }
    let account_a: Account = client1.get_account(account_a_id).await.unwrap().unwrap().into();

    let account_b_data = AccountFile::read_from_bytes(&fs::read("data/wallet_1.mac").expect("file exists")).expect("valid file");
    let account_b_id = account_b_data.account.id();
    client2.add_account(&account_b_data.account, account_b_data.account_seed, false).await.unwrap();
    for key in account_b_data.auth_secret_keys {
        authenticator_2.add_key(&key).unwrap();
    }
    let account_b: Account = client2.get_account(account_b_id).await.unwrap().unwrap().into();

    // Create ONCHAIN swap note (clientA offers 1 BTC in exchange of 25 ETH)
    // check that account now has 1 less BTC
    println!("creating swap note with accountA");
    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_ASSET_AMOUNT).unwrap();
    let requested_asset =
        FungibleAsset::new(eth_faucet_account.id(), REQUESTED_ASSET_AMOUNT).unwrap();

    println!("Running SWAP tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_swap(
            &SwapTransactionData::new(
                account_a.id(),
                Asset::Fungible(offered_asset),
                Asset::Fungible(requested_asset),
            ),
            NoteType::Public,
            client1.rng(),
        )
        .unwrap();

    let expected_output_notes: Vec<Note> = tx_request.expected_output_own_notes();
    let expected_payback_note_details: Vec<NoteDetails> =
        tx_request.expected_future_notes().cloned().map(|(n, _)| n).collect();
    assert_eq!(expected_output_notes.len(), 1);
    assert_eq!(expected_payback_note_details.len(), 1);

    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await;

    let swap_note_tag = build_swap_tag(
        NoteType::Public,
        &Asset::Fungible(offered_asset),
        &Asset::Fungible(requested_asset),
    )
    .unwrap();

    // add swap note's tag to client2
    // we could technically avoid this step, but for the first iteration of swap notes we'll
    // require to manually add tags
    println!("Adding swap tag");
    client2.add_note_tag(swap_note_tag).await.unwrap();

    // sync on client 2, we should get the swap note
    // consume swap note with accountB, and check that the vault changed appropriately
    client2.sync_state().await.unwrap();
    println!("Consuming swap note on second client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_output_notes[0].id()])
        .unwrap();
    execute_tx_and_sync(&mut client2, account_b.id(), tx_request).await;

    // sync on client 1, we should get the missing payback note details.
    // try consuming the received note with accountA, it should now have 25 ETH
    client1.sync_state().await.unwrap();
    println!("Consuming swap payback note on first client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_payback_note_details[0].id()])
        .unwrap();
    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await;

    // At the end we should end up with
    //
    // - accountA: 999 BTC, 25 ETH
    // - accountB: 1 BTC, 975 ETH

    // first reload the account
    let account_a: Account = client1.get_account(account_a.id()).await.unwrap().unwrap().into();
    let account_a_assets = account_a.vault().assets();
    assert_eq!(account_a_assets.count(), 2);
    let mut account_a_assets = account_a.vault().assets();

    let asset_1 = account_a_assets.next().unwrap();
    let asset_2 = account_a_assets.next().unwrap();

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

    let account_b: Account = client2.get_account(account_b.id()).await.unwrap().unwrap().into();
    let account_b_assets = account_b.vault().assets();
    assert_eq!(account_b_assets.count(), 2);
    let mut account_b_assets = account_b.vault().assets();

    let asset_1 = account_b_assets.next().unwrap();
    let asset_2 = account_b_assets.next().unwrap();

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
}

#[tokio::test]
async fn swap_private() {
    const OFFERED_ASSET_AMOUNT: u64 = 1;
    const REQUESTED_ASSET_AMOUNT: u64 = 25;
    let (mut client1, authenticator_1) = create_test_client().await;
    wait_for_node(&mut client1).await;
    let (mut client2, authenticator_2) = create_test_client().await;

    client1.sync_state().await.unwrap();
    client2.sync_state().await.unwrap();

    // Import pre-funded accounts from genesis
    let btc_faucet_data = AccountFile::read_from_bytes(&fs::read("data/faucet_btc.mac").expect("file exists")).expect("valid file");
    let btc_faucet_id = btc_faucet_data.account.id();
    client1.add_account(&btc_faucet_data.account, btc_faucet_data.account_seed, false).await.unwrap();
    for key in btc_faucet_data.auth_secret_keys {
        authenticator_1.add_key(&key).unwrap();
    }
    let btc_faucet_account: Account = client1.get_account(btc_faucet_id).await.unwrap().unwrap().into();

    let eth_faucet_data = AccountFile::read_from_bytes(&fs::read("data/faucet_eth.mac").expect("file exists")).expect("valid file");
    let eth_faucet_id = eth_faucet_data.account.id();
    client2.add_account(&eth_faucet_data.account, eth_faucet_data.account_seed, false).await.unwrap();
    for key in eth_faucet_data.auth_secret_keys {
        authenticator_2.add_key(&key).unwrap();
    }
    let eth_faucet_account: Account = client2.get_account(eth_faucet_id).await.unwrap().unwrap().into();

    let account_a_data = AccountFile::read_from_bytes(&fs::read("data/wallet_0.mac").expect("file exists")).expect("valid file");
    let account_a_id = account_a_data.account.id();
    client1.add_account(&account_a_data.account, account_a_data.account_seed, false).await.unwrap();
    for key in account_a_data.auth_secret_keys {
        authenticator_1.add_key(&key).unwrap();
    }
    let account_a: Account = client1.get_account(account_a_id).await.unwrap().unwrap().into();

    let account_b_data = AccountFile::read_from_bytes(&fs::read("data/wallet_1.mac").expect("file exists")).expect("valid file");
    let account_b_id = account_b_data.account.id();
    client2.add_account(&account_b_data.account, account_b_data.account_seed, false).await.unwrap();
    for key in account_b_data.auth_secret_keys {
        authenticator_2.add_key(&key).unwrap();
    }
    let account_b: Account = client2.get_account(account_b_id).await.unwrap().unwrap().into();

    // Create ONCHAIN swap note (clientA offers 1 BTC in exchange of 25 ETH)
    // check that account now has 1 less BTC
    println!("creating swap note with accountA");
    let offered_asset = FungibleAsset::new(btc_faucet_account.id(), OFFERED_ASSET_AMOUNT).unwrap();
    let requested_asset =
        FungibleAsset::new(eth_faucet_account.id(), REQUESTED_ASSET_AMOUNT).unwrap();

    println!("Running SWAP tx...");
    let tx_request = TransactionRequestBuilder::new()
        .build_swap(
            &SwapTransactionData::new(
                account_a.id(),
                Asset::Fungible(offered_asset),
                Asset::Fungible(requested_asset),
            ),
            NoteType::Private,
            client1.rng(),
        )
        .unwrap();

    let expected_output_notes: Vec<Note> = tx_request.expected_output_own_notes();
    let expected_payback_note_details =
        tx_request.expected_future_notes().cloned().map(|(n, _)| n).collect::<Vec<_>>();
    assert_eq!(expected_output_notes.len(), 1);
    assert_eq!(expected_payback_note_details.len(), 1);

    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await;

    // Export note from client 1 to client 2
    let output_note =
        client1.get_output_note(expected_output_notes[0].id()).await.unwrap().unwrap();

    let tag = build_swap_tag(
        NoteType::Private,
        &Asset::Fungible(offered_asset),
        &Asset::Fungible(requested_asset),
    )
    .unwrap();
    client2.add_note_tag(tag).await.unwrap();
    client2
        .import_note(NoteFile::NoteDetails {
            details: output_note.try_into().unwrap(),
            after_block_num: client1.get_sync_height().await.unwrap(),
            tag: Some(tag),
        })
        .await
        .unwrap();

    // Sync so we get the inclusion proof info
    client2.sync_state().await.unwrap();

    // consume swap note with accountB, and check that the vault changed appropriately
    println!("Consuming swap note on second client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_output_notes[0].id()])
        .unwrap();
    execute_tx_and_sync(&mut client2, account_b.id(), tx_request).await;

    // sync on client 1, we should get the missing payback note details.
    // try consuming the received note with accountA, it should now have 25 ETH
    client1.sync_state().await.unwrap();
    println!("Consuming swap payback note on first client...");

    let tx_request = TransactionRequestBuilder::new()
        .build_consume_notes(vec![expected_payback_note_details[0].id()])
        .unwrap();
    execute_tx_and_sync(&mut client1, account_a.id(), tx_request).await;

    // At the end we should end up with
    //
    // - accountA: 999 BTC, 25 ETH
    // - accountB: 1 BTC, 975 ETH

    // first reload the account
    let account_a: Account = client1.get_account(account_a.id()).await.unwrap().unwrap().into();
    let account_a_assets = account_a.vault().assets();
    assert_eq!(account_a_assets.count(), 2);
    let mut account_a_assets = account_a.vault().assets();

    let asset_1 = account_a_assets.next().unwrap();
    let asset_2 = account_a_assets.next().unwrap();

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

    let account_b: Account = client2.get_account(account_b.id()).await.unwrap().unwrap().into();
    let account_b_assets = account_b.vault().assets();
    assert_eq!(account_b_assets.count(), 2);
    let mut account_b_assets = account_b.vault().assets();

    let asset_1 = account_b_assets.next().unwrap();
    let asset_2 = account_b_assets.next().unwrap();

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
}
