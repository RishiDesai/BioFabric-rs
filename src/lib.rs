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
//! - [`model`] - Core data structures: Network, Node, Link
//! - [`io`] - File format parsers (SIF, GW)
//! - [`layout`] - Layout algorithms for node ordering and edge placement
//! - [`analysis`] - Graph analysis algorithms
//!
//! ## Example
//!
//! ```rust,ignore
//! use biofabric::{Network, io::sif};
//! use std::path::Path;
//!
//! // Load a network from SIF file
//! let network = sif::parse_file(Path::new("network.sif"))?;
//!
//! // Apply default layout
//! let layout = network.compute_layout(&Default::default())?;
//!
//! // Access layout information
//! for (node_id, node_layout) in layout.nodes() {
//!     println!("Node {} at row {}", node_id, node_layout.row);
//! }
//! ```
//!
//! ## References
//!
//! - Original Java implementation: <https://github.com/wjrl/BioFabric>
//! - BioFabric paper: Longabaugh WJR. Combing the hairball with BioFabric: a new
//!   approach for visualization of large networks. BMC Bioinformatics. 2012.

pub mod analysis;
pub mod io;
pub mod layout;
pub mod model;

// Re-export commonly used types at crate root
pub use model::{Link, Network, Node, NodeId};
