import { describe, it, expect } from "vitest";
import { MidenError, wrapWasmError } from "../../utils/errors";

describe("MidenError", () => {
  it("should create error with default code UNKNOWN", () => {
    const err = new MidenError("something went wrong");
    expect(err.message).toBe("something went wrong");
    expect(err.code).toBe("UNKNOWN");
    expect(err.name).toBe("MidenError");
    expect(err).toBeInstanceOf(Error);
  });

  it("should create error with specified code", () => {
    const err = new MidenError("busy", { code: "SEND_BUSY" });
    expect(err.code).toBe("SEND_BUSY");
  });

  it("should accept a cause option", () => {
    const cause = new Error("original");
    const err = new MidenError("wrapped", { cause, code: "UNKNOWN" });
    expect(err.message).toBe("wrapped");
    // cause is stored but not passed to super (ES2020 compat)
    expect(err.code).toBe("UNKNOWN");
  });
});

describe("wrapWasmError", () => {
  it("should return MidenError instances unchanged", () => {
    const original = new MidenError("already wrapped", { code: "SEND_BUSY" });
    expect(wrapWasmError(original)).toBe(original);
  });

  it("should detect _assertClass pattern as WASM_CLASS_MISMATCH", () => {
    const err = new Error("_assertClass failed for AccountId");
    const wrapped = wrapWasmError(err);
    expect(wrapped).toBeInstanceOf(MidenError);
    expect((wrapped as MidenError).code).toBe("WASM_CLASS_MISMATCH");
    expect(wrapped.message).toContain("WASM class identity mismatch");
  });

  it("should detect 'expected instance of' as WASM_CLASS_MISMATCH", () => {
    const wrapped = wrapWasmError(new Error("expected instance of AccountId"));
    expect((wrapped as MidenError).code).toBe("WASM_CLASS_MISMATCH");
  });

  it("should detect 'null pointer' as WASM_POINTER_CONSUMED", () => {
    const wrapped = wrapWasmError(new Error("null pointer passed to Rust"));
    expect((wrapped as MidenError).code).toBe("WASM_POINTER_CONSUMED");
    expect(wrapped.message).toContain("WASM object was already consumed");
    expect(wrapped.message).not.toContain("AccountId");
  });

  it("should detect 'already been freed' as WASM_POINTER_CONSUMED", () => {
    const wrapped = wrapWasmError(new Error("object has already been freed"));
    expect((wrapped as MidenError).code).toBe("WASM_POINTER_CONSUMED");
  });

  it("should detect 'dereferencing a null' as WASM_POINTER_CONSUMED", () => {
    const wrapped = wrapWasmError(
      new Error("dereferencing a null pointer in WASM")
    );
    expect((wrapped as MidenError).code).toBe("WASM_POINTER_CONSUMED");
  });

  it("should detect 'not initialized' as WASM_NOT_INITIALIZED", () => {
    const wrapped = wrapWasmError(new Error("Client not initialized"));
    expect((wrapped as MidenError).code).toBe("WASM_NOT_INITIALIZED");
  });

  it("should detect 'Cannot read properties of null' as WASM_NOT_INITIALIZED", () => {
    const wrapped = wrapWasmError(
      new TypeError("Cannot read properties of null (reading 'syncState')")
    );
    expect((wrapped as MidenError).code).toBe("WASM_NOT_INITIALIZED");
  });

  it("should detect 'state commitment mismatch' as WASM_SYNC_REQUIRED", () => {
    const wrapped = wrapWasmError(
      new Error("state commitment mismatch for account")
    );
    expect((wrapped as MidenError).code).toBe("WASM_SYNC_REQUIRED");
    expect(wrapped.message).toContain("Account state is stale");
  });

  it("should detect 'stale state' as WASM_SYNC_REQUIRED", () => {
    const wrapped = wrapWasmError(new Error("stale state detected"));
    expect((wrapped as MidenError).code).toBe("WASM_SYNC_REQUIRED");
  });

  it("should return plain Error instances unchanged for unknown patterns", () => {
    const original = new Error("some random error");
    const result = wrapWasmError(original);
    expect(result).toBe(original);
    expect(result).not.toBeInstanceOf(MidenError);
  });

  it("should wrap non-Error values in a new Error", () => {
    const result = wrapWasmError("string error");
    expect(result).toBeInstanceOf(Error);
    expect(result.message).toBe("string error");
  });

  it("should wrap number values", () => {
    const result = wrapWasmError(42);
    expect(result.message).toBe("42");
  });

  it("should handle null/undefined", () => {
    expect(wrapWasmError(null).message).toBe("null");
    expect(wrapWasmError(undefined).message).toBe("undefined");
  });
});
