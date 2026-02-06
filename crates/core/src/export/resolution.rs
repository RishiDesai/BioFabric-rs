//! Resolution settings for image export.
//!
//! Mirrors Java `ImageExporter.ResolutionSettings`.

/// Resolution and scaling configuration.
#[derive(Debug, Clone)]
pub struct ResolutionSettings {
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u32,
    pub scale: f32,
}

impl Default for ResolutionSettings {
    fn default() -> Self {
        Self {
            width_px: 1920,
            height_px: 1080,
            dpi: 300,
            scale: 1.0,
        }
    }
}
