import type { Plugin } from "vite";
import path from "path";
import { createRequire } from "node:module";

export interface MidenVitePluginOptions {
  /** Packages to deduplicate. Default: ["@miden-sdk/miden-sdk"] */
  wasmPackages?: string[];
  /** Enable COOP/COEP headers on dev server. Default: true */
  crossOriginIsolation?: boolean;
  /** gRPC-web proxy target URL. Default: "https://rpc.testnet.miden.io". Set to false to disable. */
  rpcProxyTarget?: string | false;
  /** gRPC-web proxy path prefix. Default: "/rpc.Api" */
  rpcProxyPath?: string;
}

export function midenVitePlugin(options?: MidenVitePluginOptions): Plugin {
  const {
    wasmPackages = ["@miden-sdk/miden-sdk"],
    crossOriginIsolation = true,
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
        // Validate proxy target is a valid URL
        let targetUrl: URL;
        try {
          targetUrl = new URL(
            typeof rpcProxyTarget === "string" ? rpcProxyTarget : ""
          );
        } catch {
          throw new Error(`Invalid rpcProxyTarget URL: ${rpcProxyTarget}`);
        }
        if (targetUrl.protocol !== "https:") {
          console.warn(
            `[miden-vite-plugin] Warning: rpcProxyTarget uses ${targetUrl.protocol} instead of https:`
          );
        }
        serverConfig.proxy = {
          [rpcProxyPath]: {
            target: rpcProxyTarget,
            changeOrigin: true,
          },
        };
      }

      return {
        resolve: {
          alias,
          dedupe: [...wasmPackages],
          preserveSymlinks: true,
        },
        optimizeDeps: {
          exclude: [...wasmPackages],
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
