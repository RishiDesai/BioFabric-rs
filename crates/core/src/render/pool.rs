//! Image and buffer pooling stubs.
//!
//! Parity with Java `ImgAndBufPool`.

/// Image/buffer pool placeholder.
#[derive(Debug, Default)]
pub struct ImgAndBufPool;

impl ImgAndBufPool {
    pub fn new() -> Self {
        Self
    }

    pub fn allocate(&self, _width_px: u32, _height_px: u32) {
        // TODO: Allocate a pooled image buffer.
        todo!("Implement image/buffer pool allocation")
    }
}
