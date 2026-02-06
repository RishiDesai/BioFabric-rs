//! Image export for BioFabric visualizations.
//!
//! Rasterizes a [`RenderOutput`] to a pixel buffer and encodes it as
//! PNG, JPEG, or TIFF. This is the CPU rendering path used by the CLI
//! `render` command. For interactive rendering, use the GPU path
//! (WebGL2 / wgpu) instead.
//!
//! ## Algorithm
//!
//! 1. Create a pixel buffer at the requested resolution.
//! 2. Fill with the background color.
//! 3. Rasterize annotation rectangles (semi-transparent).
//! 4. Rasterize link lines (vertical, 1px or antialiased).
//! 5. Rasterize node lines (horizontal, 2px or antialiased).
//! 6. Encode the pixel buffer to the requested format.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.cmd.CommandSet` (export action)
//! - Java: `BioFabricPanel.exportImage()` via `BufferedImage`

use crate::render::gpu_data::RenderOutput;
use crate::worker::ProgressMonitor;

/// Output image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
}

/// High-level export intent (affects default sizing presets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportProfile {
    /// Screen-oriented export (default).
    Screen,
    /// Publication-oriented export (high DPI).
    Publication,
}

/// Options for exporting an image.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ImageFormat,
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    /// Optional preset that can override width/height/dpi defaults.
    pub profile: ExportProfile,
    /// Background color as RGBA hex string (e.g. `"#FFFFFF"`).
    pub background_color: String,
    /// Line width multiplier (1.0 = default).
    pub line_width_scale: f32,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            width_px: 1920,
            height_px: 1080,
            dpi: 300,
            profile: ExportProfile::Screen,
            background_color: "#FFFFFF".to_string(),
            line_width_scale: 1.0,
        }
    }
}

/// Export result.
#[derive(Debug, Clone)]
pub struct ImageOutput {
    /// Raw encoded image bytes (PNG, JPEG, or TIFF).
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
    pub width_px: u32,
    pub height_px: u32,
}

/// Image exporter — CPU rasterizer for CLI usage.
///
/// ## Feature gate
///
/// Actual PNG/JPEG encoding requires the `png_export` feature (which
/// pulls in the `image` crate). Without it, `export()` returns an error.
pub struct ImageExporter;

impl ImageExporter {
    /// Export a rendered output to an image buffer.
    ///
    /// ## Algorithm
    ///
    /// 1. Create an `image::RgbaImage` at the requested resolution
    /// 2. Fill with the background color
    /// 3. For each `RectInstance` in `render.node_annotations` and
    ///    `render.link_annotations`:
    ///    - Map grid coordinates to pixel coordinates
    ///    - Alpha-blend the rectangle color onto the pixel buffer
    /// 4. For each `LineInstance` in `render.links`:
    ///    - Map grid coordinates to pixel coordinates
    ///    - Draw a vertical line (Bresenham or subpixel)
    /// 5. For each `LineInstance` in `render.nodes`:
    ///    - Map grid coordinates to pixel coordinates
    ///    - Draw a horizontal line
    /// 6. Encode to PNG/JPEG/TIFF
    ///
    /// ## References
    ///
    /// - Java: `ImageExporter` does this via Java2D `Graphics2D`
    pub fn export(
        _render: &RenderOutput,
        _options: &ExportOptions,
        _monitor: &dyn ProgressMonitor,
    ) -> Result<ImageOutput, String> {
        // TODO: Implement CPU rasterization.
        //
        // When `png_export` feature is enabled, use the `image` crate:
        //
        // #[cfg(feature = "png_export")]
        // {
        //     use image::{RgbaImage, Rgba, imageops};
        //     let mut img = RgbaImage::from_pixel(
        //         options.width_px, options.height_px,
        //         parse_hex_color(&options.background_color),
        //     );
        //
        //     // Compute grid-to-pixel transform
        //     let scale_x = options.width_px as f64 / layout_width;
        //     let scale_y = options.height_px as f64 / layout_height;
        //
        //     // Rasterize annotations (semi-transparent rectangles)
        //     // Rasterize links (vertical lines)
        //     // Rasterize nodes (horizontal lines)
        //
        //     let mut bytes = Vec::new();
        //     img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        //        .map_err(|e| e.to_string())?;
        //
        //     Ok(ImageOutput { bytes, format: ImageFormat::Png, width_px, height_px })
        // }
        //
        // #[cfg(not(feature = "png_export"))]
        // Err("PNG export requires the `png_export` feature".to_string())
        //
        todo!("Implement CPU image export (requires `png_export` feature)")
    }

    /// Export a rendered output directly to a file path.
    ///
    /// Convenience wrapper that calls `export()` and writes the bytes.
    pub fn export_to_file(
        render: &RenderOutput,
        options: &ExportOptions,
        path: &std::path::Path,
        monitor: &dyn ProgressMonitor,
    ) -> Result<(), String> {
        let output = Self::export(render, options, monitor)?;
        std::fs::write(path, &output.bytes).map_err(|e| e.to_string())
    }
}
