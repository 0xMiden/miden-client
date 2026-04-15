import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSyncControl } from "../../hooks/useSyncControl";
import { useMidenStore } from "../../store/MidenStore";

beforeEach(() => {
  useMidenStore.getState().reset();
  vi.clearAllMocks();
});

describe("useSyncControl", () => {
  it("should return initial state with isPaused false", () => {
    const { result } = renderHook(() => useSyncControl());

    expect(result.current.isPaused).toBe(false);
    expect(typeof result.current.pauseSync).toBe("function");
    expect(typeof result.current.resumeSync).toBe("function");
  });

  it("should pause sync", () => {
    const { result } = renderHook(() => useSyncControl());

    act(() => {
      result.current.pauseSync();
    });

    expect(result.current.isPaused).toBe(true);
    expect(useMidenStore.getState().syncPaused).toBe(true);
  });

  it("should resume sync", () => {
    useMidenStore.getState().setSyncPaused(true);

    const { result } = renderHook(() => useSyncControl());

    expect(result.current.isPaused).toBe(true);

    act(() => {
      result.current.resumeSync();
    });

    expect(result.current.isPaused).toBe(false);
    expect(useMidenStore.getState().syncPaused).toBe(false);
  });

  it("should handle multiple pause/resume cycles", () => {
    const { result } = renderHook(() => useSyncControl());

    act(() => {
      result.current.pauseSync();
    });
    expect(result.current.isPaused).toBe(true);

    act(() => {
      result.current.resumeSync();
    });
    expect(result.current.isPaused).toBe(false);

    act(() => {
      result.current.pauseSync();
    });
    expect(result.current.isPaused).toBe(true);
  });

  it("should reset syncPaused on store reset", () => {
    const { result } = renderHook(() => useSyncControl());

    act(() => {
      result.current.pauseSync();
    });
    expect(result.current.isPaused).toBe(true);

    act(() => {
      useMidenStore.getState().reset();
    });

    expect(result.current.isPaused).toBe(false);
  });
});

describe("MidenStore syncPaused", () => {
  it("should have syncPaused false by default", () => {
    expect(useMidenStore.getState().syncPaused).toBe(false);
  });

  it("should set syncPaused via setSyncPaused", () => {
    useMidenStore.getState().setSyncPaused(true);
    expect(useMidenStore.getState().syncPaused).toBe(true);

    useMidenStore.getState().setSyncPaused(false);
    expect(useMidenStore.getState().syncPaused).toBe(false);
  });

  it("should reset syncPaused to false on reset", () => {
    useMidenStore.getState().setSyncPaused(true);
    useMidenStore.getState().reset();
    expect(useMidenStore.getState().syncPaused).toBe(false);
  });
});
