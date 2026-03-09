import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { midenVitePlugin } from "@miden-sdk/vite-plugin";
import { nodePolyfills } from "vite-plugin-node-polyfills";
import path from "path";

const optionalConnectorsPath = path.resolve(__dirname, "src", "optional-connectors.ts");

/**
 * Esbuild plugin that externalizes @miden-sdk/react during dep pre-bundling.
 * This prevents esbuild from inlining SignerContext (a React.createContext call)
 * into each pre-bundled dependency chunk. Without this, each dep gets its own
 * SignerContext instance, breaking React context identity matching.
 */
const externalizeMidenReact = {
  name: "externalize-miden-react",
  setup(build: any) {
    build.onResolve({ filter: /^@miden-sdk\/react$/ }, () => ({
      path: "@miden-sdk/react",
      external: true,
    }));
  },
};

export default defineConfig({
  plugins: [
    react(),
    midenVitePlugin({ crossOriginIsolation: false }),
    nodePolyfills({ include: ["buffer", "crypto", "stream", "util"] }),
  ],
  resolve: {
    alias: {
      // Use local source so unpublished exports (MultiSignerProvider, etc.) are available
      "@miden-sdk/react": path.resolve(__dirname, "../../src/index.ts"),
      // Use local rebuilt Turnkey (npm version bundles its own SignerContext)
      "@miden-sdk/miden-turnkey-react": path.resolve(__dirname, "../../../../../miden-turnkey/packages/use-miden-turnkey-react/dist/index.mjs"),
      // Stub optional Para connectors (not needed for Miden)
      "@getpara/solana-wallet-connectors": optionalConnectorsPath,
      "@getpara/cosmos-wallet-connectors": optionalConnectorsPath,
    },
    dedupe: ["react", "react-dom", "react/jsx-runtime", "@miden-sdk/react", "@getpara/web-sdk", "@getpara/react-sdk-lite"],
  },
  optimizeDeps: {
    esbuildOptions: {
      target: "esnext",
      plugins: [externalizeMidenReact],
    },
    exclude: [
      "@getpara/solana-wallet-connectors",
      "@getpara/cosmos-wallet-connectors",
    ],
  },
  build: {
    target: "esnext",
  },
  worker: {
    format: "es",
  },
});
