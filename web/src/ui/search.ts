/**
 * Search bar — find nodes by name.
 *
 * Provides a text input with autocomplete suggestions, scrolling
 * the viewport to center on the selected node.
 */

import type { AppState } from "./app";

/**
 * Initialize the search bar.
 */
export function initSearch(_state: AppState): void {
  // TODO: Implement search
  //
  // 1. Create a search input in the toolbar
  // 2. On input change (debounced):
  //    a. Call state.wasm.searchNodes(handle, query)
  //    b. Display matching node names as suggestions
  // 3. On suggestion selected:
  //    a. Get node's row from layout
  //    b. Pan camera to center on that row
  //    c. Highlight the node
  //
}
