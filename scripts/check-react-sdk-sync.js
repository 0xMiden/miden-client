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
const expectedVersion = webClientVersion;
const expectedRange = `^${major}.${minor}.0`;

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
}

if (actualRange !== expectedRange) {
  errors.push(
    `React SDK peer range "${actualRange}" does not match expected "${expectedRange}" for web-client ${webClientVersion}.`
  );
}

if (actualVersion !== expectedVersion) {
  errors.push(
    `React SDK version "${actualVersion}" does not match web-client version "${expectedVersion}".`
  );
}

if (!walletRange) {
  errors.push(
    "Missing dependencies entry for @miden-sdk/miden-sdk in wallet example."
  );
}

if (walletRange !== expectedRange) {
  errors.push(
    `Wallet example dependency "${walletRange}" does not match expected "${expectedRange}" for web-client ${webClientVersion}.`
  );
}

if (errors.length > 0) {
  if (shouldFix) {
    let updated = false;
    if (actualRange !== expectedRange) {
      peerDeps["@miden-sdk/miden-sdk"] = expectedRange;
      reactSdkPkg.peerDependencies = peerDeps;
      updated = true;
    }

    if (actualVersion !== expectedVersion) {
      reactSdkPkg.version = expectedVersion;
      updated = true;
    }

    if (walletRange !== expectedRange) {
      walletDeps["@miden-sdk/miden-sdk"] = expectedRange;
      walletExamplePkg.dependencies = walletDeps;
      updated = true;
    }

    if (updated) {
      writeJson(reactSdkPath, reactSdkPkg);
      writeJson(walletExamplePath, walletExamplePkg);
      console.log(
        `Updated react-sdk version to "${expectedVersion}", peer range to "${expectedRange}", and wallet dependency based on web-client ${webClientVersion}.`
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
  `React SDK version/peer range and wallet dependency match web-client ${webClientVersion} (${expectedRange}).`
);
