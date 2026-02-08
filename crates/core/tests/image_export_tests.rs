// Image export tests for biofabric-rs.
//
// These tests validate that the image exporter produces files with the
// correct dimensions (width × height) in each supported format. They do
// NOT compare pixel content because the Rust CPU rasterizer intentionally
// produces different output from Java's AWT-based renderer.
//
// == Prerequisites ==
//
//   The `png_export` feature must be enabled (pulls in the `image` crate).
//   This test file is configured with `required-features = ["png_export"]`
//   in Cargo.toml, so `cargo test` skips it unless the feature is active.
//
// == Running ==
//
//   cargo test --test image_export_tests --features png_export
//   cargo test --test image_export_tests --features png_export -- --nocapture

use biofabric_core::export::image::{ExportOptions, ImageExporter, ImageFormat, ImageOutput};
use biofabric_core::render::gpu_data::{LineBatch, RectBatch, RenderOutput, TextBatch};
use biofabric_core::worker::NoopMonitor;

/// Create an empty `RenderOutput` (no geometry).
///
/// This is sufficient for dimension-validation tests — the exporter
/// should produce a correctly-sized blank image regardless of content.
fn empty_render_output() -> RenderOutput {
    RenderOutput {
        node_annotations: RectBatch::new(),
        link_annotations: RectBatch::new(),
        links: LineBatch::with_capacity(0),
        nodes: LineBatch::with_capacity(0),
        labels: TextBatch::new(),
    }
}

/// Decode an image from bytes and return its (width, height).
fn decode_dimensions(bytes: &[u8], _format: ImageFormat) -> (u32, u32) {
    use std::io::Cursor;

    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .expect("Failed to guess image format");
    let img = reader.decode().expect("Failed to decode image");
    (img.width(), img.height())
}

/// Helper: export with the given format and dimensions, then verify.
fn assert_export_dimensions(format: ImageFormat, width: u32, height: u32) {
    let render = empty_render_output();
    let monitor = NoopMonitor;

    let options = ExportOptions {
        format,
        width_px: width,
        height_px: height,
        ..Default::default()
    };

    let output: ImageOutput = ImageExporter::export(&render, &options, &monitor)
        .expect("Image export should succeed");

    // Verify the output metadata
    assert_eq!(output.format, format);
    assert_eq!(output.width_px, width, "output metadata width mismatch");
    assert_eq!(output.height_px, height, "output metadata height mismatch");

    // Verify the encoded image can be decoded to the right dimensions
    assert!(!output.bytes.is_empty(), "exported image should not be empty");
    let (decoded_w, decoded_h) = decode_dimensions(&output.bytes, format);
    assert_eq!(decoded_w, width, "decoded image width mismatch");
    assert_eq!(decoded_h, height, "decoded image height mismatch");
}

// ---------------------------------------------------------------------------
// PNG tests
// ---------------------------------------------------------------------------

#[test]
fn image_export_png_640x480() {
    assert_export_dimensions(ImageFormat::Png, 640, 480);
}

#[test]
fn image_export_png_1920x1080() {
    assert_export_dimensions(ImageFormat::Png, 1920, 1080);
}

#[test]
fn image_export_png_small_100x50() {
    assert_export_dimensions(ImageFormat::Png, 100, 50);
}

// ---------------------------------------------------------------------------
// JPEG tests
// ---------------------------------------------------------------------------

#[test]
fn image_export_jpeg_800x600() {
    assert_export_dimensions(ImageFormat::Jpeg, 800, 600);
}

#[test]
fn image_export_jpeg_1024x768() {
    assert_export_dimensions(ImageFormat::Jpeg, 1024, 768);
}

// ---------------------------------------------------------------------------
// TIFF tests
// ---------------------------------------------------------------------------

#[test]
fn image_export_tiff_640x480() {
    assert_export_dimensions(ImageFormat::Tiff, 640, 480);
}

#[test]
fn image_export_tiff_320x240() {
    assert_export_dimensions(ImageFormat::Tiff, 320, 240);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn image_export_png_1x1() {
    assert_export_dimensions(ImageFormat::Png, 1, 1);
}

#[test]
fn image_export_custom_background_color() {
    let render = empty_render_output();
    let monitor = NoopMonitor;

    let options = ExportOptions {
        format: ImageFormat::Png,
        width_px: 64,
        height_px: 64,
        background_color: "#FF0000".to_string(),
        ..Default::default()
    };

    let output = ImageExporter::export(&render, &options, &monitor)
        .expect("Export with custom background should succeed");
    assert_eq!(output.width_px, 64);
    assert_eq!(output.height_px, 64);
    assert!(!output.bytes.is_empty());
}
