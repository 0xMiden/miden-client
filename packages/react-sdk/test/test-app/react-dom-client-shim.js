const ReactDOM = window.ReactDOM;

if (!ReactDOM || typeof ReactDOM.createRoot !== "function") {
  throw new Error(
    "ReactDOM.createRoot not found. Ensure react-dom UMD is loaded."
  );
}

export const createRoot = ReactDOM.createRoot;
