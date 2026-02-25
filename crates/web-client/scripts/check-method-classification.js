#!/usr/bin/env node

/**
 * CI lint script that verifies every async method on the WASM WebClient class
 * is explicitly classified in one of: SYNC_METHODS, WRITE_METHODS,
 * READ_METHODS, or as an explicit wrapper method on the JS WebClient class.
 *
 * This prevents new write methods from silently defaulting to read-only
 * (WASM-lock-only) wrapping in the Proxy fallback.
 */

import { access, readFile } from "node:fs/promises";
import path from "node:path";
import ts from "typescript";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const wasmTypesPath = path.join(
  rootDir,
  "dist",
  "crates",
  "miden_client_web.d.ts"
);
const indexJsPath = path.join(rootDir, "js", "index.js");

// Verify required files exist
const requiredFiles = [wasmTypesPath, indexJsPath];
const missingFiles = [];

for (const filePath of requiredFiles) {
  try {
    await access(filePath);
  } catch {
    missingFiles.push(filePath);
  }
}

if (missingFiles.length > 0) {
  console.error(
    "Method classification check failed because expected files are missing. Run `yarn build` first."
  );
  for (const filePath of missingFiles) {
    console.error(`- ${filePath}`);
  }
  process.exit(1);
}

// ---------------------------------------------------------------------------
// 1. Extract all method names from the WASM WebClient class in the .d.ts file
// ---------------------------------------------------------------------------

function extractWasmMethods(sourceText, filePath) {
  const sourceFile = ts.createSourceFile(
    filePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS
  );

  const methods = new Set();

  const visit = (node) => {
    // Find `export class WebClient`
    if (
      ts.isClassDeclaration(node) &&
      node.name &&
      node.name.text === "WebClient"
    ) {
      for (const member of node.members) {
        if (
          ts.isMethodDeclaration(member) &&
          member.name &&
          ts.isIdentifier(member.name)
        ) {
          methods.add(member.name.text);
        }
      }
    }
    ts.forEachChild(node, visit);
  };

  visit(sourceFile);
  return methods;
}

// ---------------------------------------------------------------------------
// 2. Extract classified sets and explicit methods from index.js
// ---------------------------------------------------------------------------

function extractClassifications(sourceText) {
  // Match Set declarations: const SOME_METHODS = new Set([ ... ]);
  const setPattern =
    /const\s+(SYNC_METHODS|WRITE_METHODS|READ_METHODS)\s*=\s*new\s+Set\s*\(\s*\[([\s\S]*?)\]\s*\)/g;
  const sets = {};

  let match;
  while ((match = setPattern.exec(sourceText)) !== null) {
    const setName = match[1];
    const body = match[2];
    // Extract quoted strings from the set body
    const entries = new Set();
    const stringPattern = /["']([^"']+)["']/g;
    let strMatch;
    while ((strMatch = stringPattern.exec(body)) !== null) {
      entries.add(strMatch[1]);
    }
    sets[setName] = entries;
  }

  return {
    syncMethods: sets.SYNC_METHODS || new Set(),
    writeMethods: sets.WRITE_METHODS || new Set(),
    readMethods: sets.READ_METHODS || new Set(),
  };
}

function extractExplicitMethods(sourceText, filePath) {
  // Use the TypeScript parser to reliably extract method names from
  // class declarations (WebClient, MockWebClient), avoiding false
  // positives from control-flow statements or Proxy traps.
  const sourceFile = ts.createSourceFile(
    filePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.JS
  );

  const methods = new Set();

  const visit = (node) => {
    if (
      ts.isClassDeclaration(node) &&
      node.name &&
      (node.name.text === "WebClient" || node.name.text === "MockWebClient")
    ) {
      for (const member of node.members) {
        if (
          ts.isMethodDeclaration(member) &&
          member.name &&
          ts.isIdentifier(member.name)
        ) {
          methods.add(member.name.text);
        }
      }
    }
    ts.forEachChild(node, visit);
  };

  visit(sourceFile);
  return methods;
}

// ---------------------------------------------------------------------------
// 3. Run the check
// ---------------------------------------------------------------------------

const wasmTypesSource = await readFile(wasmTypesPath, "utf8");
const indexJsSource = await readFile(indexJsPath, "utf8");

const wasmMethods = extractWasmMethods(wasmTypesSource, wasmTypesPath);
const { syncMethods, writeMethods, readMethods } =
  extractClassifications(indexJsSource);
const explicitMethods = extractExplicitMethods(indexJsSource, indexJsPath);

const classified = new Set([
  ...syncMethods,
  ...writeMethods,
  ...readMethods,
  ...explicitMethods,
]);

// Constructor, lifecycle, and internal methods that don't need classification
const allowedUnclassified = new Set([
  // wasm_bindgen infrastructure
  "new",
  "free",
  "serialize",
  "deserialize",
  // Factory / init methods handled by static wrappers
  "createClient",
  "createClientWithExternalKeystore",
  "createMockClient",
  // Internal impl method called directly by syncState wrappers
  "syncStateImpl",
]);

const unclassified = [...wasmMethods].filter(
  (name) => !classified.has(name) && !allowedUnclassified.has(name)
);

if (unclassified.length > 0) {
  console.error(
    "The following WASM methods are not classified in SYNC_METHODS, WRITE_METHODS, READ_METHODS, or as explicit wrapper methods in index.js:"
  );
  unclassified.sort().forEach((name) => console.error(`  - ${name}`));
  console.error(
    "\nAdd each method to the appropriate set in js/index.js, or add an explicit wrapper method on the WebClient class."
  );
  process.exit(1);
}

console.log(
  "Method classification check passed: all WASM WebClient methods are classified."
);
