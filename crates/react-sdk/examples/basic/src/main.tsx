import React from "react";
import ReactDOM from "react-dom/client";
import { MidenProvider } from "@miden-sdk/react";
import { ErrorBoundary } from "./components/ErrorBoundary";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <MidenProvider
        config={{
          // Default testnet RPC - change this for your environment
          rpcUrl: "https://rpc.testnet.miden.io",
        }}
      >
        <App />
      </MidenProvider>
    </ErrorBoundary>
  </React.StrictMode>
);
