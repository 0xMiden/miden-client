import Dexie from "dexie";
export const isNodeRuntime = () => typeof process !== "undefined" && !!process.versions?.node;
export const isBrowserRuntime = () => typeof window !== "undefined" && typeof window.document !== "undefined";
// Helper for undefined values, like map for Option<T> in Rust.
// A better name for this is welcome.
export const mapOption = (value, func) => {
    return value != undefined ? func(value) : undefined;
};
const isDexieError = (error) => typeof Dexie !== "undefined" &&
    typeof indexedDB !== "undefined" &&
    error instanceof Dexie.DexieError;
// Anything can be thrown as an error in raw JS (also the TS compiler can't type-check exceptions),
// so we allow it here.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const logWebStoreError = (error, errorContext) => {
    if (isDexieError(error)) {
        if (errorContext) {
            console.error(`${errorContext}: Indexdb error (${error.name}): ${error.message}`);
        }
        else {
            console.error(`Indexdb error: (${error.name}): ${error.message}`);
        }
        mapOption(error.stack, (stack) => {
            console.error(`Stacktrace: \n ${stack}`);
        });
        mapOption(error.inner, (innerException) => logWebStoreError(innerException));
    }
    else if (error instanceof Error) {
        console.error(`Unexpected error while accessing persistent store: ${error.toString()}`);
        mapOption(error.stack, (stack) => {
            console.error(`Stacktrace: ${stack}`);
        });
    }
    else {
        console.error(`Got an exception with a non-error value, as JSON: \n ${JSON.stringify(error)}. As String \n ${String(error)} `);
        console.trace();
    }
    throw error;
};
export const uint8ArrayToBase64 = (bytes) => {
    if (typeof Buffer !== "undefined") {
        return Buffer.from(bytes).toString("base64");
    }
    const binary = bytes.reduce((acc, byte) => acc + String.fromCharCode(byte), "");
    return btoa(binary);
};
