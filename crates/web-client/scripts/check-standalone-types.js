#!/usr/bin/env node
//
// Verifies that every forwarder-style wrapper in `js/standalone.js` has a
// declared return type in `dist/api-types.d.ts` that matches the return type
// of the underlying wasm-bindgen method it forwards to.
//
// This catches the class of bug behind #2042: the hand-written TypeScript
// declaration drifted from the actual runtime behavior (the implementation
// just returns what the wasm method returns).
//
// A "forwarder" here means a function whose last statement is
// `return <wasmRef>.<method>(...)` for a recognised wasmRef — either `wasm`
// (the full wasm module, where the access is `wasm.<Class>.<method>`) or
// `_WebClient` (a module-local alias for the WebClient class).
//
// Non-forwarder wrappers (doing real logic, returning constructed objects)
// are skipped with a warning so a reviewer can decide whether to spot-check
// them by hand.

import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const standalonePath = path.join(rootDir, "js", "standalone.js");
const apiTypesPath = path.join(rootDir, "dist", "api-types.d.ts");
const wasmTypesPath = path.join(
  rootDir,
  "dist",
  "crates",
  "miden_client_web.d.ts"
);

// Maps a wasm reference identifier used inside standalone.js to the
// bindgen-level class whose static methods it exposes. `wasm` is the
// full module (so `wasm.Note.x` → class `Note`, method `x`). `_WebClient`
// is wired via `_setWebClient(WebClientClass)` in index.js and always
// points at the `WebClient` class.
const WASM_REF_TO_CLASS = {
  _WebClient: "WebClient",
};

function parseSourceFile(filePath, sourceText, scriptKind) {
  return ts.createSourceFile(
    filePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    scriptKind
  );
}

// Walks an exported function's body and returns { cls, method } if the final
// `return` forwards to a recognised wasm reference. Returns null otherwise.
function extractForwarderTarget(functionNode, sourceFile) {
  let target = null;

  const visit = (node) => {
    if (ts.isReturnStatement(node) && node.expression) {
      let call = node.expression;
      if (!ts.isCallExpression(call)) {
        ts.forEachChild(node, visit);
        return;
      }
      // We expect either wasm.<Class>.<method>(...) or _WebClient.<method>(...).
      const callee = call.expression;
      if (!ts.isPropertyAccessExpression(callee)) return;

      // _WebClient.<method>(...)
      if (ts.isIdentifier(callee.expression)) {
        const refName = callee.expression.text;
        const cls = WASM_REF_TO_CLASS[refName];
        if (cls) {
          target = { cls, method: callee.name.text };
        }
        return;
      }

      // wasm.<Class>.<method>(...)
      if (
        ts.isPropertyAccessExpression(callee.expression) &&
        ts.isIdentifier(callee.expression.expression) &&
        callee.expression.expression.text === "wasm"
      ) {
        target = {
          cls: callee.expression.name.text,
          method: callee.name.text,
        };
      }
      return;
    }
    ts.forEachChild(node, visit);
  };

  visit(functionNode.body);
  return target;
}

function collectForwarders(sourceFile) {
  const forwarders = new Map();
  const skipped = [];

  ts.forEachChild(sourceFile, (node) => {
    if (!ts.isFunctionDeclaration(node)) return;
    const isExported = node.modifiers?.some(
      (m) => m.kind === ts.SyntaxKind.ExportKeyword
    );
    if (!isExported || !node.name || !node.body) return;
    // Internal setter helpers begin with `_` and are not public API.
    if (node.name.text.startsWith("_")) return;

    const target = extractForwarderTarget(node, sourceFile);
    if (target) {
      forwarders.set(node.name.text, target);
    } else {
      skipped.push(node.name.text);
    }
  });

  return { forwarders, skipped };
}

function buildTypeChecker() {
  const program = ts.createProgram({
    rootNames: [apiTypesPath, wasmTypesPath],
    options: {
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.NodeJs,
      strict: true,
      skipLibCheck: true,
      noEmit: true,
    },
  });
  return { program, checker: program.getTypeChecker() };
}

