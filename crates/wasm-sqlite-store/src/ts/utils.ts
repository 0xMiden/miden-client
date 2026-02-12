/**
 * Utility functions for the WASM SQLite store.
 */

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const logError = (error: any, errorContext?: string) => {
  if (error instanceof Error) {
    if (errorContext) {
      console.error(`${errorContext}: ${error.message}`);
    } else {
      console.error(`SQLite store error: ${error.message}`);
    }
    if (error.stack) {
      console.error(`Stacktrace:\n${error.stack}`);
    }
  } else {
    console.error(
      `Got an exception with a non-error value: ${JSON.stringify(error)}`
    );
    console.trace();
  }
  throw error;
};

export const uint8ArrayToBase64 = (bytes: Uint8Array): string => {
  const binary = bytes.reduce(
    (acc, byte) => acc + String.fromCharCode(byte),
    ""
  );
  return btoa(binary);
};

export const base64ToUint8Array = (base64: string): Uint8Array => {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
};
