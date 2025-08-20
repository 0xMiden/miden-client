// This is a documented workaround that should avoid issues with Vite projects
// https://github.com/wasm-tool/rollup-plugin-rust?tab=readme-ov-file#usage-with-vite
async function loadWasm() {
  let wasmModule;
  if (!import.meta.env.SSR) {
    wasmModule = await import("../Cargo.toml");
  }
  return wasmModule;
}
export default loadWasm;
