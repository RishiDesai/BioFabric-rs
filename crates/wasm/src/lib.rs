//! WebAssembly bindings for BioFabric.
//!
//! This crate exposes the `biofabric-core` API to JavaScript via
//! `wasm-bindgen`. All heavy computation (parsing, layout, analysis)
//! runs in WASM; the web frontend only handles rendering and UI.
//!
//! ## Architecture
//!
//! - **Handles**: Large data (networks, layouts) live in WASM memory and are
//!   referenced by opaque `u32` handles. This avoids serializing megabytes of
//!   data on every call.
//!
//! - **Render data (zero-copy)**: Instead of serializing line segments to JSON,
//!   [`get_node_instances`] / [`get_link_instances`] return pointers into WASM
//!   linear memory. The JS side wraps these as `Float32Array` views and uploads
//!   directly to WebGL2 — no copies between Rust and the GPU.
//!
//! - **Metadata**: Small payloads (network info, scores, search results) are
//!   returned as JSON strings for convenience.

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// WASM state (stub)
// ---------------------------------------------------------------------------

/// Global WASM state for handle-based access (stub).
#[allow(dead_code)]
struct WasmState {
    networks: Vec<Option<biofabric_core::model::Network>>,
    layouts: Vec<Option<biofabric_core::layout::NetworkLayout>>,
    merged: Vec<Option<biofabric_core::alignment::MergedNetwork>>,
    render: Option<biofabric_core::render::RenderOutput>,
}

// ---------------------------------------------------------------------------
// Network loading
// ---------------------------------------------------------------------------

/// Load a network from file contents.
///
/// # Arguments
/// * `format` - File format: `"sif"`, `"gw"`, or `"json"`
/// * `data` - File contents as a string
///
/// # Returns
/// An opaque handle (u32) to the loaded network.
#[wasm_bindgen]
pub fn load_network(_format: &str, _data: &str) -> Result<u32, JsError> {
    // TODO: Implement network loading
    //
    // 1. Match format string to parser (sif::parse_string, gw::parse_string, json)
    // 2. Parse the data into a Network
    // 3. Store Network in a global slab/arena (e.g., Vec<Option<Network>>)
    // 4. Return the slab index as a handle
    //
    todo!("Implement WASM network loading")
}

/// Free a previously loaded network.
#[wasm_bindgen]
pub fn free_network(_handle: u32) {
    todo!("Implement WASM network free")
}

/// Get basic info about a loaded network (JSON).
///
/// Returns: `{ "node_count": N, "link_count": N, "relation_types": [...] }`
#[wasm_bindgen]
pub fn network_info(_handle: u32) -> Result<String, JsError> {
    todo!("Implement WASM network info")
}

// ---------------------------------------------------------------------------
// Layout computation
// ---------------------------------------------------------------------------

/// Compute a layout for a loaded network.
///
/// # Arguments
/// * `network_handle` - Handle from `load_network`
/// * `algorithm` - Layout algorithm name (e.g., `"default"`, `"similarity"`)
/// * `params_json` - Layout parameters as JSON string
///
/// # Returns
/// An opaque handle to the computed layout.
#[wasm_bindgen]
pub fn compute_layout(
    _network_handle: u32,
    _algorithm: &str,
    _params_json: &str,
) -> Result<u32, JsError> {
    // TODO: Implement layout computation
    //
    // 1. Retrieve network from slab by handle
    // 2. Parse params JSON into LayoutParams
    // 3. Select NodeLayout + EdgeLayout based on algorithm name
    // 4. Compute node order, then edge layout
    // 5. Store NetworkLayout in a layout slab, return handle
    //
    todo!("Implement WASM layout computation")
}

/// Free a previously computed layout.
#[wasm_bindgen]
pub fn free_layout(_handle: u32) {
    todo!("Implement WASM layout free")
}

/// Get layout dimensions (JSON).
///
/// Returns: `{ "row_count": N, "column_count": N }`
#[wasm_bindgen]
pub fn layout_dimensions(_layout_handle: u32) -> Result<String, JsError> {
    todo!("Implement WASM layout dimensions")
}

// ---------------------------------------------------------------------------
// Render data — zero-copy GPU instance buffers
// ---------------------------------------------------------------------------

/// Extract visible node instances for the current viewport.
///
/// Performs viewport culling and LOD filtering, then stores the result
/// in WASM linear memory. Returns a pointer and length that JS uses to
/// create a `Float32Array` view for direct GPU upload.
///
/// # Arguments
/// * `layout_handle` - Handle from `compute_layout`
/// * `vp_x`, `vp_y`, `vp_w`, `vp_h` - Viewport in grid coordinates
/// * `pixels_per_unit` - Zoom level (screen pixels per grid unit)
///
/// # Returns
/// Pointer (as u32) to the start of the f32 buffer in WASM memory.
/// Use [`node_instance_count`] to get the number of instances.
#[wasm_bindgen]
pub fn extract_render_data(
    _layout_handle: u32,
    _vp_x: f64,
    _vp_y: f64,
    _vp_w: f64,
    _vp_h: f64,
    _pixels_per_unit: f64,
    _canvas_width: u32,
    _canvas_height: u32,
) -> Result<(), JsError> {
    // TODO: Implement render extraction
    //
    // 1. Retrieve layout from slab
    // 2. Build RenderParams from viewport + zoom args
    // 3. Call RenderOutput::extract(layout, params, palette)
    // 4. Store the RenderOutput in a global (replacing the previous one)
    // 5. JS will then call node_instance_ptr/link_instance_ptr to get pointers
    //
    todo!("Implement WASM render extraction")
}

