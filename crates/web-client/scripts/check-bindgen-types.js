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
  const starExportModules = [];

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

    if (ts.isExportDeclaration(node)) {
      if (node.exportClause && ts.isNamedExports(node.exportClause)) {
        // Named exports: export { A, B, C } from "..."
        node.exportClause.elements.forEach((element) => {
          names.add(element.name.getText(sourceFile));
        });
      } else if (!node.exportClause && node.moduleSpecifier) {
        // Star export: export * from "..."
        const modulePath = node.moduleSpecifier
          .getText(sourceFile)
          .slice(1, -1);
        starExportModules.push(modulePath);
      }
    }

    ts.forEachChild(node, visit);
  };

  visit(sourceFile);
  return { names, starExportModules };
}

async function resolveStarExports(filePath, starExportModules, baseDir = null) {
  const resolvedNames = new Set();
  // Use baseDir if provided, otherwise use the file's directory
  // This allows resolving paths as if the file was in a different location (e.g., dist/)
  const dir = baseDir || path.dirname(filePath);

  for (const modulePath of starExportModules) {
    // Resolve the module path relative to the base directory
    let resolvedPath = path.resolve(dir, modulePath);
    if (!resolvedPath.endsWith(".d.ts") && !resolvedPath.endsWith(".ts")) {
      resolvedPath += ".d.ts";
    }

    try {
      const { names } = await collectExports(resolvedPath);
      names.forEach((name) => resolvedNames.add(name));
    } catch {
      // If we can't resolve, skip (the module might be external)
    }
  }

  return resolvedNames;
}

const { names: wasmExports } = await collectExports(wasmTypesPath);
const { names: publicExports, starExportModules } =
  await collectExports(publicTypesPath);

// Resolve star exports to get all re-exported names
// Use dist/ as base directory since that's where the file will be deployed
// and where the relative imports (./crates/...) will be resolved from
const distDir = path.join(rootDir, "dist");
const starExportedNames = await resolveStarExports(
  publicTypesPath,
  starExportModules,
  distDir
);
starExportedNames.forEach((name) => publicExports.add(name));

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
