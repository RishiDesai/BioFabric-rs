/**
 * Main application shell.
 *
 * Wires the WASM engine, renderer, and UI controls together.
 */

import type { BioFabricWasm, Handle } from "../wasm/bridge";
import type { FabricRenderer } from "../renderer/fabric-renderer";
import { initControls } from "./controls";
import { initFileLoader } from "./file-loader";
import { initInfoPanel } from "./info-panel";
import { initSearch } from "./search";

/** Application state. */
export interface AppState {
  wasm: BioFabricWasm;
  renderer: FabricRenderer;
  networkHandle: Handle | null;
  layoutHandle: Handle | null;
}

/**
 * Initialize the application: connect all UI components to the engine.
 */
export function initApp(wasm: BioFabricWasm, renderer: FabricRenderer): void {
  const state: AppState = {
    wasm,
    renderer,
    networkHandle: null,
    layoutHandle: null,
  };

  // TODO: Wire up all UI subsystems
  //
  // initFileLoader(state);   — file upload / drag-drop
  // initControls(state);     — zoom, pan, layout picker
  // initInfoPanel(state);    — node/link info on hover
  // initSearch(state);       — search bar
  //
  // Set up the render loop:
  //   1. On viewport change (pan/zoom), request new render data from WASM
  //   2. Upload render data to renderer
  //   3. Call renderer.draw()
  //
}
