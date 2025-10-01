import { noteTransportCursor } from "./schema.js";
import { logWebStoreError } from "./utils.js";
export async function getNoteTransportCursor() {
    try {
        const record = await noteTransportCursor.get(1);
        return record ? record.cursor : 0;
    }
    catch (error) {
        logWebStoreError(error, "Error getting note transport cursor");
        return 0;
    }
}
export async function updateNoteTransportCursor(cursor) {
    try {
        await noteTransportCursor.put({ id: 1, cursor: cursor });
    }
    catch (error) {
        logWebStoreError(error, "Error updating note transport cursor");
    }
}
//# sourceMappingURL=transport.js.map
