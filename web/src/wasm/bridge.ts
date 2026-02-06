/**
 * WASM Bridge — typed wrapper around the biofabric-wasm module.
 *
 * Loads the WASM binary and provides a typed TypeScript API that mirrors
 * the #[wasm_bindgen] exports from crates/wasm/src/lib.rs.
 *
 * ## Zero-copy rendering
 *
 * For GPU data, the bridge does NOT serialize to JSON. Instead:
 * 1. Rust calls `extract_render_data(...)` which fills buffers in WASM memory
 * 2. `getNodeInstances()` / `getLinkInstances()` return `Float32Array` views
 *    directly into WASM linear memory — zero copies
 * 3. The renderer uploads these arrays to WebGL2 via `bufferData()`
 */

import type {
  NetworkInfo,
  LayoutDimensions,
  AlignmentScores,
  NodeInfo,
} from "../types";

/** Handle type — opaque u32 referencing data stored in WASM memory. */
export type Handle = number;

/** The raw WASM module exports (before wrapping). */
interface WasmExports {
  // We'll fill this from the wasm-pack output.
  memory: WebAssembly.Memory;
  load_network(format: string, data: string): number;
  free_network(handle: number): void;
  network_info(handle: number): string;
  compute_layout(network_handle: number, algorithm: string, params_json: string): number;
  free_layout(handle: number): void;
  layout_dimensions(layout_handle: number): string;
  extract_render_data(
    layout_handle: number,
    vp_x: number, vp_y: number, vp_w: number, vp_h: number,
    pixels_per_unit: number,
    canvas_width: number, canvas_height: number,
  ): void;
  node_instance_ptr(): number;
  node_instance_len(): number;
  link_instance_ptr(): number;
  link_instance_len(): number;
  load_alignment(
    g1_format: string, g1_data: string,
    g2_format: string, g2_data: string,
    align_data: string,
  ): number;
  compute_alignment_scores(alignment_handle: number): string;
  search_nodes(network_handle: number, query: string): string;
  get_node_info(network_handle: number, layout_handle: number, node_id: string): string;
}

/** The typed WASM API exposed to the rest of the app. */
export interface BioFabricWasm {
  // -- Network --
  loadNetwork(format: string, data: string): Handle;
  freeNetwork(handle: Handle): void;
  networkInfo(handle: Handle): NetworkInfo;

  // -- Layout --
  computeLayout(networkHandle: Handle, algorithm: string, paramsJson: string): Handle;
  freeLayout(handle: Handle): void;
  layoutDimensions(handle: Handle): LayoutDimensions;

  // -- Render (zero-copy) --
  /**
   * Extract visible instances for the current viewport.
   * After calling this, use `getNodeInstances()` / `getLinkInstances()` to
   * get Float32Array views into WASM memory for direct GPU upload.
   */
  extractRenderData(
    layoutHandle: Handle,
    vpX: number, vpY: number, vpW: number, vpH: number,
    pixelsPerUnit: number,
    canvasWidth: number, canvasHeight: number,
  ): void;

  /** Float32Array view into WASM memory for node instances. */
  getNodeInstances(): Float32Array;
  /** Float32Array view into WASM memory for link instances. */
  getLinkInstances(): Float32Array;

  // -- Alignment --
  loadAlignment(
    g1Format: string, g1Data: string,
    g2Format: string, g2Data: string,
    alignData: string,
  ): Handle;
  computeAlignmentScores(alignmentHandle: Handle): AlignmentScores;

  // -- Search / Query --
  searchNodes(networkHandle: Handle, query: string): string[];
  getNodeInfo(networkHandle: Handle, layoutHandle: Handle, nodeId: string): NodeInfo;
}

/**
 * Initialize the WASM module and return the typed API.
 *
 * Call this once at startup. The returned object is the bridge that all
 * other modules use to communicate with the Rust core.
 */
export async function initWasm(): Promise<BioFabricWasm> {
  // TODO: Implement WASM initialization
  //
  // 1. Import the wasm-pack generated JS glue:
  //      import init, * as wasm from "./pkg/biofabric_wasm.js";
  //      await init();
  //
  // 2. Wrap each export in a typed function. For zero-copy render data:
  //
  //      getNodeInstances() {
  //        const ptr = wasm.node_instance_ptr();
  //        const len = wasm.node_instance_len();
  //        return new Float32Array(wasm.memory.buffer, ptr, len);
  //      }
  //
  //    IMPORTANT: the Float32Array is a VIEW into WASM memory. It becomes
  //    invalid if WASM memory grows (e.g., during another allocation).
  //    Always call getNodeInstances() fresh before each GPU upload — do
  //    not cache the Float32Array across frames.
  //
  // 3. Return the BioFabricWasm object
  //
  throw new Error("WASM module not yet built. Run: npm run wasm:build");
}
