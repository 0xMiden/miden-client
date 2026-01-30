import React from "react";
import ReactDOM from "react-dom/client";
import { MidenProvider } from "@miden-sdk/react";
import "./index.css";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <MidenProvider
      config={{
        rpcUrl: "devnet",
      }}
    >
      <App />
    </MidenProvider>
  </React.StrictMode>
);
