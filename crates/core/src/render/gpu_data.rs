//! GPU instance buffer layout and extraction.
//!
//! Produces flat `Vec<f32>` buffers that map directly to WebGL2 / wgpu
//! instance attribute arrays. No JSON serialization — the data can be
//! transferred to the GPU (or across the WASM boundary) as a raw byte slice.
//!
//! ## Instance Layout
//!
//! Each line instance is **8 consecutive `f32`s** (32 bytes):
//!
//! | Offset | Field | Description                          |
//! |--------|-------|--------------------------------------|
//! | 0      | x0    | Start X (grid coords)                |
//! | 1      | y0    | Start Y (grid coords)                |
//! | 2      | x1    | End X (grid coords)                  |
//! | 3      | y1    | End Y (grid coords)                  |
//! | 4      | r     | Red   (0.0 – 1.0)                    |
//! | 5      | g     | Green (0.0 – 1.0)                    |
//! | 6      | b     | Blue  (0.0 – 1.0)                    |
//! | 7      | a     | Alpha (0.0 – 1.0)                    |
//!
//! ## Usage (WASM → WebGL2)
//!
//! ```text
//! Rust:  let batch = LineBatch::extract(&layout, &params);
//! WASM:  expose batch.data.as_ptr() and batch.data.len()
//! JS:    new Float32Array(wasmMemory.buffer, ptr, len)
//! WebGL: gl.bufferData(gl.ARRAY_BUFFER, float32array, gl.DYNAMIC_DRAW)
//! ```
//!
//! Zero copies between Rust and the GPU.

use super::color::FabricColor;
use super::viewport::{LodLevel, RenderParams};
use crate::layout::result::NetworkLayout;
use crate::model::AnnotationSet;

/// Number of f32s per line instance.
pub const FLOATS_PER_INSTANCE: usize = 8;

/// Byte stride per instance (8 × 4 bytes = 32).
pub const BYTES_PER_INSTANCE: usize = FLOATS_PER_INSTANCE * 4;

/// A single line instance (convenience struct for construction).
///
/// This is an intermediate type. For GPU upload, instances are packed
/// into a flat `Vec<f32>` inside [`LineBatch`].
#[derive(Debug, Clone, Copy)]
pub struct LineInstance {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub color: FabricColor,
}

impl LineInstance {
    /// Push this instance's 8 floats into a flat buffer.
    #[inline]
    pub fn pack_into(&self, buf: &mut Vec<f32>) {
        let [r, g, b, a] = self.color.to_f32_array();
        buf.extend_from_slice(&[self.x0, self.y0, self.x1, self.y1, r, g, b, a]);
    }
}

/// A batch of line instances, ready for GPU upload.
///
/// Contains a flat `Vec<f32>` where every 8 consecutive floats describe
/// one line instance (see module-level docs for the layout).
#[derive(Debug, Clone)]
pub struct LineBatch {
    /// Packed instance data. Length is always a multiple of [`FLOATS_PER_INSTANCE`].
    pub data: Vec<f32>,
}

impl LineBatch {
    /// Create an empty batch with pre-allocated capacity.
    pub fn with_capacity(instance_count: usize) -> Self {
        Self {
            data: Vec::with_capacity(instance_count * FLOATS_PER_INSTANCE),
        }
    }

    /// Number of line instances in this batch.
    pub fn instance_count(&self) -> usize {
        self.data.len() / FLOATS_PER_INSTANCE
    }

    /// Push a single line instance.
    pub fn push(&mut self, instance: LineInstance) {
        instance.pack_into(&mut self.data);
    }

    /// Raw f32 slice (for WASM pointer export or GPU upload).
    pub fn as_f32_slice(&self) -> &[f32] {
        &self.data
    }

    /// Raw byte slice (safe cast via `bytemuck`).
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}

/// Number of f32s per rectangle instance.
pub const FLOATS_PER_RECT: usize = 8;

/// A filled rectangle instance (for annotations).
///
/// Each annotation rectangle is **8 consecutive `f32`s**:
///
/// | Offset | Field | Description                          |
/// |--------|-------|--------------------------------------|
/// | 0      | x     | Left edge (grid coords)              |
/// | 1      | y     | Top edge (grid coords)               |
/// | 2      | w     | Width (grid units)                   |
/// | 3      | h     | Height (grid units)                  |
/// | 4      | r     | Red   (0.0 – 1.0)                    |
/// | 5      | g     | Green (0.0 – 1.0)                    |
/// | 6      | b     | Blue  (0.0 – 1.0)                    |
/// | 7      | a     | Alpha (0.0 – 1.0)                    |
#[derive(Debug, Clone, Copy)]
pub struct RectInstance {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: FabricColor,
}

