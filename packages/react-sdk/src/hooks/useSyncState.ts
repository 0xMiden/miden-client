import { useCallback } from "react";
import { useMiden } from "../context/MidenProvider";
import { useSyncStateStore } from "../store/MidenStore";
import type { SyncState } from "../types";

export interface UseSyncStateResult extends SyncState {
  /** Trigger a manual sync */
  sync: () => Promise<void>;
}

/**
 * Hook to access sync state and trigger manual syncs.
 *
 * @example
 * ```tsx
 * function SyncStatus() {
 *   const { syncHeight, isSyncing, sync } = useSyncState();
 *
 *   return (
 *     <div>
 *       <p>Block height: {syncHeight}</p>
 *       <button onClick={sync} disabled={isSyncing}>
 *         {isSyncing ? 'Syncing...' : 'Sync'}
 *       </button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useSyncState(): UseSyncStateResult {
  const { sync: triggerSync } = useMiden();
  const syncState = useSyncStateStore();

  const sync = useCallback(async () => {
    await triggerSync();
  }, [triggerSync]);

  return {
    ...syncState,
    sync,
  };
}
