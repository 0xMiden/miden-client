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
const reactSdkPath = path.join(repoRoot, "crates", "react-sdk", "package.json");

const readJson = (filePath) => JSON.parse(fs.readFileSync(filePath, "utf8"));

const writeJson = (filePath, data) => {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`);
};

const webClientPkg = readJson(webClientPath);
const reactSdkPkg = readJson(reactSdkPath);

const webClientVersion = webClientPkg.version;
const versionMatch = /^(\d+)\.(\d+)\.(\d+)(-.+)?$/.exec(webClientVersion);

if (!versionMatch) {
  console.error(`Unsupported web-client version format: "${webClientVersion}"`);
  process.exit(1);
}

const major = Number(versionMatch[1]);
const minor = Number(versionMatch[2]);
const hasPrerelease = Boolean(versionMatch[4]);

const expectedVersion = webClientVersion;
const expectedRange = hasPrerelease
  ? `^${major}.${minor}.0-0`
  : `^${major}.${minor}.0`;

const peerDeps = reactSdkPkg.peerDependencies || {};
const actualRange = peerDeps["@miden-sdk/miden-sdk"];
const actualVersion = reactSdkPkg.version;
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

    if (updated) {
      writeJson(reactSdkPath, reactSdkPkg);
      console.log(
        `Updated react-sdk version to "${expectedVersion}" and peer range to "${expectedRange}" based on web-client ${webClientVersion}.`
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
  `React SDK version and peer range match web-client ${webClientVersion} (${expectedRange}).`
);
