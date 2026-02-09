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
        render: &RenderOutput,
        options: &ExportOptions,
        _monitor: &dyn ProgressMonitor,
    ) -> Result<ImageOutput, String> {
        #[cfg(feature = "png_export")]
        {
            use image::{DynamicImage, RgbaImage, Rgba};
            use std::io::Cursor;
            use crate::render::gpu_data::{FLOATS_PER_INSTANCE, FLOATS_PER_RECT};

            let w = options.width_px;
            let h = options.height_px;
            let bg = parse_hex_color(&options.background_color);
            let mut img = RgbaImage::from_pixel(w, h, bg);

            // Determine the grid → pixel transform.
            // We need to know the total grid extent from the render data.
            // Scan all instances to find the bounding box.
            let (grid_w, grid_h) = compute_grid_extent(render);
            if grid_w <= 0.0 || grid_h <= 0.0 {
                // Nothing to render — return background-only image
                return encode_image(&img, options);
            }

            // Add a small margin (2% each side)
            let margin_frac = 0.02;
            let view_w = grid_w * (1.0 + 2.0 * margin_frac);
            let view_h = grid_h * (1.0 + 2.0 * margin_frac);
            let offset_x = -grid_w * margin_frac;
            let offset_y = -grid_h * margin_frac;

            // Pixels per grid unit (uniform scaling to fit)
            let scale_x = w as f64 / view_w;
            let scale_y = h as f64 / view_h;
            let scale = scale_x.min(scale_y);

            // Center the layout in the image
            let total_scaled_w = grid_w * scale;
            let total_scaled_h = grid_h * scale;
            let pad_x = (w as f64 - total_scaled_w) / 2.0 - offset_x * scale;
            let pad_y = (h as f64 - total_scaled_h) / 2.0 - offset_y * scale;

            let to_px_x = |grid_x: f64| -> f64 { grid_x * scale + pad_x };
            let to_px_y = |grid_y: f64| -> f64 { grid_y * scale + pad_y };

            // ---- Rasterize annotation rectangles ----
            rasterize_rects(&mut img, &render.node_annotations, w, h, &to_px_x, &to_px_y, scale);
            rasterize_rects(&mut img, &render.link_annotations, w, h, &to_px_x, &to_px_y, scale);

            // ---- Rasterize link lines (vertical) ----
            let line_w = (scale * options.line_width_scale as f64).max(1.0);
            rasterize_lines(&mut img, &render.links, w, h, &to_px_x, &to_px_y, line_w, false);

            // ---- Rasterize node lines (horizontal, on top) ----
            let node_w = (scale * options.line_width_scale as f64 * 2.0).max(1.0);
            rasterize_lines(&mut img, &render.nodes, w, h, &to_px_x, &to_px_y, node_w, true);

            encode_image(&img, options)
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

// ---------------------------------------------------------------------------
// Rasterization helpers
// ---------------------------------------------------------------------------

/// Encode an RgbaImage to the requested format and wrap in ImageOutput.
#[cfg(feature = "png_export")]
fn encode_image(
    img: &image::RgbaImage,
    options: &ExportOptions,
) -> Result<ImageOutput, String> {
    use image::DynamicImage;
    use std::io::Cursor;

    let img_format = match options.format {
        ImageFormat::Png => image::ImageFormat::Png,
        ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        ImageFormat::Tiff => image::ImageFormat::Tiff,
    };

    let dynamic_img = DynamicImage::ImageRgba8(img.clone());
    let writable: DynamicImage = if options.format == ImageFormat::Jpeg {
        DynamicImage::ImageRgb8(dynamic_img.to_rgb8())
    } else {
        dynamic_img
    };

    let mut bytes = Vec::new();
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

/// Compute the grid-space bounding box from all instances in a RenderOutput.
#[cfg(feature = "png_export")]
fn compute_grid_extent(render: &RenderOutput) -> (f64, f64) {
    use crate::render::gpu_data::{FLOATS_PER_INSTANCE, FLOATS_PER_RECT};

    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;

    // From node lines (horizontal: x0,y0 → x1,y1)
    for chunk in render.nodes.data.chunks_exact(FLOATS_PER_INSTANCE) {
        let x1 = chunk[2] as f64;
        let y0 = chunk[1] as f64;
        if x1 > max_x { max_x = x1; }
        if y0 > max_y { max_y = y0; }
    }

    // From link lines (vertical: x0,y0 → x0,y1)
    for chunk in render.links.data.chunks_exact(FLOATS_PER_INSTANCE) {
        let x0 = chunk[0] as f64;
        let y1 = chunk[3] as f64;
        if x0 > max_x { max_x = x0; }
        if y1 > max_y { max_y = y1; }
    }

    // From annotation rects
    for batch in [&render.node_annotations, &render.link_annotations] {
        for chunk in batch.data.chunks_exact(FLOATS_PER_RECT) {
            let right = chunk[0] as f64 + chunk[2] as f64;
            let bottom = chunk[1] as f64 + chunk[3] as f64;
            if right > max_x { max_x = right; }
            if bottom > max_y { max_y = bottom; }
        }
    }

    // Add 1 to convert from 0-indexed max to extent
    (max_x + 1.0, max_y + 1.0)
}

/// Rasterize annotation rectangles onto an image.
#[cfg(feature = "png_export")]
fn rasterize_rects(
    img: &mut image::RgbaImage,
    batch: &crate::render::gpu_data::RectBatch,
    w: u32,
    h: u32,
    to_px_x: &dyn Fn(f64) -> f64,
    to_px_y: &dyn Fn(f64) -> f64,
    scale: f64,
) {
    use crate::render::gpu_data::FLOATS_PER_RECT;

    for chunk in batch.data.chunks_exact(FLOATS_PER_RECT) {
        let gx = chunk[0] as f64;
        let gy = chunk[1] as f64;
        let gw = chunk[2] as f64;
        let gh = chunk[3] as f64;
        let r = (chunk[4] * 255.0) as u8;
        let g = (chunk[5] * 255.0) as u8;
        let b = (chunk[6] * 255.0) as u8;
        let a = (chunk[7] * 255.0) as u8;

        let px0 = to_px_x(gx).max(0.0) as u32;
        let py0 = to_px_y(gy).max(0.0) as u32;
        let px1 = (to_px_x(gx + gw)).min(w as f64) as u32;
        let py1 = (to_px_y(gy + gh)).min(h as f64) as u32;

        for py in py0..py1.min(h) {
            for px in px0..px1.min(w) {
                alpha_blend(img, px, py, r, g, b, a);
            }
        }
    }
}

/// Rasterize line instances onto an image.
///
/// `is_horizontal`: true for node lines (horizontal), false for link lines (vertical).
#[cfg(feature = "png_export")]
fn rasterize_lines(
    img: &mut image::RgbaImage,
    batch: &crate::render::gpu_data::LineBatch,
    w: u32,
    h: u32,
    to_px_x: &dyn Fn(f64) -> f64,
    to_px_y: &dyn Fn(f64) -> f64,
    line_width: f64,
    is_horizontal: bool,
) {
    use crate::render::gpu_data::FLOATS_PER_INSTANCE;

    let half_w = (line_width / 2.0).max(0.5);

    for chunk in batch.data.chunks_exact(FLOATS_PER_INSTANCE) {
        let gx0 = chunk[0] as f64;
        let gy0 = chunk[1] as f64;
        let gx1 = chunk[2] as f64;
        let gy1 = chunk[3] as f64;
        let r = (chunk[4] * 255.0) as u8;
        let g = (chunk[5] * 255.0) as u8;
        let b = (chunk[6] * 255.0) as u8;
        let a = (chunk[7] * 255.0) as u8;

        if is_horizontal {
            // Horizontal line: y0 == y1, x0 → x1
            let py_center = to_px_y(gy0);
            let px0 = to_px_x(gx0).max(0.0) as u32;
            let px1 = (to_px_x(gx1) + 1.0).min(w as f64) as u32;
            let py_start = (py_center - half_w).max(0.0) as u32;
            let py_end = ((py_center + half_w).ceil() as u32).min(h);

            for py in py_start..py_end {
                for px in px0..px1.min(w) {
                    blend_opaque(img, px, py, r, g, b);
                }
            }
        } else {
            // Vertical line: x0 == x1, y0 → y1
            let px_center = to_px_x(gx0);
            let py0 = to_px_y(gy0).max(0.0) as u32;
            let py1 = (to_px_y(gy1) + 1.0).min(h as f64) as u32;
            let px_start = (px_center - half_w).max(0.0) as u32;
            let px_end = ((px_center + half_w).ceil() as u32).min(w);

            for py in py0..py1.min(h) {
                for px in px_start..px_end {
                    blend_opaque(img, px, py, r, g, b);
                }
            }
        }
    }
}

/// Alpha-blend a single pixel.
#[cfg(feature = "png_export")]
#[inline]
fn alpha_blend(img: &mut image::RgbaImage, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    let pixel = img.get_pixel_mut(x, y);
    let alpha = a as f32 / 255.0;
    let inv = 1.0 - alpha;
    pixel[0] = (r as f32 * alpha + pixel[0] as f32 * inv) as u8;
    pixel[1] = (g as f32 * alpha + pixel[1] as f32 * inv) as u8;
    pixel[2] = (b as f32 * alpha + pixel[2] as f32 * inv) as u8;
    // Keep destination alpha at 255 (opaque)
}

/// Write an opaque pixel (fast path for lines with a=255).
#[cfg(feature = "png_export")]
#[inline]
fn blend_opaque(img: &mut image::RgbaImage, x: u32, y: u32, r: u8, g: u8, b: u8) {
    img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
}
