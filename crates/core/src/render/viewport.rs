//! Viewport culling and level-of-detail (LOD).
//!
//! Given the current camera position and zoom level, determines:
//! 1. Which nodes and links are visible (culling)
//! 2. What level of detail to render at (LOD)
//!
//! ## LOD Strategy
//!
//! At extreme zoom-out, millions of sub-pixel lines are wasteful to draw
//! individually. Rather than a separate "bucket renderer" (as in the Java
//! implementation), we apply progressive culling:
//!
//! | Zoom level          | Strategy                                      |
//! |---------------------|-----------------------------------------------|
//! | **Full**            | Render every visible node and link             |
//! | **Culled**          | Skip shadow links and very short node spans    |
//! | **Sparse**          | Sample every Nth row/column, skip tiny spans   |

use serde::{Deserialize, Serialize};

/// Axis-aligned viewport rectangle in BioFabric grid coordinates.
///
/// The grid coordinate system has:
/// - X axis = columns (link positions), increasing right
/// - Y axis = rows (node positions), increasing down
/// - One grid unit = one row or one column
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    /// Left edge (column).
    pub x: f64,
    /// Top edge (row).
    pub y: f64,
    /// Width in grid units.
    pub width: f64,
    /// Height in grid units.
    pub height: f64,
}

impl Viewport {
    /// Create a new viewport.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Right edge.
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Bottom edge.
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Check if a horizontal span (node) intersects this viewport.
    ///
    /// A node at `row` spanning columns `[min_col, max_col]` is visible if:
    /// - Its row is within the viewport's vertical range
    /// - Its column span overlaps the viewport's horizontal range
    pub fn intersects_node(&self, row: f64, min_col: f64, max_col: f64) -> bool {
        row >= self.y
            && row <= self.bottom()
            && max_col >= self.x
            && min_col <= self.right()
    }

    /// Check if a vertical span (link) intersects this viewport.
    ///
    /// A link at `column` spanning rows `[top_row, bottom_row]` is visible if:
    /// - Its column is within the viewport's horizontal range
    /// - Its row span overlaps the viewport's vertical range
    pub fn intersects_link(&self, column: f64, top_row: f64, bottom_row: f64) -> bool {
        column >= self.x
            && column <= self.right()
            && bottom_row >= self.y
            && top_row <= self.bottom()
    }
}

/// Level-of-detail setting, derived from the zoom level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LodLevel {
    /// Render every visible element. Used when zoomed in enough that
    /// individual lines are clearly distinguishable.
    Full,

    /// Drop shadow links and node spans shorter than a threshold.
    /// Used at moderate zoom-out where shadows just add noise.
    Culled,

    /// Aggressive decimation: sample every Nth element.
    /// Used at extreme zoom-out where the display is mostly solid color.
    Sparse,
}

impl LodLevel {
    /// Determine the appropriate LOD level from the current zoom.
    ///
    /// `pixels_per_grid_unit` is how many screen pixels correspond to one
    /// BioFabric grid unit (row or column).
    pub fn from_zoom(pixels_per_grid_unit: f64) -> Self {
        if pixels_per_grid_unit >= 2.0 {
            LodLevel::Full
        } else if pixels_per_grid_unit >= 0.2 {
            LodLevel::Culled
        } else {
            LodLevel::Sparse
        }
    }

    /// For [`Sparse`](LodLevel::Sparse) mode, how many elements to skip.
    ///
    /// Returns 1 (no skip) for Full/Culled, or a decimation factor for Sparse.
    pub fn decimation_factor(&self, pixels_per_grid_unit: f64) -> usize {
        match self {
            LodLevel::Full | LodLevel::Culled => 1,
            LodLevel::Sparse => {
                // At 0.1 px/unit we need ~10x decimation, at 0.01 ~100x, etc.
                (1.0 / pixels_per_grid_unit).ceil().max(1.0) as usize
            }
        }
    }
}

/// Parameters for a render extraction pass.
#[derive(Debug, Clone)]
pub struct RenderParams {
    /// Current viewport in grid coordinates.
    pub viewport: Viewport,

    /// Screen pixels per grid unit (i.e., zoom level).
    pub pixels_per_grid_unit: f64,

    /// Derived LOD level.
    pub lod: LodLevel,

    /// Canvas width in physical pixels.
    pub canvas_width: u32,

    /// Canvas height in physical pixels.
    pub canvas_height: u32,

    /// Whether to display shadow links.
    ///
    /// When `true`, use `LinkLayout::column` and `NodeLayout::{min_col, max_col}`.
    /// When `false`, skip shadow links entirely and use
    /// `LinkLayout::column_no_shadows` and `NodeLayout::{min_col_no_shadows, max_col_no_shadows}`.
    ///
    /// This corresponds to `FabricDisplayOptions.displayShadows_` in the Java implementation.
    pub show_shadows: bool,
}

impl RenderParams {
    /// Create render parameters from a viewport and zoom level.
    pub fn new(
        viewport: Viewport,
        pixels_per_grid_unit: f64,
        canvas_width: u32,
        canvas_height: u32,
        show_shadows: bool,
    ) -> Self {
        Self {
            viewport,
            pixels_per_grid_unit,
            lod: LodLevel::from_zoom(pixels_per_grid_unit),
            canvas_width,
            canvas_height,
            show_shadows,
        }
    }
}
