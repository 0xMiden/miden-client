import React from "react";
import ReactDOM from "react-dom/client";
import { MidenProvider, MultiSignerProvider, SignerSlot } from "@miden-sdk/react";
import { ParaSignerProvider } from "@miden-sdk/use-miden-para-react";
import { TurnkeySignerProvider } from "@miden-sdk/miden-turnkey-react";
import { MidenFiSignerProvider } from "@miden-sdk/miden-wallet-adapter-react";
import "./index.css";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <MultiSignerProvider>
      <ParaSignerProvider
        apiKey={import.meta.env.VITE_PARA_API_KEY}
        environment="BETA"
      >
        <SignerSlot />
      </ParaSignerProvider>
      <TurnkeySignerProvider>
        <SignerSlot />
      </TurnkeySignerProvider>
      <MidenFiSignerProvider network="testnet" autoConnect={false}>
        <SignerSlot />
      </MidenFiSignerProvider>
      <MidenProvider
        config={{
          rpcUrl: "testnet",
          prover: "testnet",
        }}
      >
        <App />
      </MidenProvider>
    </MultiSignerProvider>
  </React.StrictMode>
);
