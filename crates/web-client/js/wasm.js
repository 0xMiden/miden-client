// This is a documented workaround that should avoid issues with Vite projects
// https://github.com/wasm-tool/rollup-plugin-rust?tab=readme-ov-file#usage-with-vite
// Also, this effectively disables SSR.
async function loadWasm() {
  let wasmModule;
  if (!import.meta.env || (import.meta.env && !import.meta.env.SSR)) {
    wasmModule = await import("../Cargo.toml");
    // The Cargo glue's __wbg_init TLA is stripped by the rollup build to
    // prevent blocking WKWebView module evaluation. Call it explicitly here
    // with the WASM URL that the Cargo glue pre-resolves (relative to its
    // own import.meta.url, which downstream bundlers handle correctly).
    if (wasmModule && typeof wasmModule.__wbg_init === "function") {
      await wasmModule.__wbg_init({
        // eslint-disable-next-line camelcase -- wasm-bindgen init API parameter name
        module_or_path: wasmModule.__wasm_url,
      });
    }
  }
  return wasmModule;
}
export default loadWasm;
