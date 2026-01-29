import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";
import {
  MidenProvider,
  useImportAccount,
  useTransaction,
  useMiden,
} from "@miden-sdk/react";
import {
  WebClient,
  MockWebClient,
  AccountStorageMode,
  NoteType,
} from "@miden-sdk/miden-sdk";

window.testAppError = null;
window.sdkLoaded = false;
window.sdkLoadError = null;
window.reactSdkReady = false;

window.addEventListener("error", (event) => {
  const message = event?.error?.message ?? event?.message ?? "Unknown error";
  const details = {
    message,
    filename: event?.filename ?? null,
    line: event?.lineno ?? null,
    column: event?.colno ?? null,
  };
  window.testAppError = JSON.stringify(details);
});

window.addEventListener("unhandledrejection", (event) => {
  const message =
    event?.reason?.message ?? event?.reason ?? "Unhandled rejection";
  const stack = event?.reason?.stack ?? null;
  window.testAppError = JSON.stringify({ message, stack });
});

const exposeSdk = (sdkExports) => {
  for (const [key, value] of Object.entries(sdkExports)) {
    window[key] = value;
  }
};

const initSdk = async () => {
  try {
    const sdkExports = await import("@miden-sdk/miden-sdk");
    exposeSdk(sdkExports);
    window.sdkLoaded = true;
  } catch (err) {
    console.error("Failed to load SDK:", err);
    window.sdkLoadError = err?.message ?? String(err);
  }
};

const patchWebClient = () => {
  if (typeof WebClient?.createClient === "function") {
    WebClient.createClient = MockWebClient.createClient.bind(MockWebClient);
  }
};

const waitForClient = (isReady, client) => {
  if (!isReady || !client) {
    throw new Error("Miden client not ready");
  }
  return client;
};

const TestHarness = () => {
  const { client, isReady } = useMiden();
  const { execute, stage: transactionStage, error: transactionError } =
    useTransaction();
  const { importAccount, account, isImporting, error: importError } =
    useImportAccount();

  useEffect(() => {
    window.reactSdkReady = !!isReady;
  }, [isReady]);

  useEffect(() => {
    window.__reactSdkState = () => ({
      isReady: !!isReady,
      transactionStage,
      transactionError: transactionError ? transactionError.message : null,
      isImporting,
      importError: importError ? importError.message : null,
      importedAccountId: account ? account.id().toString() : null,
    });
  }, [
    isReady,
    transactionStage,
    transactionError,
    isImporting,
    importError,
    account,
  ]);

  useEffect(() => {
    if (!isReady || !client) {
      return;
    }

    window.__reactSdk = {
      runTransaction: async () => {
        const readyClient = waitForClient(isReady, client);
        const wallet = await readyClient.newWallet(
          AccountStorageMode.private(),
          true,
          0
        );
        const faucet = await readyClient.newFaucet(
          AccountStorageMode.private(),
          false,
          "TEST",
          8,
          BigInt(1000000),
          0
        );

        const request = readyClient.newMintTransactionRequest(
          wallet.id(),
          faucet.id(),
          NoteType.Public,
          BigInt(1)
        );

        const result = await execute({
          accountId: faucet.id(),
          request,
        });

        return { transactionId: result.transactionId };
      },
      importAccountFromFile: async () => {
        const readyClient = waitForClient(isReady, client);
        const wallet = await readyClient.newWallet(
          AccountStorageMode.private(),
          true,
          0
        );
        const accountFile = await readyClient.exportAccountFile(wallet.id());
        const imported = await importAccount({
          type: "file",
          file: accountFile,
        });
        return { accountId: imported.id().toString() };
      },
    };
  }, [client, execute, importAccount, isReady]);

  return null;
};

const App = () =>
  React.createElement(
    MidenProvider,
    { config: { autoSyncInterval: 0 } },
    React.createElement(TestHarness, null)
  );

patchWebClient();
initSdk();

const root = createRoot(document.getElementById("root"));
root.render(React.createElement(App, null));
