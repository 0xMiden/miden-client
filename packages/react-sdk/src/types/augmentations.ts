import "@miden-sdk/miden-sdk";

declare module "@miden-sdk/miden-sdk" {
  interface Account {
    /** Returns the bech32-encoded account id using the configured network. */
    bech32id(): string;
  }
}

export {};
