#!/bin/bash

if [ -z "$FILTER" ]; then
    FILTER="all()"
else
    FILTER="test($FILTER)"
fi


cargo nextest run --workspace --exclude miden-client-web --release --test=integration --filterset "$FILTER"

if [ -n "$FULL" ]; then
  cargo nextest run --workspace --exclude miden-client-web --release --test=integration --run-ignored ignored-only -- import_genesis_accounts_can_be_used_for_transactions
fi
