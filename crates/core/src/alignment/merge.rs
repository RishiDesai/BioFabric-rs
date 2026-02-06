//! Network merging for alignment visualization.
//!
//! Given two networks (G1, G2) and an alignment mapping, produces a single
//! merged network where aligned nodes are combined and edges are classified
//! by their alignment status.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.NetworkAlignment.mergeNetworks()`

use super::types::{EdgeType, MergedNodeId, NodeColor};
use crate::io::align::AlignmentMap;
use crate::model::{Network, NodeId};
use crate::worker::ProgressMonitor;
use std::collections::HashMap;

/// The result of merging two networks via an alignment.
///
/// Contains the merged network plus metadata about each node and edge's
/// alignment classification.
#[derive(Debug, Clone)]
pub struct MergedNetwork {
    /// The merged network (nodes use `"G1::G2"` naming convention).
    pub network: Network,

    /// Color classification for each merged node.
    pub node_colors: HashMap<NodeId, NodeColor>,

    /// Edge type classification for each link (by link index in the network).
    pub edge_types: Vec<EdgeType>,

    /// Map from merged node IDs back to their original components.
    pub node_origins: HashMap<NodeId, MergedNodeId>,

    /// Per-node correctness when a perfect (reference) alignment is provided.
    ///
    /// Maps merged node IDs to `true` if the alignment matches the perfect
    /// alignment for that node. Populated only when a perfect alignment is
    /// provided; `None` otherwise.
    ///
    /// For **purple** (aligned) nodes:
    ///   `true` if `alignment[g1] == perfect_alignment[g1]`
    ///
    /// For **blue** (unaligned G1) nodes:
    ///   `true` if the perfect alignment also does NOT align this G1 node
    ///   (i.e., `perfect_alignment[g1]` is `None`)
    ///
    /// Red (G2-only) nodes are not tracked (the Java code doesn't track them
    /// either — correctness is only meaningful for G1 nodes).
    ///
    /// ## References
    ///
    /// - Java: `NetworkAlignment.mergedToCorrectNC_`
    /// - Java: `NetworkAlignment.createMergedNodes()` (line 277)
    /// - Java: `NetworkAlignment.createUnmergedNodes()` (line 322)
    pub merged_to_correct: Option<HashMap<NodeId, bool>>,

    /// Number of nodes from G1.
    pub g1_node_count: usize,

    /// Number of nodes from G2.
    pub g2_node_count: usize,

    /// Number of aligned (purple) nodes.
    pub aligned_count: usize,
}

impl MergedNetwork {
    /// Merge two networks using an alignment mapping.
    ///
    /// # Arguments
    ///
    /// * `g1` — The smaller network
    /// * `g2` — The larger network
    /// * `alignment` — Mapping from G1 node IDs to G2 node IDs
    /// * `perfect` — Optional perfect (reference) alignment for correctness
    ///   tracking. When provided, `merged_to_correct` is populated.
    /// * `monitor` — Progress/cancellation monitor
    ///
    /// # Returns
    ///
    /// A `MergedNetwork` containing the combined graph with classified nodes
    /// and edges.
    pub fn from_alignment(
        _g1: &Network,
        _g2: &Network,
        _alignment: &AlignmentMap,
        _perfect: Option<&AlignmentMap>,
        _monitor: &dyn ProgressMonitor,
    ) -> Result<Self, String> {
        // TODO: Implement network merging
        //
        // Algorithm (see NetworkAlignment.mergeNetworks()):
        //
        // 1. Create merged nodes:
        //    a. For each (g1_node, g2_node) in alignment: create purple node "g1::g2"
        //       - If `perfect` is Some, compute correctness:
        //         `correct = perfect[g1] == Some(g2)` and store in merged_to_correct
        //    b. For each G1 node NOT in alignment: create blue node "g1::"
        //       - If `perfect` is Some, compute correctness:
        //         `correct = perfect[g1].is_none()` (correctly unaligned if perfect
        //         also doesn't align it)
        //    c. For each G2 node NOT in alignment values: create red node "::g2"
        //       - Red nodes do NOT get correctness entries (Java doesn't track them)
        //
        // 2. Build lookup maps:
        //    a. g1_name -> merged_node_id
        //    b. g2_name -> merged_node_id
        //
        // 3. Transform edges:
        //    a. For each edge in G1: map source/target to merged IDs, add to merged network
        //    b. For each edge in G2: map source/target to merged IDs, add to merged network
        //    c. Deduplicate: edges present in both G1 and G2 (via aligned endpoints) = COVERED
        //
        // 4. Classify edges:
        //    For each merged edge, determine EdgeType based on endpoint colors:
        //    - Both purple, exists in both -> Covered
        //    - Both purple, G1 only -> InducedGraph1
        //    - Purple + Blue -> HalfOrphanGraph1
        //    - Both Blue -> FullOrphanGraph1
        //    - Both purple, G2 only -> InducedGraph2
        //    - Purple + Red -> HalfUnalignedGraph2
        //    - Both Red -> FullUnalignedGraph2
        //
        // 5. Populate counts and return
        //    - Set merged_to_correct = Some(map) if perfect.is_some(), else None
        //
        todo!("Implement network merging - see NetworkAlignment.mergeNetworks()")
    }