/// Pointer to the node instance f32 buffer (for Float32Array wrapping).
#[wasm_bindgen]
pub fn node_instance_ptr() -> *const f32 {
    // TODO: Return pointer to the stored RenderOutput.nodes.data
    todo!("Implement node instance pointer")
}

/// Number of f32s in the node instance buffer.
#[wasm_bindgen]
pub fn node_instance_len() -> usize {
    // TODO: Return stored RenderOutput.nodes.data.len()
    todo!("Implement node instance length")
}

/// Pointer to the link instance f32 buffer (for Float32Array wrapping).
#[wasm_bindgen]
pub fn link_instance_ptr() -> *const f32 {
    todo!("Implement link instance pointer")
}

/// Number of f32s in the link instance buffer.
#[wasm_bindgen]
pub fn link_instance_len() -> usize {
    todo!("Implement link instance length")
}

// ---------------------------------------------------------------------------
// Annotation rect instance buffers
// ---------------------------------------------------------------------------

/// Pointer to the node annotation rect f32 buffer.
#[wasm_bindgen]
pub fn node_annotation_ptr() -> *const f32 {
    todo!("Implement node annotation pointer")
}

/// Number of f32s in the node annotation buffer.
#[wasm_bindgen]
pub fn node_annotation_len() -> usize {
    todo!("Implement node annotation length")
}

/// Pointer to the link annotation rect f32 buffer.
#[wasm_bindgen]
pub fn link_annotation_ptr() -> *const f32 {
    todo!("Implement link annotation pointer")
}

/// Number of f32s in the link annotation buffer.
#[wasm_bindgen]
pub fn link_annotation_len() -> usize {
    todo!("Implement link annotation length")
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

/// Hit-test at a grid coordinate (JSON result).
///
/// # Arguments
/// * `layout_handle` - Handle from `compute_layout`
/// * `grid_x` - X coordinate in grid space (column)
/// * `grid_y` - Y coordinate in grid space (row)
/// * `tolerance` - Hit tolerance in grid units
/// * `show_shadows` - Whether shadow links are shown
///
/// # Returns
/// JSON: `{ "nodes": [{ "id": "...", "row": N }], "links": [{ "column": N, ... }] }`
#[wasm_bindgen]
pub fn hit_test(
    _layout_handle: u32,
    _grid_x: f64,
    _grid_y: f64,
    _tolerance: f64,
    _show_shadows: bool,
) -> Result<String, JsError> {
    // TODO: Implement hit testing
    //
    // 1. Retrieve layout from slab by handle
    // 2. Build or cache a HitIndex for the layout
    // 3. Call HitIndex::hit_test(grid_x, grid_y, tolerance)
    // 4. Serialize HitResult to JSON
    //
    todo!("Implement WASM hit testing")
}

// ---------------------------------------------------------------------------
// Alignment
// ---------------------------------------------------------------------------

/// Load an alignment and merge two networks.
///
/// # Arguments
/// * `g1_format` / `g1_data` - First network (smaller)
/// * `g2_format` / `g2_data` - Second network (larger)
/// * `align_data` - Alignment file contents
///
/// # Returns
/// Handle to the merged alignment network.
#[wasm_bindgen]
pub fn load_alignment(
    _g1_format: &str,
    _g1_data: &str,
    _g2_format: &str,
    _g2_data: &str,
    _align_data: &str,
) -> Result<u32, JsError> {
    // TODO: Implement alignment loading
    //
    // 1. Parse both networks
    // 2. Parse alignment file
    // 3. MergedNetwork::from_alignment(...)
    // 4. Store MergedNetwork in slab, return handle
    //
    todo!("Implement WASM alignment loading")
}

/// Compute alignment quality scores (JSON).
///
/// Returns: `{ "ec": N, "s3": N, "ics": N, "nc": N|null, ... }`
#[wasm_bindgen]
pub fn compute_alignment_scores(_alignment_handle: u32) -> Result<String, JsError> {
    todo!("Implement WASM alignment scoring")
}

// ---------------------------------------------------------------------------
// Search and query
// ---------------------------------------------------------------------------

/// Search for nodes by name prefix (JSON array of matching node IDs).
#[wasm_bindgen]
pub fn search_nodes(_network_handle: u32, _query: &str) -> Result<String, JsError> {
    // TODO: Implement node search
    //
    // 1. Retrieve network by handle
    // 2. Filter node IDs by case-insensitive prefix match
    // 3. Return as JSON array (small payload, JSON is fine)
    //
    todo!("Implement WASM node search")
}

/// Get detailed info about a specific node (JSON).
///
/// Returns: `{ "id": "...", "degree": N, "neighbors": [...], "row": N, ... }`
#[wasm_bindgen]
pub fn get_node_info(
    _network_handle: u32,
    _layout_handle: u32,
    _node_id: &str,
) -> Result<String, JsError> {
    todo!("Implement WASM node info")
}
