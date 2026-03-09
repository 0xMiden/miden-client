---
title: Getting started
sidebar_position: 0
---

# Getting started

This section shows you how to get started with the Miden Web SDK by creating accounts, requesting funds from a public faucet, and transferring assets between accounts — all from the browser using TypeScript.

By the end of these tutorials, you will have:

- Initialized the Miden client in a web application
- Created an account and requested funds from the faucet
- Transferred assets between accounts using public and private notes
- Tested SDK workflows locally using the mock client

## Prerequisites

- [Node.js](https://nodejs.org/) v18 or later
- A WASM-capable browser (Chrome, Firefox, Safari, Edge)
- The SDK installed in your project — see [Installation](../install.md)

## Tutorials

1. [Create account and use faucet](./create-account-use-faucet.md) — create a wallet, request testnet tokens, and consume them
2. [Public peer-to-peer transfer](./p2p-public.md) — send tokens between accounts using public notes
3. [Private peer-to-peer transfer](./p2p-private.md) — send tokens privately with note transport
4. [Mock client](./mock-client.md) — test the full workflow locally without a network connection