impl RectInstance {
    /// Push this instance's 8 floats into a flat buffer.
    #[inline]
    pub fn pack_into(&self, buf: &mut Vec<f32>) {
        let [r, g, b, a] = self.color.to_f32_array();
        buf.extend_from_slice(&[self.x, self.y, self.w, self.h, r, g, b, a]);
    }
}

/// A batch of rectangle instances (for annotation rendering).
#[derive(Debug, Clone)]
pub struct RectBatch {
    /// Packed instance data. Length is always a multiple of [`FLOATS_PER_RECT`].
    pub data: Vec<f32>,
}

impl RectBatch {
    /// Create an empty batch with pre-allocated capacity.
    pub fn with_capacity(instance_count: usize) -> Self {
        Self {
            data: Vec::with_capacity(instance_count * FLOATS_PER_RECT),
        }
    }

    /// Create an empty batch.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Number of rectangle instances in this batch.
    pub fn instance_count(&self) -> usize {
        self.data.len() / FLOATS_PER_RECT
    }

    /// Push a single rectangle instance.
    pub fn push(&mut self, instance: RectInstance) {
        instance.pack_into(&mut self.data);
    }

    /// Raw f32 slice.
    pub fn as_f32_slice(&self) -> &[f32] {
        &self.data
    }
}

impl Default for RectBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Render output: annotations behind links behind nodes, with optional labels.
///
/// ## Draw order (back to front)
///
/// 1. `node_annotations` — colored rectangles for node group ranges
/// 2. `link_annotations` — colored rectangles for link group ranges
/// 3. `links` — vertical edge lines
/// 4. `nodes` — horizontal node lines
/// 5. `labels` — text labels (drawn on top of everything)
///
/// ## Usage
///
/// For GPU rendering (WebGL2/wgpu):
/// - Upload `node_annotations`, `link_annotations`, `links`, `nodes` to GPU
///   buffers and render with instanced draw calls.
/// - Render `labels` via a separate text rendering path (Canvas2D overlay,
///   glyph atlas, etc.).
///
/// For CPU rendering (image export):
/// - Rasterize all batches to a pixel buffer, then encode as PNG.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    /// Node annotation rectangles (drawn first = furthest back).
    pub node_annotations: RectBatch,

    /// Link annotation rectangles (drawn second).
    pub link_annotations: RectBatch,

    /// Vertical link lines (drawn third).
    pub links: LineBatch,

    /// Horizontal node lines (drawn fourth = on top).
    pub nodes: LineBatch,

    /// Text labels (node names, link relation labels).
    ///
    /// Only populated when `DisplayOptions::show_node_labels` or
    /// `show_link_labels` is true. For CLI image export, labels are
    /// rasterized into the output image. For interactive rendering,
    /// labels are typically rendered as a separate overlay.
    pub labels: TextBatch,
}

impl RenderOutput {
    /// Create an empty render output (no instances).
    ///
    /// Useful as a placeholder when the full render extraction pipeline
    /// is not yet implemented, or when exporting a background-only image.
    pub fn empty() -> Self {
        Self {
            node_annotations: RectBatch::new(),
            link_annotations: RectBatch::new(),
            links: LineBatch::with_capacity(0),
            nodes: LineBatch::with_capacity(0),
            labels: TextBatch::new(),
        }
    }

