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
        layout: &NetworkLayout,
        params: &RenderParams,
        palette: &super::color::ColorPalette,
    ) -> Self {
        let show_shadows = params.show_shadows;
        let vp = &params.viewport;
        let decimation = params.lod.decimation_factor(params.pixels_per_grid_unit);
        let total_cols = if show_shadows {
            layout.column_count
        } else {
            layout.column_count_no_shadows
        } as f32;

        // ---- PHASE 0: ANNOTATIONS ----

        let mut node_annotations = RectBatch::with_capacity(layout.node_annotations.len());
        for ann in layout.node_annotations.iter() {
            let y0 = ann.start as f64;
            let y1 = ann.end as f64;
            // Viewport cull on row range
            if y1 < vp.y || y0 > vp.bottom() {
                continue;
            }
            let color = parse_annotation_color(&ann.color);
            node_annotations.push(RectInstance {
                x: 0.0,
                y: ann.start as f32,
                w: total_cols,
                h: (ann.end - ann.start + 1) as f32,
                color,
            });
        }

        let link_ann_set = if show_shadows {
            &layout.link_annotations
        } else {
            &layout.link_annotations_no_shadows
        };
        let mut link_annotations = RectBatch::with_capacity(link_ann_set.len());
        for ann in link_ann_set.iter() {
            let x0 = ann.start as f64;
            let x1 = ann.end as f64;
            if x1 < vp.x || x0 > vp.right() {
                continue;
            }
            let color = parse_annotation_color(&ann.color);
            link_annotations.push(RectInstance {
                x: ann.start as f32,
                y: 0.0,
                w: (ann.end - ann.start + 1) as f32,
                h: layout.row_count as f32,
                color,
            });
        }

        // ---- PHASE 1: NODES ----

        let mut nodes = LineBatch::with_capacity(layout.nodes.len());
        for (i, (_nid, nl)) in layout.nodes.iter().enumerate() {
            if decimation > 1 && i % decimation != 0 {
                continue;
            }
            let (min_c, max_c, has) = if show_shadows {
                (nl.min_col, nl.max_col, nl.has_edges())
            } else {
                (nl.min_col_no_shadows, nl.max_col_no_shadows, nl.has_edges_no_shadows())
            };
            if !has {
                continue;
            }
            let row = nl.row as f64;
            let min_cf = min_c as f64;
            let max_cf = max_c as f64;
            if !vp.intersects_node(row, min_cf, max_cf) {
                continue;
            }
            // Clip to viewport
            let x0 = (min_cf.max(vp.x)) as f32;
            let x1 = (max_cf.min(vp.right())) as f32;
            let color = palette.get(nl.color_index);
            nodes.push(LineInstance {
                x0,
                y0: nl.row as f32,
                x1,
                y1: nl.row as f32,
                color,
            });
        }

        // ---- PHASE 2: LINKS ----

        let mut links = LineBatch::with_capacity(layout.links.len());
        for (i, ll) in layout.links.iter().enumerate() {
            if !show_shadows && ll.is_shadow {
                continue;
            }
            if decimation > 1 && i % decimation != 0 {
                continue;
            }
            let col = if show_shadows {
                ll.column
            } else {
                match ll.column_no_shadows {
                    Some(c) => c,
                    None => continue, // shadow link in no-shadow mode
                }
            };
            let col_f = col as f64;
            let top = ll.top_row() as f64;
            let bot = ll.bottom_row() as f64;
            if !vp.intersects_link(col_f, top, bot) {
                continue;
            }
            // Clip to viewport
            let y0 = (top.max(vp.y)) as f32;
            let y1 = (bot.min(vp.bottom())) as f32;
            let color = palette.get(ll.color_index);
            links.push(LineInstance {
                x0: col as f32,
                y0,
                x1: col as f32,
                y1,
                color,
            });
        }

        // ---- PHASE 3: LABELS (skipped for image export — no text rasterizer yet) ----

        Self {
            node_annotations,
            link_annotations,
            links,
            nodes,
            labels: TextBatch::new(),
        }
    }
}

/// Parse an annotation color string (hex like `"#RRGGBB"` or `"#RRGGBBAA"`)
/// into a [`FabricColor`]. Falls back to a semi-transparent gray if unparseable.
fn parse_annotation_color(hex: &str) -> FabricColor {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(200);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(200);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(200);
            FabricColor::rgba(r, g, b, 64)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(200);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(200);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(200);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(64);
            FabricColor::rgba(r, g, b, a)
        }
        _ => FabricColor::rgba(200, 200, 200, 64),
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
