//! Layout algorithm traits.
//!
//! These traits define the interface for layout algorithms. Custom layouts
//! can be implemented by implementing these traits.

use super::result::NetworkLayout;
use crate::model::{Network, NodeId};
use thiserror::Error;

/// Errors that can occur during layout computation.
#[derive(Error, Debug)]
pub enum LayoutError {
    /// The network doesn't meet the criteria for this layout.
    #[error("Layout criteria not met: {0}")]
    CriteriaNotMet(String),

    /// The layout computation was cancelled.
    #[error("Layout cancelled")]
    Cancelled,

    /// An internal error occurred.
    #[error("Layout error: {0}")]
    Internal(String),
}

/// Result type for layout operations.
pub type LayoutResult<T> = Result<T, LayoutError>;

/// Parameters that can be passed to layout algorithms.
#[derive(Debug, Clone, Default)]
pub struct LayoutParams {
    /// Optional starting node for BFS-based layouts.
    pub start_node: Option<NodeId>,

    /// Whether to include shadow links in the layout.
    pub include_shadows: bool,
}

/// Trait for node layout algorithms.
///
/// A node layout assigns each node to a row (y-coordinate).
///
/// ## Implementation Notes
///
/// Implementors should:
/// - Return nodes in order (index = row number)
/// - Include all nodes from the network
/// - Handle disconnected components
///
/// ## Example Implementation
///
/// ```rust,ignore
/// struct AlphabeticalLayout;
///
/// impl NodeLayout for AlphabeticalLayout {
///     fn layout_nodes(&self, network: &Network, _params: &LayoutParams)
///         -> LayoutResult<Vec<NodeId>>
///     {
///         let mut nodes: Vec<_> = network.node_ids().cloned().collect();
///         nodes.sort();
///         Ok(nodes)
///     }
/// }
/// ```
pub trait NodeLayout {
    /// Compute the node ordering.
    ///
    /// Returns a vector of node IDs where the index is the row number.
    fn layout_nodes(
        &self,
        network: &Network,
        params: &LayoutParams,
    ) -> LayoutResult<Vec<NodeId>>;

    /// Check if this layout can handle the given network.
    ///
    /// Some layouts have specific requirements (e.g., connected graph,
    /// no self-loops). This method allows checking before attempting layout.
    fn criteria_met(&self, _network: &Network) -> LayoutResult<()> {
        Ok(())
    }

    /// Human-readable name for this layout.
    fn name(&self) -> &'static str;
}

/// Trait for edge layout algorithms.
///
/// An edge layout assigns each edge to a column (x-coordinate) and
/// computes the horizontal span of each node.
///
/// ## Implementation Notes
///
/// Implementors should:
/// - Assign each link to a unique column (unless shadows overlap)
/// - Compute min/max columns for each node based on incident edges
/// - Handle the node ordering from a prior NodeLayout
pub trait EdgeLayout {
    /// Compute the full network layout given a node ordering.
    ///
    /// # Arguments
    /// * `network` - The network to layout
    /// * `node_order` - Node IDs in row order (from NodeLayout)
    /// * `params` - Layout parameters
    ///
    /// # Returns
    /// Complete layout with node positions and link columns.
    fn layout_edges(
        &self,
        network: &Network,
        node_order: &[NodeId],
        params: &LayoutParams,
    ) -> LayoutResult<NetworkLayout>;

    /// Human-readable name for this layout.
    fn name(&self) -> &'static str;
}
