//! Platform-agnostic rendering data for BioFabric visualization.
//!
//! This module does **not** draw anything. It computes *what* to draw and
//! packages it into GPU-friendly data structures that any renderer (WebGL2,
//! wgpu, image export) can consume directly.
//!
//! ## Modern approach
//!
//! Instead of the Java BioFabric's dual renderer (standard + bucket), we use
//! a single GPU-instanced rendering pipeline:
//!
//! - Each line (node or link) is one **instance**: 8 floats
//!   `[x0, y0, x1, y1, r, g, b, a]` packed into a flat `Vec<f32>`.
//! - The GPU vertex shader expands each instance into a screen-aligned quad.
//! - **One draw call** renders all nodes; one more renders all links.
//!   Even at 2M edges this runs at 60 fps on commodity GPUs.
//!
//! ## Modules
//!
//! - [`color`] — Color palette generation and assignment
//! - [`viewport`] — Viewport culling and level-of-detail decisions
//! - [`gpu_data`] — GPU instance buffer layout and extraction

pub mod camera;
pub mod color;
pub mod bucket;
pub mod buffer;
pub mod display_options;
pub mod gpu_data;
pub mod pipeline;
pub mod pool;
pub mod raster;
pub mod paths;
pub mod viewport;

pub use camera::Camera;
pub use color::{ColorPalette, FabricColor};
pub use display_options::DisplayOptions;
pub use bucket::{BucketRenderOutput, BucketRenderParams, BucketRenderer, BufBuildDrawer};
pub use buffer::BufferBuilder;
pub use gpu_data::{LineBatch, LineInstance, RectBatch, RectInstance, RenderOutput, TextBatch, TextLabel};
pub use pipeline::RenderPipeline;
pub use pool::ImgAndBufPool;
pub use raster::{PaintCacheSmall, RasterCache};
pub use paths::{BoxPath, GlyphPath, LinePath, TextPath};
pub use viewport::{LodLevel, RenderParams, Viewport};
