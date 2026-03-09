import type { Plugin } from "vite";
import path from "path";
import { createRequire } from "node:module";

export interface MidenVitePluginOptions {
  /** Packages to deduplicate. Default: ["@miden-sdk/miden-sdk"] */
  wasmPackages?: string[];
  /**
   * Enable COOP/COEP headers on dev server for SharedArrayBuffer support.
   * Default: false — enabling this breaks OAuth popup flows (e.g. Para)
   * because `same-origin` COOP nullifies `window.opener` in popups.
   */
  crossOriginIsolation?: boolean;
  /** gRPC-web proxy target URL. Default: "https://rpc.testnet.miden.io". Set to false to disable. */
  rpcProxyTarget?: string | false;
  /** gRPC-web proxy path prefix. Default: "/rpc.Api" */
  rpcProxyPath?: string;
}

/**
 * Esbuild plugin that externalizes @miden-sdk/react during Vite's dep pre-bundling.
 * Without this, esbuild inlines a separate copy of the module (and its
 * React.createContext calls) into each pre-bundled dependency chunk, breaking
 * React context identity matching across signer providers.
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

export function midenVitePlugin(options?: MidenVitePluginOptions): Plugin {
  const {
    wasmPackages = ["@miden-sdk/miden-sdk"],
    crossOriginIsolation = false,
    rpcProxyTarget = "https://rpc.testnet.miden.io",
    rpcProxyPath = "/rpc.Api",
  } = options ?? {};

  return {
    name: "@miden-sdk/vite-plugin",
    enforce: "pre",

    config(userConfig, env) {
      const root = userConfig.root ?? process.cwd();

      // Use array form for resolve.alias so Vite appends rather than replaces
      // any existing aliases the user may have configured.
      // Use require.resolve for portable resolution in pnpm/Yarn Plug'n'Play setups.
      const esmRequire = createRequire(`file://${root}/`);
      const alias = wasmPackages.map((pkg) => {
        let replacement: string;
        try {
          replacement = path.dirname(esmRequire.resolve(`${pkg}/package.json`));
        } catch {
          replacement = path.resolve(root, "node_modules", pkg);
        }
        return { find: pkg, replacement };
      });

      const serverConfig: Record<string, unknown> = {};
      const previewConfig: Record<string, unknown> = {};

      if (crossOriginIsolation) {
        const coopCoepHeaders = {
          "Cross-Origin-Opener-Policy": "same-origin",
          "Cross-Origin-Embedder-Policy": "require-corp",
        };
        serverConfig.headers = coopCoepHeaders;
        previewConfig.headers = coopCoepHeaders;
      }

      if (rpcProxyTarget !== false && env.command === "serve") {
        serverConfig.proxy = {
          [rpcProxyPath]: {
            target: rpcProxyTarget,
            changeOrigin: true,
          },
        };
      }

      // Merge with any existing esbuild plugins the user may have configured
      const existingPlugins =
        userConfig.optimizeDeps?.esbuildOptions?.plugins ?? [];

      return {
        resolve: {
          alias,
          dedupe: [
            ...wasmPackages,
            "react",
            "react-dom",
            "react/jsx-runtime",
            "@miden-sdk/react",
          ],
          preserveSymlinks: true,
        },
        optimizeDeps: {
          exclude: [...wasmPackages],
          esbuildOptions: {
            target: "esnext",
            plugins: [...existingPlugins, externalizeMidenReact],
          },
        },
        build: {
          target: "esnext",
        },
        worker: {
          format: "es" as const,
          rollupOptions: { output: { format: "es" as const } },
        },
        server: serverConfig,
        preview: previewConfig,
      };
    },
  };
}

export default midenVitePlugin;
