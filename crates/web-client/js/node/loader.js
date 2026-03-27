/**
 * Finds and loads the napi native module (.node binary).
 *
 * Search order:
 * 1. MIDEN_MODULE_PATH environment variable (explicit override)
 * 2. Platform-specific npm package (@miden-sdk/node-darwin-arm64, etc.)
 * 3. Package prebuilds directory
 * 4. Repo target directory (for local development)
 */
import { createRequire } from "module";
import path from "path";
import fs from "fs";
import os from "os";

const require = createRequire(import.meta.url);

let _sdk = null;

/**
 * Loads the napi SDK module. Caches the result after first load.
 *
 * @param {object} [options]
 * @param {string} [options.modulePath] - Explicit path to the .node file.
 * @returns {object} The napi SDK module.
 */
export function loadNativeModule(options) {
  if (_sdk) return _sdk;

  // 1. Explicit path (option or env var)
  const explicit = options?.modulePath || process.env.MIDEN_MODULE_PATH;
  if (explicit) {
    _sdk = require(explicit);
    return _sdk;
  }

  // 2. Platform-specific npm package (installed via optionalDependencies)
  const platformPackage = getPlatformPackageName();
  if (platformPackage) {
    try {
      _sdk = require(platformPackage);
      return _sdk;
    } catch {
      // Not installed -- fall through to other methods
    }
  }

  const archMap = { arm64: "aarch64", x64: "x86_64" };
  const arch = archMap[os.arch()] || os.arch();
  const platform =
    os.platform() === "darwin" ? "apple-darwin" : "unknown-linux-gnu";
  const target = `${arch}-${platform}`;
  const ext = os.platform() === "darwin" ? "dylib" : "so";
  const libName = `libmiden_client_web.${ext}`;

  // 3. Package prebuilds directory
  const packageRoot = path.resolve(import.meta.dirname, "..");
  const prebuildCandidates = [
    path.join(
      packageRoot,
      "prebuilds",
      `${os.platform()}-${os.arch()}`,
      "miden_client_web.node"
    ),
    path.join(packageRoot, "prebuilds", "miden_client_web.node"),
  ];

  for (const p of prebuildCandidates) {
    if (fs.existsSync(p)) {
      _sdk = require(p);
      return _sdk;
    }
  }

  // 4. Repo target directory (development)
  const repoRoot = findRepoRoot(packageRoot);
  if (repoRoot) {
    const targetCandidates = [
      path.join(repoRoot, "target", target, "release", libName),
      path.join(repoRoot, "target", "release", libName),
      path.join(repoRoot, "target", target, "debug", libName),
      path.join(repoRoot, "target", "debug", libName),
    ];

    for (const p of targetCandidates) {
      if (fs.existsSync(p)) {
        // napi requires a .node extension -- copy if needed
        const nodeFile = path.join(path.dirname(p), "miden_client_web.node");
        if (
          !fs.existsSync(nodeFile) ||
          fs.statSync(p).mtimeMs > fs.statSync(nodeFile).mtimeMs
        ) {
          fs.copyFileSync(p, nodeFile);
        }
        _sdk = require(nodeFile);
        return _sdk;
      }
    }
  }

  throw new Error(
    `Miden napi module not found.\n\n` +
      `Build it with:\n` +
      `  cargo build -p miden-client-web --no-default-features --features nodejs --release\n\n` +
      `Or set MIDEN_MODULE_PATH to the .node file location.`
  );
}

/**
 * Returns the platform-specific npm package name for the current OS/arch,
 * or null if the platform is not supported.
 */
function getPlatformPackageName() {
  const platformMap = {
    "darwin-arm64": "@miden-sdk/node-darwin-arm64",
    "darwin-x64": "@miden-sdk/node-darwin-x64",
    "linux-x64": "@miden-sdk/node-linux-x64-gnu",
  };
  return platformMap[`${os.platform()}-${os.arch()}`] || null;
}

/**
 * Walks up from startDir looking for the repo root (has Cargo.toml + crates/).
 */
function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 10; i++) {
    if (
      fs.existsSync(path.join(dir, "Cargo.toml")) &&
      fs.existsSync(path.join(dir, "crates"))
    ) {
      return dir;
    }
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}
