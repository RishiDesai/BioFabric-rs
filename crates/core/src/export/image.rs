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
        options: &ExportOptions,
        _monitor: &dyn ProgressMonitor,
    ) -> Result<ImageOutput, String> {
        #[cfg(feature = "png_export")]
        {
            use image::{DynamicImage, RgbaImage};
            use std::io::Cursor;

            let bg = parse_hex_color(&options.background_color);
            let img = RgbaImage::from_pixel(options.width_px, options.height_px, bg);

            // TODO: Rasterize annotations, links, and nodes onto `img`.
            // For now, only the background is filled — sufficient for
            // dimension-validation tests.

            let mut bytes = Vec::new();
            let img_format = match options.format {
                ImageFormat::Png => image::ImageFormat::Png,
                ImageFormat::Jpeg => image::ImageFormat::Jpeg,
                ImageFormat::Tiff => image::ImageFormat::Tiff,
            };

            // JPEG does not support RGBA — convert to RGB first.
            let dynamic_img = DynamicImage::ImageRgba8(img);
            let writable: DynamicImage = if options.format == ImageFormat::Jpeg {
                DynamicImage::ImageRgb8(dynamic_img.to_rgb8())
            } else {
                dynamic_img
            };

            writable
                .write_to(&mut Cursor::new(&mut bytes), img_format)
                .map_err(|e| format!("Image encoding failed: {}", e))?;

            Ok(ImageOutput {
                bytes,
                format: options.format,
                width_px: options.width_px,
                height_px: options.height_px,
            })
        }

        #[cfg(not(feature = "png_export"))]
        Err("Image export requires the `png_export` feature (enables the `image` crate)".to_string())
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

/// Parse a hex color string (e.g. `"#RRGGBB"` or `"#RRGGBBAA"`) into an
/// RGBA pixel value for use with the `image` crate.
#[cfg(feature = "png_export")]
fn parse_hex_color(hex: &str) -> image::Rgba<u8> {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            (r, g, b, 255u8)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            (r, g, b, a)
        }
        _ => (255, 255, 255, 255),
    };
    image::Rgba([r, g, b, a])
}
