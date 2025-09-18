import { transportLayerCursor } from "./schema.js";
import { logWebStoreError } from "./utils.js";
export async function getTransportLayerCursor() {
    try {
        const record = await transportLayerCursor.get(1);
        return record ? record.cursor : 0;
    }
    catch (error) {
        logWebStoreError(error, "Error getting transport layer cursor");
        return 0;
    }
}
export async function updateTransportLayerCursor(cursor) {
    try {
        await transportLayerCursor.put({ id: 1, cursor: cursor });
    }
    catch (error) {
        logWebStoreError(error, "Error updating transport layer cursor");
    }
}
//# sourceMappingURL=transport.js.map