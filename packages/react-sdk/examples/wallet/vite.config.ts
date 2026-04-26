import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";
import { paraVitePlugin } from "@miden-sdk/use-miden-para-react/vite";
import { nodePolyfills } from "vite-plugin-node-polyfills";
import path from "path";

// Optional Para chain/integration modules — dynamically imported by
// @getpara/react-core and @getpara/react-sdk-lite. We use only Para's
// core EVM signer, so externalize all the AA / chain-specific / wallet-
// connector packages so Rollup doesn't fail trying to resolve them.
// Anything matching this regex is treated as external by both dev and
// build; runtime dynamic imports for these will throw when actually
// invoked, but our App.tsx never reaches those code paths.
const PARA_OPTIONAL_REGEX = /^@getpara\/(aa-|cosm|ethers-|evm-wallet|solana-|stellar-|viem-v2-|core-sdk|shared|react-common)/;
const PARA_OPTIONAL = [
  "@getpara/aa-alchemy",
  "@getpara/aa-biconomy",
  "@getpara/aa-cdp",
  "@getpara/aa-gelato",
  "@getpara/aa-pimlico",
  "@getpara/aa-porto",
  "@getpara/aa-rhinestone",
  "@getpara/aa-safe",
  "@getpara/aa-thirdweb",
  "@getpara/aa-zerodev",
  "@getpara/core-sdk",
  "@getpara/cosmjs-v0-integration",
  "@getpara/cosmos-wallet-connectors",
  "@getpara/ethers-v6-integration",
  "@getpara/evm-wallet-connectors",
  "@getpara/react-common",
  "@getpara/shared",
  "@getpara/solana-signers-v2-integration",
  "@getpara/solana-wallet-connectors",
  "@getpara/stellar-sdk-v14-integration",
  "@getpara/viem-v2-integration",
];

export default defineConfig({
  plugins: [
    react(),
    midenVitePlugin(),
    paraVitePlugin(),
    // Local @miden-sdk/miden-sdk worker bundle imports
    // `vite-plugin-node-polyfills/shims/global` directly, so the polyfills
    // plugin must be active in the consumer.
    nodePolyfills({ globals: { Buffer: true, global: true, process: true } }),
  ],
  resolve: {
    alias: {
      // Use local source for react-sdk development
      "@miden-sdk/react": path.resolve(__dirname, "../../src/index.ts"),
    },
  },
  optimizeDeps: {
    exclude: PARA_OPTIONAL,
  },
  build: {
    rollupOptions: {
      // Match by regex so we don't have to enumerate every optional
      // sub-path Para's react-core dynamically imports (chain-specific
      // signer files etc).
      external: (id) => PARA_OPTIONAL_REGEX.test(id),
    },
  },
});