    /// Extract visible instances from a computed layout.
    ///
    /// Performs viewport culling and LOD filtering, then packs the
    /// surviving elements into GPU-ready batches.
    ///
    /// ## Draw order
    ///
    /// 1. Node annotation rectangles (background)
    /// 2. Link annotation rectangles
    /// 3. Link lines (vertical)
    /// 4. Node lines (horizontal, on top)
    ///
    /// ## Shadow-aware rendering
    ///
    /// When `params.show_shadows` is `true`:
    /// - All links are candidates (shadow + regular)
    /// - Uses `link.column` and `node.{min_col, max_col}` for positions
    /// - Uses `layout.link_annotations` for link annotation ranges
    ///
    /// When `params.show_shadows` is `false`:
    /// - Shadow links are skipped entirely
    /// - Uses `link.column_no_shadows` and `node.{min_col_no_shadows, max_col_no_shadows}`
    /// - Uses `layout.link_annotations_no_shadows` for link annotation ranges
    pub fn extract(
        _layout: &NetworkLayout,
        _params: &RenderParams,
        _palette: &super::color::ColorPalette,
    ) -> Self {
        // TODO: Implement render extraction
        //
        // let show_shadows = params.show_shadows;
        //
        // PHASE 0 — ANNOTATIONS
        //
        // 0a. For each node annotation in layout.node_annotations:
        //     - The annotation spans rows [start, end]
        //     - Full width of the layout (0..column_count or 0..column_count_no_shadows)
        //     - Parse annotation.color as FabricColor (with transparency)
        //     - Viewport cull: skip if row range doesn't overlap viewport
        //     - Push RectInstance { x: 0, y: start, w: total_cols, h: end-start+1, color }
        //
        // 0b. For each link annotation:
        //     - Select annotation set: link_annotations or link_annotations_no_shadows
        //     - The annotation spans columns [start, end]
        //     - Full height of the layout (0..row_count)
        //     - Viewport cull: skip if column range doesn't overlap viewport
        //     - Push RectInstance { x: start, y: 0, w: end-start+1, h: row_count, color }
        //
        // PHASE 1 — NODES
        //
        // 1. For each node in layout:
        //    a. Select column span based on shadow mode
        //    b. Viewport cull
        //    c. LOD cull
        //    d. Clip the horizontal span to the viewport
        //    e. Look up color from palette using node.color_index
        //    f. Push LineInstance { x0: clipped_min, y0: row, x1: clipped_max, y1: row, color }
        //
        // PHASE 2 — LINKS
        //
        // 2. For each link in layout:
        //    a. If !show_shadows && link.is_shadow: skip
        //    b. Select column (shadow-on or shadow-off)
        //    c. Viewport cull
        //    d. LOD cull
        //    e. Clip the vertical span to the viewport
        //    f. Look up color from palette
        //    g. Push LineInstance { x0: col, y0: clipped_top, x1: col, y1: clipped_bottom, color }
        //
        // PHASE 3 — LABELS (when display options enable them)
        //
        // 3. If show_node_labels:
        //    a. For each visible node, create a TextLabel at (min_col - label_offset, row)
        //    b. Font size ~ 0.8 grid units (scales with zoom)
        //
        // 4. If show_link_labels:
        //    a. For each visible link, create a TextLabel at (col, top_row - label_offset)
        //
        // PHASE 4 — RETURN
        //
        // Return RenderOutput { node_annotations, link_annotations, links, nodes, labels }
        //
        todo!("Implement render extraction with annotations + viewport culling + LOD + shadow awareness + labels")
    }
}

/// A text label positioned in grid space (for node/link name rendering).
///
/// Labels are extracted alongside line instances during render extraction.
/// The renderer can display these as HTML overlays, Canvas2D text, or
/// a glyph atlas — the core crate is agnostic to the rendering method.
///
/// ## References
///
/// - Java: `PaintCacheSmall.TextPath`, `PaintCacheSmall.GlyphPath`
#[derive(Debug, Clone)]
pub struct TextLabel {
    /// X position in grid coordinates (column).
    pub x: f32,
    /// Y position in grid coordinates (row).
    pub y: f32,
    /// The label text.
    pub text: String,
    /// Font size hint (in grid units; the renderer scales to screen pixels).
    pub font_size: f32,
    /// Text color.
    pub color: FabricColor,
    /// Whether this label is for a node (true) or a link (false).
    pub is_node_label: bool,
}

/// A batch of text labels, ready for rendering.
///
/// Unlike `LineBatch` and `RectBatch`, text labels are not packed into
/// a flat GPU buffer because text rendering typically uses a different
/// code path (Canvas2D overlay, glyph atlas, etc.).
#[derive(Debug, Clone, Default)]
pub struct TextBatch {
    /// All labels in this batch.
    pub labels: Vec<TextLabel>,
}

impl TextBatch {
    /// Create an empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a batch with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            labels: Vec::with_capacity(capacity),
        }
    }

    /// Push a label.
    pub fn push(&mut self, label: TextLabel) {
        self.labels.push(label);
    }

    /// Number of labels.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}
