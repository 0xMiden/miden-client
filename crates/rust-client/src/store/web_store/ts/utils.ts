import Dexie from "dexie";
// Helper for undefined values, like map for Option<T> in Rust.
// A better name for this is welcome.
export const mapOption = <T, U>(
  value: T | null | undefined,
  func: (value: T) => U
): U | undefined => {
  return value != undefined ? func(value) : undefined;
};

export const logDexieError = (error: any, errorContext?: string) => {
  if (error instanceof Dexie.DexieError) {
    if (errorContext) {
      console.error(
        `${errorContext}: Indexdb error (${error.name}): ${error.message}`
      );
    } else {
      console.error(`Indexdb error: (${error.name}): ${error.message}`);
    }
    mapOption(error.stack, (stack) => console.error(`Stacktrace: \n ${stack}`));
    mapOption(error.inner, (innerException) => logDexieError(innerException));
    throw error;
  } else {
    console.error(
      `Unexpected error while accessing indexdb: ${error.toString()}`
    );
    mapOption(error.stack, (stack) => console.error(`Stacktrace: n\ ${stack}`));
    throw error;
  }
};

export const uint8ArrayToBase64 = (bytes: Uint8Array) => {
  const binary = bytes.reduce(
    (acc, byte) => acc + String.fromCharCode(byte),
    ""
  );
  return btoa(binary);
};
