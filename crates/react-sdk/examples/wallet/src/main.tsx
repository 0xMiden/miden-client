import React from "react";
import ReactDOM from "react-dom/client";
import { MidenProvider } from "@miden-sdk/react";
import "./index.css";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <MidenProvider
      config={{
        rpcUrl: "https://rpc.devnet.miden.io",
        noteTransportUrl: "http://transport.miden.io:57292",
        autoSyncInterval: 15000,
      }}
    >
      <App />
    </MidenProvider>
  </React.StrictMode>
);
