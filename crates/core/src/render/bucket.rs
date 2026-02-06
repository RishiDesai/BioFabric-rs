//! Bucket renderer stubs for very large networks.
//!
//! Parity with Java `BucketRenderer` and `BufBuildDrawer`.

use crate::render::RenderOutput;
use crate::worker::ProgressMonitor;

/// Parameters for bucket rendering.
#[derive(Debug, Clone)]
pub struct BucketRenderParams {
    pub width_px: u32,
    pub height_px: u32,
    pub links_per_pixel_threshold: usize,
}

impl Default for BucketRenderParams {
    fn default() -> Self {
        Self {
            width_px: 1920,
            height_px: 1080,
            links_per_pixel_threshold: 100,
        }
    }
}

/// Output of a bucket render pass (placeholder).
#[derive(Debug, Clone)]
pub struct BucketRenderOutput {
    pub pixels: Vec<u8>,
    pub width_px: u32,
    pub height_px: u32,
}

/// Trait for bucket drawing backends (stub).
pub trait BufBuildDrawer {
    fn begin(&mut self, width_px: u32, height_px: u32);
    fn draw_pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]);
    fn finish(&mut self) -> BucketRenderOutput;
}

/// Bucket renderer (stub).
pub struct BucketRenderer;

impl BucketRenderer {
    /// Render using a bucket strategy.
    pub fn render(
        _render: &RenderOutput,
        _params: &BucketRenderParams,
        _drawer: &mut dyn BufBuildDrawer,
        _monitor: &dyn ProgressMonitor,
    ) -> Result<BucketRenderOutput, String> {
        // TODO: Implement bucket rendering for dense networks.
        todo!("Implement bucket renderer")
    }
}
