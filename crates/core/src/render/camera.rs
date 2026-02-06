//! Camera/zoom state and coordinate transforms.
//!
//! Manages the mapping between **grid coordinates** (rows and columns) and
//! **screen coordinates** (pixels). This is the single source of truth for
//! what portion of the layout is visible at what zoom level.
//!
//! ## Coordinate Systems
//!
//! - **Grid coordinates**: the BioFabric row/column space. One grid unit =
//!   one row = one column. Origin (0, 0) is top-left.
//! - **Screen coordinates**: pixel space of the output canvas. Origin (0, 0)
//!   is top-left.
//!
//! ## Usage (CLI image export)
//!
//! ```rust,ignore
//! let layout = /* computed NetworkLayout */;
//! let mut camera = Camera::for_canvas(1920, 1080);
//! camera.zoom_to_fit(&layout, true);  // fit entire network
//! let params = camera.render_params(true);
//! let output = RenderOutput::extract(&layout, &params, &palette);
//! ```
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.ui.display.BasicZoomTargetSupport`
//! - Java: `org.systemsbiology.biofabric.cmd.ZoomCommandSupport`

use super::display_options::DisplayOptions;
use super::viewport::{RenderParams, Viewport};
use crate::layout::result::NetworkLayout;

/// Camera state: center position, zoom level, and canvas size.
///
/// All navigation operations (pan, zoom, zoom-to-fit, zoom-to-selection)
/// modify the camera, which then produces a [`Viewport`] for render
/// extraction.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Center of the viewport in grid coordinates (column, row).
    pub center_x: f64,
    pub center_y: f64,

    /// Zoom level: screen pixels per grid unit.
    ///
    /// - `zoom = 10.0` → each row/column is 10px apart on screen
    /// - `zoom = 0.1` → 10 rows/columns per pixel (extreme zoom-out)
    pub zoom: f64,

    /// Output canvas size in physical pixels.
    pub canvas_width: u32,
    pub canvas_height: u32,
}

impl Camera {
    /// Create a camera for a canvas of the given size.
    ///
    /// Starts centered at origin with zoom = 1.0.
    pub fn for_canvas(width: u32, height: u32) -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            zoom: 1.0,
            canvas_width: width,
            canvas_height: height,
        }
    }

    /// Compute the current viewport in grid coordinates.
    pub fn viewport(&self) -> Viewport {
        let half_w = (self.canvas_width as f64) / (2.0 * self.zoom);
        let half_h = (self.canvas_height as f64) / (2.0 * self.zoom);
        Viewport::new(
            self.center_x - half_w,
            self.center_y - half_h,
            half_w * 2.0,
            half_h * 2.0,
        )
    }

    /// Build [`RenderParams`] from the current camera state.
    ///
    /// This is the primary entry point for render extraction.
    pub fn render_params(&self, show_shadows: bool) -> RenderParams {
        RenderParams::new(
            self.viewport(),
            self.zoom,
            self.canvas_width,
            self.canvas_height,
            show_shadows,
        )
    }

    /// Build [`RenderParams`] from the current camera state and display options.
    ///
    /// Convenience method that reads the shadow flag from [`DisplayOptions`].
    pub fn render_params_with_options(&self, opts: &DisplayOptions) -> RenderParams {
        self.render_params(opts.show_shadows)
    }

    // =========================================================================
    // Zoom operations
    // =========================================================================

    /// Adjust zoom to fit the entire layout within the canvas.
    ///
    /// Adds a small margin (default: 2% on each side) so the outermost
    /// elements aren't flush against the edge.
    ///
    /// # Arguments
    ///
    /// * `layout` — The computed network layout
    /// * `show_shadows` — Whether to fit the shadow-on or shadow-off extents
    pub fn zoom_to_fit(&mut self, layout: &NetworkLayout, show_shadows: bool) {
        let cols = if show_shadows {
            layout.column_count
        } else {
            layout.column_count_no_shadows
        };

        if layout.row_count == 0 || cols == 0 {
            return;
        }

        let margin = 0.02;
        let grid_w = cols as f64;
        let grid_h = layout.row_count as f64;

        let zoom_x = self.canvas_width as f64 / (grid_w * (1.0 + 2.0 * margin));
        let zoom_y = self.canvas_height as f64 / (grid_h * (1.0 + 2.0 * margin));

        self.zoom = zoom_x.min(zoom_y);
        self.center_x = grid_w / 2.0;
        self.center_y = grid_h / 2.0;
    }

    /// Zoom to fit a specific rectangular region in grid coordinates.
    ///
    /// Useful for zoom-to-selection or zoom-to-annotation.
    pub fn zoom_to_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let margin = 0.05;
        let zoom_x = self.canvas_width as f64 / (width * (1.0 + 2.0 * margin));
        let zoom_y = self.canvas_height as f64 / (height * (1.0 + 2.0 * margin));

        self.zoom = zoom_x.min(zoom_y);
        self.center_x = x + width / 2.0;
        self.center_y = y + height / 2.0;
    }

    /// Zoom to show a specific node's full horizontal extent.
    ///
    /// Centers vertically on the node's row and horizontally on its span.
    pub fn zoom_to_node(&mut self, layout: &NetworkLayout, node_id: &crate::model::NodeId, show_shadows: bool) {
        if let Some(node) = layout.get_node(node_id) {
            let (min_col, max_col) = if show_shadows {
                (node.min_col, node.max_col)
            } else {
                (node.min_col_no_shadows, node.max_col_no_shadows)
            };

            if min_col > max_col {
                return; // no edges in this mode
            }

            let width = (max_col - min_col + 1) as f64;
            // Show some vertical context around the node
            let context_rows = (layout.row_count as f64 * 0.1).max(5.0);
            self.zoom_to_rect(
                min_col as f64,
                (node.row as f64 - context_rows).max(0.0),
                width,
                context_rows * 2.0,
            );
        }
    }

    /// Multiply the current zoom level.
    ///
    /// `factor > 1.0` zooms in, `factor < 1.0` zooms out.
    pub fn zoom_by(&mut self, factor: f64) {
        self.zoom = (self.zoom * factor).max(1e-6);
    }

    // =========================================================================
    // Pan operations
    // =========================================================================

    /// Pan the camera by the given offset in **screen pixels**.
    pub fn pan_by_pixels(&mut self, dx_px: f64, dy_px: f64) {
        self.center_x -= dx_px / self.zoom;
        self.center_y -= dy_px / self.zoom;
    }

    /// Pan the camera by the given offset in **grid units**.
    pub fn pan_by_grid(&mut self, dx: f64, dy: f64) {
        self.center_x += dx;
        self.center_y += dy;
    }

    /// Center the camera on a specific grid point.
    pub fn center_on(&mut self, x: f64, y: f64) {
        self.center_x = x;
        self.center_y = y;
    }

    // =========================================================================
    // Coordinate transforms
    // =========================================================================

    /// Convert a screen-space point to grid coordinates.
    ///
    /// Used for mouse interaction: map a click position to a row/column.
    pub fn screen_to_grid(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let vp = self.viewport();
        let grid_x = vp.x + screen_x / self.zoom;
        let grid_y = vp.y + screen_y / self.zoom;
        (grid_x, grid_y)
    }

    /// Convert a grid-space point to screen coordinates.
    pub fn grid_to_screen(&self, grid_x: f64, grid_y: f64) -> (f64, f64) {
        let vp = self.viewport();
        let screen_x = (grid_x - vp.x) * self.zoom;
        let screen_y = (grid_y - vp.y) * self.zoom;
        (screen_x, screen_y)
    }

    /// Size of one grid unit in screen pixels at the current zoom.
    pub fn grid_unit_size_px(&self) -> f64 {
        self.zoom
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::for_canvas(1920, 1080)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_at_default() {
        let cam = Camera::for_canvas(1000, 500);
        let vp = cam.viewport();
        // zoom = 1.0, center = (0, 0)
        // half_w = 500, half_h = 250
        assert!((vp.x - (-500.0)).abs() < 1e-9);
        assert!((vp.y - (-250.0)).abs() < 1e-9);
        assert!((vp.width - 1000.0).abs() < 1e-9);
        assert!((vp.height - 500.0).abs() < 1e-9);
    }

    #[test]
    fn test_zoom_to_fit() {
        let mut cam = Camera::for_canvas(1000, 500);
        let mut layout = NetworkLayout::new();
        layout.row_count = 100;
        layout.column_count = 200;
        layout.column_count_no_shadows = 150;

        cam.zoom_to_fit(&layout, true);

        // Center should be at middle of the layout
        assert!((cam.center_x - 100.0).abs() < 1e-9);
        assert!((cam.center_y - 50.0).abs() < 1e-9);

        // The viewport should contain the full layout
        let vp = cam.viewport();
        assert!(vp.x <= 0.0, "viewport should start at or before column 0");
        assert!(vp.y <= 0.0, "viewport should start at or before row 0");
        assert!(vp.right() >= 200.0, "viewport should reach past last column");
        assert!(vp.bottom() >= 100.0, "viewport should reach past last row");
    }

    #[test]
    fn test_screen_to_grid_roundtrip() {
        let mut cam = Camera::for_canvas(800, 600);
        cam.center_x = 50.0;
        cam.center_y = 30.0;
        cam.zoom = 4.0;

        let (gx, gy) = cam.screen_to_grid(400.0, 300.0);
        // Screen center (400, 300) should map to grid center (50, 30)
        assert!((gx - 50.0).abs() < 1e-9);
        assert!((gy - 30.0).abs() < 1e-9);

        // Roundtrip
        let (sx, sy) = cam.grid_to_screen(gx, gy);
        assert!((sx - 400.0).abs() < 1e-9);
        assert!((sy - 300.0).abs() < 1e-9);
    }

    #[test]
    fn test_pan_by_pixels() {
        let mut cam = Camera::for_canvas(800, 600);
        cam.center_x = 100.0;
        cam.center_y = 50.0;
        cam.zoom = 2.0;

        cam.pan_by_pixels(20.0, 10.0);

        // Moving +20px right in screen = -10 grid units
        assert!((cam.center_x - 90.0).abs() < 1e-9);
        assert!((cam.center_y - 45.0).abs() < 1e-9);
    }

    #[test]
    fn test_zoom_by() {
        let mut cam = Camera::for_canvas(800, 600);
        cam.zoom = 4.0;

        cam.zoom_by(2.0); // zoom in 2x
        assert!((cam.zoom - 8.0).abs() < 1e-9);

        cam.zoom_by(0.5); // zoom out 2x
        assert!((cam.zoom - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_render_params() {
        let mut cam = Camera::for_canvas(1920, 1080);
        cam.center_x = 500.0;
        cam.center_y = 250.0;
        cam.zoom = 3.0;

        let params = cam.render_params(true);
        assert_eq!(params.canvas_width, 1920);
        assert_eq!(params.canvas_height, 1080);
        assert!(params.show_shadows);
        assert!((params.pixels_per_grid_unit - 3.0).abs() < 1e-9);
    }
}
