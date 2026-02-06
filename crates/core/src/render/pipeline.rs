//! Render extraction pipeline glue.
//!
//! This is a thin adapter around `RenderOutput::extract` so callers can
//! assemble the required inputs (layout, viewport params, palette) in a
//! single place. The actual extraction logic remains unimplemented.

use super::color::ColorPalette;
use super::gpu_data::RenderOutput;
use super::viewport::RenderParams;
use crate::layout::result::NetworkLayout;

/// Simple render pipeline wrapper (stub).
#[derive(Debug, Clone)]
pub struct RenderPipeline {
    pub layout: NetworkLayout,
    pub params: RenderParams,
    pub palette: ColorPalette,
}

impl RenderPipeline {
    /// Create a new render pipeline.
    pub fn new(layout: NetworkLayout, params: RenderParams, palette: ColorPalette) -> Self {
        Self {
            layout,
            params,
            palette,
        }
    }

    /// Extract GPU-ready render output.
    pub fn extract(&self) -> RenderOutput {
        RenderOutput::extract(&self.layout, &self.params, &self.palette)
    }
}
