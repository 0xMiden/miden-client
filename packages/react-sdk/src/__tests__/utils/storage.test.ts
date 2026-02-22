import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  migrateStorage,
  clearMidenStorage,
  createMidenStorage,
} from "../../utils/storage";

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    key: vi.fn((index: number) => Object.keys(store)[index] ?? null),
    get length() {
      return Object.keys(store).length;
    },
    clear: vi.fn(() => {
      store = {};
    }),
    _store: store,
    _reset: () => {
      store = {};
    },
  };
})();

beforeEach(() => {
  localStorageMock._reset();
  vi.stubGlobal("localStorage", localStorageMock);
  vi.clearAllMocks();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("createMidenStorage", () => {
  it("should set and get values with prefixed keys", () => {
    const storage = createMidenStorage("myapp");
    storage.set("foo", { bar: 42 });
    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      "myapp:foo",
      JSON.stringify({ bar: 42 })
    );

    const result = storage.get<{ bar: number }>("foo");
    expect(result).toEqual({ bar: 42 });
  });

  it("should return null for missing keys", () => {
    const storage = createMidenStorage("myapp");
    expect(storage.get("nonexistent")).toBeNull();
  });

  it("should remove values", () => {
    const storage = createMidenStorage("myapp");
    storage.set("key", "value");
    storage.remove("key");
    expect(localStorageMock.removeItem).toHaveBeenCalledWith("myapp:key");
  });

  it("should clear only prefixed keys", () => {
    const storage = createMidenStorage("myapp");
    storage.set("a", 1);
    storage.set("b", 2);
    // Simulate a key from another prefix
    localStorageMock.setItem("other:c", "3");

    storage.clear();

    // "myapp:a" and "myapp:b" should be removed
    expect(localStorageMock.removeItem).toHaveBeenCalledWith("myapp:a");
    expect(localStorageMock.removeItem).toHaveBeenCalledWith("myapp:b");
    // "other:c" should still exist
    expect(localStorageMock.getItem("other:c")).toBe("3");
  });

  it("should handle get with invalid JSON gracefully", () => {
    localStorageMock.setItem("myapp:bad", "not-json{{{");
    const storage = createMidenStorage("myapp");
    expect(storage.get("bad")).toBeNull();
  });

  it("should store primitive values", () => {
    const storage = createMidenStorage("test");
    storage.set("str", "hello");
    expect(storage.get("str")).toBe("hello");

    storage.set("num", 42);
    expect(storage.get("num")).toBe(42);

    storage.set("bool", true);
    expect(storage.get("bool")).toBe(true);
  });
});

describe("migrateStorage", () => {
  // Mock indexedDB for clearMidenStorage
  const mockDeleteDatabase = vi.fn().mockImplementation(() => {
    const req = { onsuccess: null as any, onerror: null as any, onblocked: null as any };
    setTimeout(() => req.onsuccess?.(), 0);
    return req;
  });

  beforeEach(() => {
    vi.stubGlobal("indexedDB", {
      databases: vi.fn().mockResolvedValue([]),
      deleteDatabase: mockDeleteDatabase,
    });
    // Mock window.location.reload
    vi.stubGlobal("window", {
      location: { reload: vi.fn() },
    });
  });

  it("should return false when version matches", async () => {
    localStorageMock.setItem("miden:storageVersion", "1.0.0");
    const result = await migrateStorage({ version: "1.0.0" });
    expect(result).toBe(false);
  });

  it("should return true and clear when version mismatches", async () => {
    localStorageMock.setItem("miden:storageVersion", "0.9.0");
    const result = await migrateStorage({
      version: "1.0.0",
      reloadOnClear: false,
    });
    expect(result).toBe(true);
    expect(localStorageMock.getItem("miden:storageVersion")).toBe("1.0.0");
  });

  it("should trigger migration on first run (no stored version)", async () => {
    const result = await migrateStorage({
      version: "1.0.0",
      reloadOnClear: false,
    });
    expect(result).toBe(true);
  });

  it("should call onBeforeClear callback", async () => {
    localStorageMock.setItem("miden:storageVersion", "old");
    const onBeforeClear = vi.fn();
    await migrateStorage({
      version: "new",
      onBeforeClear,
      reloadOnClear: false,
    });
    expect(onBeforeClear).toHaveBeenCalledOnce();
  });

  it("should use custom version key", async () => {
    localStorageMock.setItem("custom:version", "1.0.0");
    const result = await migrateStorage({
      version: "1.0.0",
      versionKey: "custom:version",
    });
    expect(result).toBe(false);
  });

  it("should reload by default when clearing", async () => {
    const reloadFn = vi.fn();
    vi.stubGlobal("window", { location: { reload: reloadFn } });

    await migrateStorage({ version: "1.0.0" });
    expect(reloadFn).toHaveBeenCalled();
  });

  it("should not reload when reloadOnClear is false", async () => {
    const reloadFn = vi.fn();
    vi.stubGlobal("window", { location: { reload: reloadFn } });

    await migrateStorage({ version: "1.0.0", reloadOnClear: false });
    expect(reloadFn).not.toHaveBeenCalled();
  });
});

describe("clearMidenStorage", () => {
  it("should delete miden-related databases", async () => {
    const deleteDb = vi.fn().mockImplementation(() => {
      const req = { onsuccess: null as any, onerror: null as any, onblocked: null as any };
      setTimeout(() => req.onsuccess?.(), 0);
      return req;
    });

    vi.stubGlobal("indexedDB", {
      databases: vi.fn().mockResolvedValue([
        { name: "miden-client-db" },
        { name: "other-db" },
        { name: "MidenStore" },
      ]),
      deleteDatabase: deleteDb,
    });

    await clearMidenStorage();

    // Should only delete miden-related dbs
    expect(deleteDb).toHaveBeenCalledTimes(2);
    expect(deleteDb).toHaveBeenCalledWith("miden-client-db");
    expect(deleteDb).toHaveBeenCalledWith("MidenStore");
  });

  it("should handle missing indexedDB gracefully", async () => {
    vi.stubGlobal("indexedDB", undefined);
    await expect(clearMidenStorage()).resolves.toBeUndefined();
  });

  it("should handle databases() rejection gracefully", async () => {
    vi.stubGlobal("indexedDB", {
      databases: vi.fn().mockRejectedValue(new Error("not supported")),
    });
    await expect(clearMidenStorage()).resolves.toBeUndefined();
  });
});
