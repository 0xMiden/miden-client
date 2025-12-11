#!/usr/bin/env node

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
const publicTypesPath = path.join(rootDir, "js", "types", "index.d.ts");

const requiredFiles = [wasmTypesPath, publicTypesPath];
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
    "Bindgen type check failed because expected files are missing. Run `yarn build` first."
  );
  for (const filePath of missingFiles) {
    console.error(`- ${filePath}`);
  }
  process.exit(1);
}

async function collectExports(filePath) {
  const sourceText = await readFile(filePath, "utf8");
  const sourceFile = ts.createSourceFile(
    filePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS
  );

  const names = new Set();

  const visit = (node) => {
    if (
      node.modifiers?.some(
        (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword
      )
    ) {
      if ("name" in node && node.name) {
        names.add(node.name.getText(sourceFile));
      } else if (ts.isVariableStatement(node)) {
        node.declarationList.declarations.forEach((declaration) => {
          names.add(declaration.name.getText(sourceFile));
        });
      }
    }

    if (
      ts.isExportDeclaration(node) &&
      node.exportClause &&
      ts.isNamedExports(node.exportClause)
    ) {
      node.exportClause.elements.forEach((element) => {
        names.add(element.name.getText(sourceFile));
      });
    }

    ts.forEachChild(node, visit);
  };

  visit(sourceFile);
  return names;
}

const wasmExports = await collectExports(wasmTypesPath);
const publicExports = await collectExports(publicTypesPath);

// The wrapper defines its own WebClient, so we do not expect to re-export the wasm-bindgen version.
const allowedMissing = new Set(["WebClient"]);
const missing = [...wasmExports].filter(
  (name) => !allowedMissing.has(name) && !publicExports.has(name)
);

if (missing.length > 0) {
  console.error(
    "Type declarations are missing the following wasm-bindgen exports:"
  );
  missing.forEach((name) => console.error(`- ${name}`));
  console.error(
    "Update js/types/index.d.ts so the published types reflect the generated bindings."
  );
  process.exit(1);
}

console.log(
  "Bindgen type check passed: all wasm exports are covered by the public TypeScript definitions."
);
