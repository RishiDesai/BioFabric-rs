//! World bank (hub-spoke) layout algorithm.
//!
//! Designed for networks with a hub-and-spoke pattern, where many "satellite"
//! nodes connect to exactly one "hub" node. Groups satellite nodes around
//! their respective hubs, ordering hubs by degree.
//!
//! ## Algorithm
//!
//! 1. Identify hub nodes (nodes with degree > threshold)
//! 2. Identify satellite nodes (degree == 1, connected to a hub)
//! 3. Order hubs by degree (descending)
//! 4. For each hub, place its satellites after it
//! 5. Remaining non-hub, non-satellite nodes are placed using default BFS
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.layouts.WorldBankLayout`

use super::traits::{LayoutParams, LayoutResult, NodeLayout};
use crate::model::{Network, NodeId};
use crate::worker::ProgressMonitor;

/// World bank (hub-spoke) layout.
///
/// Groups degree-1 satellite nodes around their hub neighbors.
#[derive(Debug, Clone, Default)]
pub struct WorldBankLayout;

impl WorldBankLayout {
    /// Create a new world bank layout.
    pub fn new() -> Self {
        Self
    }
}

impl NodeLayout for WorldBankLayout {
    fn layout_nodes(
        &self,
        _network: &Network,
        _params: &LayoutParams,
        _monitor: &dyn ProgressMonitor,
    ) -> LayoutResult<Vec<NodeId>> {
        // TODO: Implement world bank layout
        //
        // 1. Find "popular" nodes (hubs) — nodes with many degree-1 neighbors
        // 2. Group degree-1 nodes by their single neighbor
        // 3. Order hub groups by hub degree (descending)
        // 4. Within each group: hub first, then satellites (sorted)
        // 5. Non-hub, non-satellite nodes placed via default BFS
        //
        // See WorldBankLayout.java: doNodeLayout()
        //
        todo!("Implement world bank layout - see WorldBankLayout.java")
    }

    fn name(&self) -> &'static str {
        "World Bank (Hub-Spoke)"
    }
}
