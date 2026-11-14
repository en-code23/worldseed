// Loads the Rust core (compiled to WebAssembly).
import init, { WorldHandle, build_seed, seed_catalog } from "../pkg/seedcore.js";
import wasmUrl from "../pkg/seedcore_bg.wasm?url";

let ready: Promise<unknown> | null = null;
export function loadCore() {
  ready ??= init({ module_or_path: wasmUrl });
  return ready;
}
export { WorldHandle, build_seed, seed_catalog };