function getExportedFunctionReturnType(program, checker, filePath, name) {
  const sourceFile = program.getSourceFile(filePath);
  if (!sourceFile) throw new Error(`source file not loaded: ${filePath}`);
  const moduleSymbol = checker.getSymbolAtLocation(sourceFile);
  if (!moduleSymbol) throw new Error(`no module symbol: ${filePath}`);
  const exports = checker.getExportsOfModule(moduleSymbol);
  const symbol = exports.find((s) => s.name === name);
  if (!symbol) return null;
  const type = checker.getTypeOfSymbolAtLocation(
    symbol,
    symbol.declarations?.[0] ?? sourceFile
  );
  const signature = type.getCallSignatures()[0];
  if (!signature) return null;
  return signature.getReturnType();
}

function getStaticMethodReturnType(program, checker, cls, method) {
  const sourceFile = program.getSourceFile(wasmTypesPath);
  if (!sourceFile) throw new Error(`source file not loaded: ${wasmTypesPath}`);
  const moduleSymbol = checker.getSymbolAtLocation(sourceFile);
  if (!moduleSymbol) throw new Error("no bindgen module symbol");
  const classSymbol = checker
    .getExportsOfModule(moduleSymbol)
    .find((s) => s.name === cls);
  if (!classSymbol) return null;
  // Static members live on the class symbol's own exports (not on the
  // instance type). `getTypeOfSymbolAtLocation` on the class gives the
  // constructor type, whose properties include the statics.
  const classType = checker.getTypeOfSymbolAtLocation(
    classSymbol,
    classSymbol.declarations?.[0] ?? sourceFile
  );
  const methodSymbol = classType.getProperty(method);
  if (!methodSymbol) return null;
  const methodType = checker.getTypeOfSymbolAtLocation(
    methodSymbol,
    methodSymbol.declarations?.[0] ?? sourceFile
  );
  const signature = methodType.getCallSignatures()[0];
  if (!signature) return null;
  return signature.getReturnType();
}

const standaloneText = await readFile(standalonePath, "utf8").catch(() => null);
if (!standaloneText) {
  console.error(
    `standalone.js not found at ${standalonePath} — nothing to check.`
  );
  process.exit(0);
}

try {
  await readFile(apiTypesPath);
  await readFile(wasmTypesPath);
} catch {
  console.error(
    "Standalone type check failed: expected type files are missing. Run `yarn build` first."
  );
  process.exit(1);
}

const sourceFile = parseSourceFile(
  standalonePath,
  standaloneText,
  ts.ScriptKind.JS
);
const { forwarders, skipped } = collectForwarders(sourceFile);

if (skipped.length > 0) {
  console.warn(
    `[check-standalone-types] Skipping non-forwarder exports (their return types are not auto-checked): ${skipped.join(", ")}`
  );
}

if (forwarders.size === 0) {
  console.log(
    "[check-standalone-types] No forwarder wrappers found in standalone.js."
  );
  process.exit(0);
}

const { program, checker } = buildTypeChecker();
const mismatches = [];

for (const [wrapperName, { cls, method }] of forwarders) {
  const declaredReturn = getExportedFunctionReturnType(
    program,
    checker,
    apiTypesPath,
    wrapperName
  );
  if (!declaredReturn) {
    mismatches.push({
      wrapperName,
      reason: `no exported declaration in ${path.relative(rootDir, apiTypesPath)}`,
    });
    continue;
  }
  const bindgenReturn = getStaticMethodReturnType(
    program,
    checker,
    cls,
    method
  );
  if (!bindgenReturn) {
    mismatches.push({
      wrapperName,
      reason: `could not resolve ${cls}.${method} in wasm-bindgen types`,
    });
    continue;
  }
  const declared = checker.typeToString(declaredReturn);
  const bindgen = checker.typeToString(bindgenReturn);
  if (declared !== bindgen) {
    mismatches.push({
      wrapperName,
      reason: `declared return \`${declared}\` does not match ${cls}.${method} return \`${bindgen}\``,
    });
  }
}

if (mismatches.length > 0) {
  console.error(
    "[check-standalone-types] Standalone wrapper return types drift from the underlying wasm-bindgen methods:"
  );
  for (const { wrapperName, reason } of mismatches) {
    console.error(`- ${wrapperName}: ${reason}`);
  }
  console.error(
    'Update js/types/api-types.d.ts so the declaration matches the wasm binding (or use `ReturnType<WasmModule["Class"]["method"]>` to source it automatically).'
  );
  process.exit(1);
}

console.log(
  `[check-standalone-types] All ${forwarders.size} forwarder wrapper(s) have return types matching their wasm-bindgen sources.`
);
