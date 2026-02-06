/**
 * BioFabric Web — Entry Point
 *
 * Initializes the WASM module, sets up the WebGL2 renderer, and wires
 * UI controls to the core BioFabric engine.
 */

import { initWasm } from "./wasm/bridge";
import { FabricRenderer } from "./renderer/fabric-renderer";
import { initApp } from "./ui/app";

async function main(): Promise<void> {
  const status = document.getElementById("status")!;

  try {
    // 1. Initialize WASM module
    status.textContent = "Loading WASM...";
    const wasm = await initWasm();

    // 2. Initialize WebGL2 renderer
    status.textContent = "Initializing renderer...";
    const canvas = document.getElementById("fabric-canvas") as HTMLCanvasElement;
    const renderer = new FabricRenderer(canvas);

    // 3. Wire up UI
    status.textContent = "Ready";
    initApp(wasm, renderer);
  } catch (err) {
    status.textContent = `Error: ${err}`;
    console.error("BioFabric init failed:", err);
  }
}

main();
