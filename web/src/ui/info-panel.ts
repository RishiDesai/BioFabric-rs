/**
 * Info panel — shows details about the hovered/selected node or link.
 *
 * Displayed as a side panel on the right side of the canvas.
 */

import type { AppState } from "./app";

/**
 * Initialize the info panel.
 */
export function initInfoPanel(_state: AppState): void {
  // TODO: Implement info panel
  //
  // On mouse hover over canvas:
  //   1. Convert screen coordinates to grid coordinates
  //   2. Hit-test against nodes and links (via WASM or local quadtree)
  //   3. If a node/link is under the cursor, display its info:
  //      - Node: name, degree, row, column span, neighbors
  //      - Link: source, target, relation, column, edge type
  //   4. Highlight the hovered element in the renderer
  //
  // On click:
  //   1. Select the node/link
  //   2. Pin its info in the panel
  //   3. Optionally highlight all neighbors
  //
}
