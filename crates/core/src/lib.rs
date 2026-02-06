// Stubs use todo!() — suppress noise until implementation begins.
#![allow(unused_variables, unused_imports)]

//! # BioFabric
//!
//! A Rust implementation of BioFabric, a network visualization tool that represents
//! nodes as horizontal lines and edges as vertical line segments.
//!
//! ## Overview
//!
//! BioFabric uses a unique visualization approach:
//! - Each **node** is represented as a horizontal line at a specific row
//! - Each **edge** is represented as a vertical line segment at a specific column
//! - A node's horizontal extent spans from its leftmost to rightmost incident edge
//!
//! This approach allows visualization of very large networks (100K+ nodes) while
//! preserving the ability to see individual edges.
//!
//! ## Modules
//!
//! - [`model`] - Core data structures: Network, Node, Link, Annotation
//! - [`io`] - File format parsers (SIF, GW, alignment, JSON, XML)
//! - [`layout`] - Layout algorithms for node ordering and edge placement
//! - [`alignment`] - Network alignment: merging, scoring, and cycle detection
//! - [`analysis`] - Graph analysis algorithms (BFS, DFS, components, cycles)
//! - [`render`] - Platform-agnostic rendering data (colors, buckets, tiles)
//! - [`util`] - Shared utilities (spatial indexing, data helpers)
//!
//! ## References
//!
//! - Original Java implementation: <https://github.com/wjrl/BioFabric>
//! - BioFabric paper: Longabaugh WJR. Combing the hairball with BioFabric: a new
//!   approach for visualization of large networks. BMC Bioinformatics. 2012.
//! - VISNAB paper: Desai, R.M., Longabaugh, W.J.R., Hayes, W.B. (2021).
//!   BioFabric Visualization of Network Alignments. Springer, Cham.

pub mod alignment;
pub mod analysis;
pub mod error;
pub mod export;
pub mod io;
pub mod layout;
pub mod model;
pub mod render;
pub mod util;
pub mod worker;

// Re-export commonly used types at crate root
pub use error::{BioFabricError, Result};
pub use io::session::Session;
pub use model::{Annotation, AnnotationSet, Link, Network, NetworkMetadata, Node, NodeComparison, NodeId, SelectionState};
pub use worker::{CancelledError, LoopReporter, NoopMonitor, ProgressMonitor};
