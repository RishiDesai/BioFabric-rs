//! Control-top layout algorithm.
//!
//! Places designated "control" nodes at the top of the layout, followed by
//! their targets ordered using various strategies. Useful for regulatory
//! networks where certain nodes (transcription factors, kinases, etc.) are
//! upstream controllers.
//!
//! ## Control Ordering Modes
//!
//! - **PartialOrder** - Topological partial order of controllers
//! - **IntraDegree** - Order by intra-control-set connectivity
//! - **MedianTargetDegree** - Order by median degree of targets
//! - **DegreeOnly** - Simple degree ordering
//! - **FixedList** - User-provided fixed ordering
//!
//! ## Target Ordering Modes
//!
//! - **GrayCode** - Gray code ordering for minimal visual disruption
//! - **DegreeOdometer** - Odometer-style ordering by degree
//! - **TargetDegree** - Order targets by their degree
//! - **BreadthOrder** - Breadth-first order from controllers
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.layouts.ControlTopLayout`

use super::traits::{LayoutError, LayoutParams, LayoutResult, NodeLayout};
use crate::model::{Network, NodeId};
use crate::worker::ProgressMonitor;

/// How to order control nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlOrder {
    /// Topological partial order.
    #[default]
    PartialOrder,
    /// By intra-control-set connectivity.
    IntraDegree,
    /// By median degree of their targets.
    MedianTargetDegree,
    /// Simple degree ordering.
    DegreeOnly,
    /// User-provided fixed list.
    FixedList,
}

/// How to order target nodes (non-control nodes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetOrder {
    /// Gray code ordering.
    #[default]
    GrayCode,
    /// Odometer-style degree ordering.
    DegreeOdometer,
    /// By target degree.
    TargetDegree,
    /// Breadth-first from controllers.
    BreadthOrder,
}

/// Control-top layout configuration.
#[derive(Debug, Clone, Default)]
pub struct ControlTopLayoutParams {
    /// How to order the control nodes.
    pub control_order: ControlOrder,
    /// How to order target nodes.
    pub target_order: TargetOrder,
    /// Explicit list of control node IDs (if using FixedList mode,
    /// this is the order; otherwise, it identifies which nodes are controllers).
    pub control_nodes: Vec<NodeId>,
}

/// Control-top layout.
///
/// Places control nodes at the top, targets below, using the specified
/// ordering strategies.
#[derive(Debug, Clone, Default)]
pub struct ControlTopLayout {
    /// Layout configuration.
    pub config: ControlTopLayoutParams,
}

impl ControlTopLayout {
    /// Create a new control-top layout.
    pub fn new(control_nodes: Vec<NodeId>) -> Self {
        Self {
            config: ControlTopLayoutParams {
                control_nodes,
                ..Default::default()
            },
        }
    }

    /// Set the control ordering mode.
    pub fn with_control_order(mut self, order: ControlOrder) -> Self {
        self.config.control_order = order;
        self
    }

    /// Set the target ordering mode.
    pub fn with_target_order(mut self, order: TargetOrder) -> Self {
        self.config.target_order = order;
        self
    }
}

impl NodeLayout for ControlTopLayout {
    fn layout_nodes(
        &self,
        _network: &Network,
        _params: &LayoutParams,
        _monitor: &dyn ProgressMonitor,
    ) -> LayoutResult<Vec<NodeId>> {
        // TODO: Implement control-top layout
        //
        // 1. Separate nodes into control set and target set
        // 2. Order control nodes using self.config.control_order
        // 3. Order target nodes using self.config.target_order
        // 4. Concatenate: controls first, then targets
        //
        // See ControlTopLayout.java: doNodeLayout()
        //
        todo!("Implement control-top layout - see ControlTopLayout.java")
    }

    fn criteria_met(&self, network: &Network) -> LayoutResult<()> {
        if !network.metadata.is_directed {
            return Err(LayoutError::CriteriaNotMet(
                "ControlTopLayout requires a fully directed network. \
                 All links must have directed == Some(true)."
                    .into(),
            ));
        }
        if self.config.control_nodes.is_empty() {
            return Err(LayoutError::CriteriaNotMet(
                "ControlTopLayout requires at least one control node to be specified."
                    .into(),
            ));
        }
        // Verify all specified control nodes exist in the network
        for node_id in &self.config.control_nodes {
            if !network.contains_node(node_id) {
                return Err(LayoutError::CriteriaNotMet(format!(
                    "Control node '{}' not found in network",
                    node_id
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Control Top"
    }
}
