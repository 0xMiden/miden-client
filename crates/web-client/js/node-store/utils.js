export const mapOption = (value, func) => {
  return value != undefined ? func(value) : undefined;
};

export const logWebStoreError = (error, errorContext) => {
  if (errorContext) {
    console.error(`${errorContext}: ${error}`);
  } else {
    console.error(`Store error: ${error}`);
  }
  if (error instanceof Error && error.stack) {
    console.error(`Stacktrace:\n${error.stack}`);
  }
  throw error;
};

export const uint8ArrayToBase64 = (bytes) => {
  const binary = bytes.reduce(
    (acc, byte) => acc + String.fromCharCode(byte),
    ""
  );
  return btoa(binary);
};
