import { useCallback } from "react";
import { useMidenStore } from "../store/MidenStore";

export interface UseSyncControlResult {
  /** Pause auto-sync. Manual sync via useSyncState().sync() still works. */
  pauseSync: () => void;
  /** Resume auto-sync. */
  resumeSync: () => void;
  /** Whether auto-sync is currently paused. */
  isPaused: boolean;
}

/**
 * Hook to pause and resume the automatic background sync.
 *
 * Useful during long-running operations (e.g. transaction proving) where
 * sync would compete for the WASM lock, or when the app is in the background.
 *
 * @example
 * ```tsx
 * function SyncToggle() {
 *   const { pauseSync, resumeSync, isPaused } = useSyncControl();
 *
 *   return (
 *     <button onClick={isPaused ? resumeSync : pauseSync}>
 *       {isPaused ? 'Resume Sync' : 'Pause Sync'}
 *     </button>
 *   );
 * }
 * ```
 */
export function useSyncControl(): UseSyncControlResult {
  const isPaused = useMidenStore((state) => state.syncPaused);
  const setSyncPaused = useMidenStore((state) => state.setSyncPaused);

  const pauseSync = useCallback(() => setSyncPaused(true), [setSyncPaused]);
  const resumeSync = useCallback(() => setSyncPaused(false), [setSyncPaused]);

  return {
    pauseSync,
    resumeSync,
    isPaused,
  };
}
