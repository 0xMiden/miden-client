async function loadWasm() {
  if (!import.meta.env.SSR) {
    const wasmModule = await import("../Cargo.toml");
  }
  return wasmModule;
}
export default loadWasm;
