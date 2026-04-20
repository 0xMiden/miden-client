#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const webClientPath = path.join(
  repoRoot,
  "crates",
  "web-client",
  "package.json"
);
const reactSdkPath = path.join(
  repoRoot,
  "packages",
  "react-sdk",
  "package.json"
);
const walletExamplePath = path.join(
  repoRoot,
  "packages",
  "react-sdk",
  "examples",
  "wallet",
  "package.json"
);

const readJson = (filePath) => JSON.parse(fs.readFileSync(filePath, "utf8"));

const writeJson = (filePath, data) => {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`);
};

const webClientPkg = readJson(webClientPath);
const reactSdkPkg = readJson(reactSdkPath);
const walletExamplePkg = readJson(walletExamplePath);

const webClientVersion = webClientPkg.version;
const versionMatch = /^(\d+)\.(\d+)\.(\d+)(-.+)?$/.exec(webClientVersion);

if (!versionMatch) {
  console.error(`Unsupported web-client version format: "${webClientVersion}"`);
  process.exit(1);
}

const major = Number(versionMatch[1]);
const minor = Number(versionMatch[2]);
const patch = Number(versionMatch[3]);
const prerelease = versionMatch[4] || "";
// Default range used by --fix when a range needs to be created or repaired.
const defaultRange = prerelease
  ? `^${major}.${minor}.${patch}${prerelease}`
  : `^${major}.${minor}.0`;

// A range is compatible when it is a caret range within the same
// major.minor.prerelease as the web-client, with a patch no higher than the
// current web-client patch. This permits tightening the lower bound past .0
// (e.g. "^0.14.1" when web-client is 0.14.1) to exclude buggy prior patches,
// while still rejecting ranges that cross the minor boundary.
const parseCaretRange = (range) => {
  const m = /^\^(\d+)\.(\d+)\.(\d+)(-.+)?$/.exec(range || "");
  return m
    ? {
        major: Number(m[1]),
        minor: Number(m[2]),
        patch: Number(m[3]),
        prerelease: m[4] || "",
      }
    : null;
};

const isCompatibleRange = (range) => {
  const parsed = parseCaretRange(range);
  if (!parsed) return false;
  if (parsed.major !== major || parsed.minor !== minor) return false;
  if (parsed.prerelease !== prerelease) return false;
  return parsed.patch <= patch;
};

const compatibilityHint = prerelease
  ? `^${major}.${minor}.X${prerelease} with X <= ${patch}`
  : `^${major}.${minor}.X with X between 0 and ${patch}`;

const peerDeps = reactSdkPkg.peerDependencies || {};
const actualRange = peerDeps["@miden-sdk/miden-sdk"];
const actualVersion = reactSdkPkg.version;
const walletDeps = walletExamplePkg.dependencies || {};
const walletRange = walletDeps["@miden-sdk/miden-sdk"];
const shouldFix = process.argv.includes("--fix");
const errors = [];

if (!actualRange) {
  errors.push(
    "Missing peerDependencies entry for @miden-sdk/miden-sdk in react-sdk."
  );
} else if (!isCompatibleRange(actualRange)) {
  errors.push(
    `React SDK peer range "${actualRange}" is not compatible with web-client ${webClientVersion}. Expected ${compatibilityHint}.`
  );
}

const reactVersionMatch = /^(\d+)\.(\d+)\.(\d+)(-.+)?$/.exec(actualVersion);
if (!reactVersionMatch) {
  errors.push(`Unsupported react-sdk version format: "${actualVersion}"`);
} else if (
  Number(reactVersionMatch[1]) !== major ||
  Number(reactVersionMatch[2]) !== minor
) {
  errors.push(
    `React SDK version "${actualVersion}" has different major.minor than web-client "${webClientVersion}". They must share the same major.minor version.`
  );
}

if (!walletRange) {
  errors.push(
    "Missing dependencies entry for @miden-sdk/miden-sdk in wallet example."
  );
} else if (!isCompatibleRange(walletRange)) {
  errors.push(
    `Wallet example dependency "${walletRange}" is not compatible with web-client ${webClientVersion}. Expected ${compatibilityHint}.`
  );
}

if (errors.length > 0) {
  if (shouldFix) {
    let updated = false;
    if (!actualRange || !isCompatibleRange(actualRange)) {
      peerDeps["@miden-sdk/miden-sdk"] = defaultRange;
      reactSdkPkg.peerDependencies = peerDeps;
      updated = true;
    }

    if (!walletRange || !isCompatibleRange(walletRange)) {
      walletDeps["@miden-sdk/miden-sdk"] = defaultRange;
      walletExamplePkg.dependencies = walletDeps;
      updated = true;
    }

    if (updated) {
      writeJson(reactSdkPath, reactSdkPkg);
      writeJson(walletExamplePath, walletExamplePkg);
      console.log(
        `Updated react-sdk peer range and wallet dependency to "${defaultRange}" based on web-client ${webClientVersion}.`
      );
    }

    process.exit(0);
  }

  for (const message of errors) {
    console.error(message);
  }
  process.exit(1);
}

console.log(
  `React SDK peer range "${actualRange}" and wallet dependency "${walletRange}" are compatible with web-client ${webClientVersion}.`
);