    /// Count of nodes by color.
    pub fn count_by_color(&self, color: NodeColor) -> usize {
        self.node_colors.values().filter(|&&c| c == color).count()
    }

    /// Count of edges by type.
    pub fn count_by_edge_type(&self, edge_type: EdgeType) -> usize {
        self.edge_types.iter().filter(|&&t| t == edge_type).count()
    }

    /// Get the node color for a merged node.
    pub fn node_color(&self, node_id: &NodeId) -> Option<NodeColor> {
        self.node_colors.get(node_id).copied()
    }

    /// Get the original (G1/G2) components for a merged node.
    pub fn node_origin(&self, node_id: &NodeId) -> Option<&MergedNodeId> {
        self.node_origins.get(node_id)
    }

    /// Get the edge type for a link index in the merged network.
    pub fn edge_type(&self, link_index: usize) -> Option<EdgeType> {
        self.edge_types.get(link_index).copied()
    }

    /// Whether a node is aligned (purple).
    pub fn is_aligned_node(&self, node_id: &NodeId) -> bool {
        matches!(self.node_color(node_id), Some(NodeColor::Purple))
    }

    /// Whether a perfect alignment was provided (i.e., correctness data is available).
    pub fn has_perfect_alignment(&self) -> bool {
        self.merged_to_correct.is_some()
    }

    /// Whether a node is correctly aligned/unaligned according to the perfect alignment.
    ///
    /// Returns `None` if no perfect alignment was provided, or if the node
    /// is not tracked (e.g., red/G2-only nodes).
    pub fn is_node_correct(&self, node_id: &NodeId) -> Option<bool> {
        self.merged_to_correct.as_ref()?.get(node_id).copied()
    }

    /// Number of correctly aligned/unaligned nodes (from `merged_to_correct`).
    ///
    /// Returns `None` if no perfect alignment was provided.
    pub fn correct_count(&self) -> Option<usize> {
        Some(
            self.merged_to_correct
                .as_ref()?
                .values()
                .filter(|&&v| v)
                .count(),
        )
    }

    /// Node correctness (NC) metric: fraction of tracked nodes that are correct.
    ///
    /// NC = |{n : merged_to_correct[n] = true}| / |merged_to_correct|
    ///
    /// Returns `None` if no perfect alignment was provided.
    pub fn node_correctness(&self) -> Option<f64> {
        let map = self.merged_to_correct.as_ref()?;
        if map.is_empty() {
            return Some(0.0);
        }
        let correct = map.values().filter(|&&v| v).count();
        Some(correct as f64 / map.len() as f64)
    }
}
