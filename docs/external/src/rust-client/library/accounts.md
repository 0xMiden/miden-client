---
title: Accounts
sidebar_position: 1
---

# Retrieving Accounts

This guide demonstrates how to retrieve and inspect existing accounts using the Miden Client Rust library.

## Get a single account

```rust
let account_id = AccountId::from_hex("0x1234...")?;
let account = client.get_account(account_id).await?;

if let Some(account) = account {
    println!("Account ID: {:?}", account.id());
    println!("Nonce: {:?}", account.nonce());
    println!("Vault: {:?}", account.vault());
}
```

## List all accounts

```rust
let account_headers = client.get_account_headers().await?;

for (header, status) in &account_headers {
    println!("Account: {:?}, Status: {:?}", header.id(), status);
}
```

## Check account balance

After syncing, use an `AccountReader` to check an account's balance, or retrieve the full account to inspect its vault:

```rust
// Using AccountReader (lightweight — fetches only what you need)
let reader = client.account_reader(account_id);
let balance = reader.get_balance(faucet_id).await?;
println!("Balance: {}", balance);

// Or retrieve the full account
let account = client.get_account(account_id).await?.expect("account exists");

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
