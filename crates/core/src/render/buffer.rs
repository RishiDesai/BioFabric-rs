//! Buffer builder stubs.
//!
//! Parity with Java `BufferBuilder`.

/// Image buffer builder (stub).
#[derive(Debug, Clone)]
pub struct BufferBuilder {
    pub width_px: u32,
    pub height_px: u32,
}

impl BufferBuilder {
    pub fn new(width_px: u32, height_px: u32) -> Self {
        Self { width_px, height_px }
    }

    pub fn clear(&mut self) {
        // TODO: Clear backing buffer.
        todo!("Implement buffer clear")
    }
}
