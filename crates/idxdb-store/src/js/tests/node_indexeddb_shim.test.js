import { randomUUID } from "node:crypto";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { test } from "node:test";

const schemaUrl = new URL("../schema.js", import.meta.url);

const importSchemaFresh = () => {
  const freshUrl = new URL(schemaUrl.href);
  freshUrl.search = `?${randomUUID()}`;
  return import(freshUrl.href);
};

const removeDir = (targetPath) =>
  fs.rm(targetPath, { recursive: true, force: true });

test(
  "initializes the Node IndexedDB shim at the configured base path",
  async (t) => {
    const customBasePath = path.join(
      process.cwd(),
      "tmp-indexeddb-shim",
      randomUUID()
    );

    process.env.MIDEN_NODE_INDEXEDDB_BASE_PATH = customBasePath;
    await removeDir(customBasePath);

    let db;
    t.after(async () => {
      if (db) {
        db.close();
      }
      await removeDir(path.join(process.cwd(), "tmp-indexeddb-shim"));
      delete process.env.MIDEN_NODE_INDEXEDDB_BASE_PATH;
    });

    const schemaModule = await importSchemaFresh();

    db = schemaModule.db;
    const opened = await schemaModule.openDatabase("0.0.0-test");
    assert.equal(opened, true);
    assert.equal(schemaModule.nodeIndexedDbBasePath, customBasePath);

    const stats = await fs.stat(customBasePath);
    assert.ok(stats.isDirectory());
  }
);
