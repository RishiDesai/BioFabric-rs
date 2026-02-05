//! Graph analysis algorithms.
//!
//! This module provides algorithms for analyzing network structure:
//!
//! - [`graph`] - Basic graph algorithms (BFS, DFS, connected components)
//! - Cycle detection (TODO)
//! - Centrality measures (TODO)
//!
//! These algorithms are used by layout algorithms and can also be used
//! directly for network analysis.

pub mod graph;

pub use graph::{bfs, connected_components, dfs};
