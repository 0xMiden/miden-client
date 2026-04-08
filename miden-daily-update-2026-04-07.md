# Miden Daily Update: April 7th, 2026

## Fabrizio

### April 7th

* Merged PR fixing CI build for remote prover. [#1969](https://github.com/0xMiden/miden-client/pull/1969)
* Merged PR fixing remote prover test. [#1962](https://github.com/0xMiden/miden-client/pull/1962)

## Ignacio

### April 7th

* Client:
    * Opened PR migrating to protocol 0.14.3. [#1972](https://github.com/0xMiden/miden-client/pull/1972)
    * Merged PR making NTL client lazy. [#1970](https://github.com/0xMiden/miden-client/pull/1970)
    * Reviewed PRs:
        * fix: CI build remote prover. [#1969](https://github.com/0xMiden/miden-client/pull/1969)
        * feat(WebClient): add network-aware factory methods createTestnet/createDevnet. [#1964](https://github.com/0xMiden/miden-client/pull/1964)
        * feat: avoid fetch chain tip header on sync. [#1963](https://github.com/0xMiden/miden-client/pull/1963)
        * fix: remote prover test. [#1962](https://github.com/0xMiden/miden-client/pull/1962)
        * feat: add vault retrieval to get_account_proof. [#1960](https://github.com/0xMiden/miden-client/pull/1960)
        * feat(web-client): add StorageView JS wrapper. [#1955](https://github.com/0xMiden/miden-client/pull/1955)
        * refactor: retrieve deltas from large public accounts. [#1916](https://github.com/0xMiden/miden-client/pull/1916)
* Node:
    * Merged PR fixing error on missing notes. [#1899](https://github.com/0xMiden/node/pull/1899)
    * Merged PR adding inclusion proofs to `SyncTransactions` output notes. [#1893](https://github.com/0xMiden/node/pull/1893)
    * Merged PR including block header in `SyncChainMmr`. [#1881](https://github.com/0xMiden/node/pull/1881)
* Protocol:
    * Reviewed Prepare v0.14.3 release. [#2744](https://github.com/0xMiden/protocol/pull/2744)

## Jeremias

### April 7th

* Merged PR adding vault retrieval to get_account_proof. [#1960](https://github.com/0xMiden/miden-client/pull/1960)
* Merged PR adding network-aware factory methods createTestnet/createDevnet to WebClient. [#1964](https://github.com/0xMiden/miden-client/pull/1964)
* Reviewed PR refactoring delta retrieval from large public accounts. [#1916](https://github.com/0xMiden/miden-client/pull/1916)

## Juan

### April 7th

* Reviewed PRs:
    * chore: migrate to protocol 0.14.3. [#1972](https://github.com/0xMiden/miden-client/pull/1972)
    * refactor: make NTL client lazy. [#1970](https://github.com/0xMiden/miden-client/pull/1970)

## Santiago

### April 7th

* Node:
    * Opened PR adding Validator card to network monitor dashboard. [#1900](https://github.com/0xMiden/node/pull/1900)
* Reviewed PRs:
    * Client:
        * chore: migrate to protocol 0.14.3. [#1972](https://github.com/0xMiden/miden-client/pull/1972)
        * refactor: make NTL client lazy. [#1970](https://github.com/0xMiden/miden-client/pull/1970)
        * refactor: retrieve deltas from large public accounts. [#1916](https://github.com/0xMiden/miden-client/pull/1916)
    * Node:
        * fix: Fix transact spans. [#1897](https://github.com/0xMiden/node/pull/1897)
        * feat: include block header in `SyncChainMmr`. [#1881](https://github.com/0xMiden/node/pull/1881)
    * Faucet:
        * feat: store API Keys in the store. [#225](https://github.com/0xMiden/faucet/pull/225)

## Tomas

### April 7th

* Opened PR tracking consumer on notes consumed externally. [#1973](https://github.com/0xMiden/miden-client/pull/1973)
* Merged PR avoiding fetch of chain tip header on sync. [#1963](https://github.com/0xMiden/miden-client/pull/1963)
* Reviewed PRs:
    * Client:
        * chore: migrate to protocol 0.14.3. [#1972](https://github.com/0xMiden/miden-client/pull/1972)
        * refactor: make NTL client lazy. [#1970](https://github.com/0xMiden/miden-client/pull/1970)
        * feat: agglayer integration tests. [#1967](https://github.com/0xMiden/miden-client/pull/1967)
        * refactor: retrieve deltas from large public accounts. [#1916](https://github.com/0xMiden/miden-client/pull/1916)
    * Node:
        * feat: include block header in `SyncChainMmr`. [#1881](https://github.com/0xMiden/node/pull/1881)
