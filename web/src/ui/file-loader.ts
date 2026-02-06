/**
 * File upload and drag-drop handling.
 *
 * Supports loading network files (.sif, .gw, .json) and alignment
 * files (.align) via file input or drag-and-drop onto the canvas.
 */

import type { AppState } from "./app";

/**
 * Initialize file loading UI (upload button + drag-drop).
 */
export function initFileLoader(_state: AppState): void {
  // TODO: Implement file loading
  //
  // 1. Create a hidden <input type="file"> element
  // 2. Add "Open File" button to toolbar that triggers the input
  // 3. Handle drag-and-drop on the canvas container
  // 4. On file selected:
  //    a. Read file contents as text
  //    b. Detect format from extension
  //    c. Call state.wasm.loadNetwork(format, data)
  //    d. Compute default layout
  //    e. Request render data and draw
  //
  // For alignment:
  //    a. Prompt for G1, G2, and .align files
  //    b. Call state.wasm.loadAlignment(...)
  //
}
