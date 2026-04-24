import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { CompilerResource, getWasmOrThrow } from "@miden-sdk/miden-sdk/lazy";
import { useCompile } from "../../hooks/useCompile";
import { useMiden } from "../../context/MidenProvider";
import { createMockWebClient } from "../mocks/miden-sdk";

vi.mock("../../context/MidenProvider", () => ({
  useMiden: vi.fn(),
}));

vi.mock("@miden-sdk/miden-sdk/lazy", () => {
  const instances: Array<{
    component: ReturnType<typeof vi.fn>;
    txScript: ReturnType<typeof vi.fn>;
    noteScript: ReturnType<typeof vi.fn>;
  }> = [];

  const CompilerResourceMock = vi.fn().mockImplementation(() => {
    const instance = {
      component: vi.fn().mockResolvedValue({ kind: "component" }),
      txScript: vi.fn().mockResolvedValue({ kind: "txScript" }),
      noteScript: vi.fn().mockResolvedValue({ kind: "noteScript" }),
    };
    instances.push(instance);
    return instance;
  });
  (
    CompilerResourceMock as unknown as { __instances: typeof instances }
  ).__instances = instances;

  return {
    CompilerResource: CompilerResourceMock,
    getWasmOrThrow: vi.fn().mockResolvedValue({}),
    Linking: { Dynamic: "dynamic", Static: "static" },
  };
});

const mockUseMiden = useMiden as ReturnType<typeof vi.fn>;
const CompilerResourceCtor = CompilerResource as unknown as ReturnType<
  typeof vi.fn
> & {
  __instances: Array<{
    component: ReturnType<typeof vi.fn>;
    txScript: ReturnType<typeof vi.fn>;
    noteScript: ReturnType<typeof vi.fn>;
  }>;
};

beforeEach(() => {
  vi.clearAllMocks();
  CompilerResourceCtor.__instances.length = 0;
});

describe("useCompile", () => {
  describe("initial state", () => {
    it("returns the three compile functions and isReady", () => {
      mockUseMiden.mockReturnValue({ client: null, isReady: false });

      const { result } = renderHook(() => useCompile());

      expect(typeof result.current.component).toBe("function");
      expect(typeof result.current.txScript).toBe("function");
      expect(typeof result.current.noteScript).toBe("function");
      expect(result.current.isReady).toBe(false);
    });

    it("does not instantiate CompilerResource when client is not ready", () => {
      mockUseMiden.mockReturnValue({ client: null, isReady: false });

      renderHook(() => useCompile());

      expect(CompilerResourceCtor).not.toHaveBeenCalled();
    });

    it("instantiates CompilerResource with the client and getWasmOrThrow when ready", () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({ client: mockClient, isReady: true });

      renderHook(() => useCompile());

      expect(CompilerResourceCtor).toHaveBeenCalledTimes(1);
      expect(CompilerResourceCtor).toHaveBeenCalledWith(
        mockClient,
        getWasmOrThrow
      );
    });
  });

  describe("not-ready guard", () => {
    it("component() throws when client is not ready", async () => {
      mockUseMiden.mockReturnValue({ client: null, isReady: false });

      const { result } = renderHook(() => useCompile());

      await expect(
        result.current.component({ code: "begin end" })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("txScript() throws when client is not ready", async () => {
      mockUseMiden.mockReturnValue({ client: null, isReady: false });

      const { result } = renderHook(() => useCompile());

      await expect(
        result.current.txScript({ code: "begin end" })
      ).rejects.toThrow("Miden client is not ready");
    });

    it("noteScript() throws when client is not ready", async () => {
      mockUseMiden.mockReturnValue({ client: null, isReady: false });

      const { result } = renderHook(() => useCompile());

      await expect(
        result.current.noteScript({ code: "begin end" })
      ).rejects.toThrow("Miden client is not ready");
    });
  });

  describe("delegation to CompilerResource", () => {
    it("component() forwards options to resource.component()", async () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({ client: mockClient, isReady: true });

      const { result } = renderHook(() => useCompile());
      const options = { code: "masm source", slots: [] };

      let out: unknown;
      await act(async () => {
        out = await result.current.component(options);
      });

      const resource = CompilerResourceCtor.__instances[0];
      expect(resource.component).toHaveBeenCalledWith(options);
      expect(out).toEqual({ kind: "component" });
    });

    it("txScript() forwards options to resource.txScript()", async () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({ client: mockClient, isReady: true });

      const { result } = renderHook(() => useCompile());
      const options = {
        code: "masm tx",
        libraries: [
          { namespace: "lib::m", code: "...", linking: "dynamic" as const },
        ],
      };

      let out: unknown;
      await act(async () => {
        out = await result.current.txScript(options);
      });

      const resource = CompilerResourceCtor.__instances[0];
      expect(resource.txScript).toHaveBeenCalledWith(options);
      expect(out).toEqual({ kind: "txScript" });
    });

    it("noteScript() forwards options to resource.noteScript()", async () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({ client: mockClient, isReady: true });

      const { result } = renderHook(() => useCompile());
      const options = {
        code: "masm note",
        libraries: [
          { namespace: "lib::m", code: "...", linking: "static" as const },
        ],
      };

      let out: unknown;
      await act(async () => {
        out = await result.current.noteScript(options);
      });

      const resource = CompilerResourceCtor.__instances[0];
      expect(resource.noteScript).toHaveBeenCalledWith(options);
      expect(out).toEqual({ kind: "noteScript" });
    });
  });

  describe("resource lifecycle", () => {
    it("reuses the same CompilerResource across calls on the same client", async () => {
      const mockClient = createMockWebClient();
      mockUseMiden.mockReturnValue({ client: mockClient, isReady: true });

      const { result } = renderHook(() => useCompile());

      await act(async () => {
        await result.current.noteScript({ code: "begin end" });
        await result.current.txScript({ code: "begin end" });
      });

      expect(CompilerResourceCtor).toHaveBeenCalledTimes(1);
    });
  });
});
