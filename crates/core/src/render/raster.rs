//! Raster cache stubs.
//!
//! Parity with Java `RasterCache` and `PaintCacheSmall`.

/// Cached raster for repeated drawing (placeholder).
#[derive(Debug, Default)]
pub struct RasterCache;

impl RasterCache {
    pub fn new() -> Self {
        Self
    }

    pub fn clear(&mut self) {
        // TODO: Clear cached rasters.
        todo!("Implement raster cache clear")
    }
}

/// Small paint cache placeholder.
#[derive(Debug, Default)]
pub struct PaintCacheSmall;

impl PaintCacheSmall {
    pub fn new() -> Self {
        Self
    }
}
