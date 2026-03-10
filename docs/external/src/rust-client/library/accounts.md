---
title: Accounts
sidebar_position: 1
---

# Retrieving Accounts

This guide demonstrates how to retrieve and inspect existing accounts using the Miden Client Rust library.

## Get a single account

```rust
let account_id = AccountId::from_hex("0x1234...")?;
let (account, _seed) = client.get_account(account_id).await?;

println!("Account ID: {:?}", account.id());
println!("Nonce: {:?}", account.nonce());
println!("Vault: {:?}", account.vault());
```

## List all accounts

```rust
let accounts = client.get_accounts().await?;

for (account, _seed) in &accounts {
    println!("Account: {:?}", account.id());
}
```

## Check account balance

After syncing, inspect an account's vault to see its assets:

```rust
let (account, _) = client.get_account(account_id).await?;

for asset in account.vault().assets() {
    match asset {
        Asset::Fungible(fungible) => {
            println!("Faucet: {:?}, Amount: {}", fungible.faucet_id(), fungible.amount());
        }
        Asset::NonFungible(nft) => {
            println!("NFT: {:?}", nft);
        }
    }
}
```

For importing and exporting accounts, see [Import](./import.md) and [Export](./export.md).
