//! Layout algorithms for BioFabric visualization.
//!
//! BioFabric layout consists of two phases:
//!
//! 1. **Node Layout** - Assigns each node to a row (y-coordinate)
//! 2. **Edge Layout** - Assigns each edge to a column (x-coordinate)
//!
//! The layout determines the final visualization. Different layout algorithms
//! produce different orderings that can reveal different network structures.
//!
//! ## Available Layouts
//!
//! - [`DefaultNodeLayout`] - Orders nodes by BFS traversal from highest-degree node
//! - [`DefaultEdgeLayout`] - Places edges to minimize node horizontal span
//! - [`NodeSimilarityLayout`] - Groups similar nodes together
//!
//! ## Layout Result
//!
//! The [`NetworkLayout`] struct contains the computed layout:
//!
//! ```rust,ignore
//! let layout = network.compute_layout(&Default::default())?;
//!
//! for (node_id, info) in layout.nodes() {
//!     println!("Node {} at row {}, spans columns {}-{}",
//!         node_id, info.row, info.min_col, info.max_col);
//! }
//!
//! for (idx, info) in layout.links().enumerate() {
//!     println!("Link {} at column {}", idx, info.column);
//! }
//! ```

pub mod default;
pub mod result;
pub mod similarity;
pub mod traits;

pub use default::{DefaultEdgeLayout, DefaultNodeLayout};
pub use result::{LinkLayout, NetworkLayout, NodeLayout as NodeLayoutInfo};
pub use similarity::NodeSimilarityLayout;
pub use traits::{EdgeLayout, NodeLayout};
