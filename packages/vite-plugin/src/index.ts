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
  /**
   * Inject `vite-plugin-node-polyfills` into the worker bundle so the SDK's
   * worker chunk (which imports `vite-plugin-node-polyfills/shims/global`
   * directly) resolves cleanly. Default: true. Set false if your app already
   * configures `worker.plugins` with nodePolyfills, or if you don't want
   * polyfills in workers at all.
   *
   * Requires `vite-plugin-node-polyfills` as a dev dependency in the
   * consuming app (lazy-required at config time; missing → warning + skip).
   */
  injectWorkerPolyfills?: boolean;
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
    injectWorkerPolyfills = true,
  } = options ?? {};

  // Lazy-require nodePolyfills from the consuming project so missing the dep
  // is a warning + skip rather than a hard ESM import failure here. The SDK's
  // worker bundle imports `vite-plugin-node-polyfills/shims/global` directly,
  // so the worker context needs the plugin even if the host app doesn't use
  // node polyfills elsewhere.
  let workerPolyfillsFactory: (() => Plugin | Plugin[]) | null = null;
  if (injectWorkerPolyfills) {
    try {
      const projectRequire = createRequire(`file://${process.cwd()}/`);
      const polyfillsPkg = projectRequire("vite-plugin-node-polyfills");
      workerPolyfillsFactory = () =>
        polyfillsPkg.nodePolyfills({
          globals: { Buffer: true, global: true, process: true },
        });
    } catch {
      console.warn(
        "[@miden-sdk/vite-plugin] vite-plugin-node-polyfills not found; the " +
          "SDK worker chunk needs it to resolve `shims/global`. Install " +
          "with: npm install -D vite-plugin-node-polyfills"
      );
    }
  }

  const requiredDedupe = [
    "react",
    "react-dom",
    "react/jsx-runtime",
    "@miden-sdk/react",
  ];

  return {
    name: "@miden-sdk/vite-plugin",
    enforce: "pre",

    config(userConfig, env) {
      const root = userConfig.root ?? process.cwd();

      // Use array form for resolve.alias so Vite appends rather than replaces
      // any existing aliases the user may have configured.
      // Use require.resolve for portable resolution in pnpm/Yarn Plug'n'Play setups.
      const esmRequire = createRequire(`file://${root}/`);
      // Exact-match regex alias (not prefix match). Vite's default string-form
      // alias matches prefixes, which rewrites subpath imports (e.g.
      // `@miden-sdk/miden-sdk/lazy`) to file-path lookups and bypasses the
      // package's `exports` map. Using `^<pkg>$` keeps the alias scoped to the
      // root specifier; subpath imports fall through to Vite's standard ESM
      // resolution, which honors `exports`. Package-level deduplication is
      // handled separately via `resolve.dedupe` (see below).
      const escapeRegExp = (s: string) =>
        s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const alias = wasmPackages.map((pkg) => {
        let replacement: string;
        try {
          replacement = path.dirname(esmRequire.resolve(`${pkg}/package.json`));
        } catch {
          replacement = path.resolve(root, "node_modules", pkg);
        }
        return { find: new RegExp(`^${escapeRegExp(pkg)}$`), replacement };
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

      const workerConfig: Record<string, unknown> = {
        format: "es" as const,
        rollupOptions: { output: { format: "es" as const } },
      };
      if (workerPolyfillsFactory) {
        // `worker.plugins` must be a function (Vite calls it per worker
        // bundle) — wrap so each worker gets a fresh plugin instance.
        workerConfig.plugins = () => {
          const out = workerPolyfillsFactory!();
          return Array.isArray(out) ? out : [out];
        };
      }

      return {
        resolve: {
          alias,
          dedupe: [...wasmPackages, ...requiredDedupe],
          preserveSymlinks: true,
        },
        optimizeDeps: {
          exclude: [...wasmPackages],
        },
        build: {
          target: "esnext",
        },
        worker: workerConfig,
        server: serverConfig,
        preview: previewConfig,
      };
    },

    // Use configResolved to inject the esbuild externalization plugin and
    // dedupe entries into the final resolved config. This runs AFTER all
    // plugins' config() hooks have been merged, so other plugins (e.g.
    // vite-plugin-node-polyfills) can't overwrite these entries.
    configResolved(config) {
      // Ensure esbuild externalization plugin is present
      if (!config.optimizeDeps.esbuildOptions) {
        config.optimizeDeps.esbuildOptions = {};
      }
      const esbuildOpts = config.optimizeDeps.esbuildOptions;
      if (!esbuildOpts.plugins) {
        esbuildOpts.plugins = [];
      }
      const hasPlugin = esbuildOpts.plugins.some(
        (p: any) => p.name === "externalize-miden-react"
      );
      if (!hasPlugin) {
        esbuildOpts.plugins.push(externalizeMidenReact);
      }

      // Ensure esnext target for top-level await in WASM
      if (!esbuildOpts.target) {
        esbuildOpts.target = "esnext";
      }

      // Ensure required dedupe entries are present
      if (!config.resolve.dedupe) {
        (config.resolve as any).dedupe = [];
      }
      for (const dep of requiredDedupe) {
        if (!config.resolve.dedupe.includes(dep)) {
          config.resolve.dedupe.push(dep);
        }
      }
    },
  };
}

export default midenVitePlugin;
