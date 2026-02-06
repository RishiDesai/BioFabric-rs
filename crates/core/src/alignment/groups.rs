//! Node group classification for alignment visualization.
//!
//! Each node in the merged network is assigned to a **node group** based on
//! its color (Purple/Blue/Red) and the set of edge types incident to it.
//! For example, a purple node with both COVERED and INDUCED_GRAPH1 edges
//! belongs to group `(P:P/pBp)`.
//!
//! The Java implementation produces 40-76 distinct groups depending on the
//! network. These groups drive both the layout ordering and the color
//! assignments in the visualization.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.NodeGroupMap`

use super::merge::MergedNetwork;
use super::types::{EdgeType, NodeColor};
use crate::model::NodeId;
use crate::worker::ProgressMonitor;
use std::collections::HashMap;

/// How to subdivide node groups when a perfect alignment is available.
///
/// Without a perfect alignment, there are ~40 base node groups. With a
/// perfect alignment, each base group can be split into "correct" and
/// "incorrect" subgroups, yielding ~76 groups.
///
/// ## References
///
/// - Java: `NodeGroupMap.PerfectNGMode`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PerfectNGMode {
    /// No perfect alignment — use base groups only (~40 groups).
    #[default]
    None,
    /// Split groups by node correctness (does alignment match perfect?).
    /// Produces ~76 groups (each base group split into correct/incorrect).
    NodeCorrectness,
    /// Split groups by Jaccard similarity threshold (default 0.75).
    /// Produces ~76 groups (each base group split into above/below threshold).
    JaccardSimilarity,
}

/// Jaccard similarity threshold for [`PerfectNGMode::JaccardSimilarity`].
///
/// ## References
///
/// - Java: `NodeGroupMap.JACCARD_THRESHOLD` (0.75)
pub const DEFAULT_JACCARD_THRESHOLD: f64 = 0.75;

/// Tag identifying a node group.
///
/// Format: `"(color:edge_type1/edge_type2/...)"`, where color is `P`, `b`,
/// or `r` and edge types are in canonical order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeGroupTag(pub String);

impl NodeGroupTag {
    /// Build a tag from a node color and its incident edge types.
    pub fn from_parts(color: NodeColor, edge_types: &[EdgeType]) -> Self {
        let color_char = match color {
            NodeColor::Purple => "P",
            NodeColor::Blue => "b",
            NodeColor::Red => "r",
        };
        let types_str: Vec<&str> = edge_types.iter().map(|t| t.short_code()).collect();
        Self(format!("({}:{})", color_char, types_str.join("/")))
    }
}

/// A single node group — a set of nodes sharing the same color + edge pattern.
#[derive(Debug, Clone)]
pub struct NodeGroup {
    /// The group tag (e.g., `"(P:P/pBp)"`).
    pub tag: NodeGroupTag,
    /// Node color for all members.
    pub color: NodeColor,
    /// Edge types incident to members of this group.
    pub edge_types: Vec<EdgeType>,
    /// Member node IDs.
    pub members: Vec<NodeId>,
}

/// Complete node group classification for a merged network.
///
/// Maps each node to its group and provides group-level statistics.
#[derive(Debug, Clone)]
pub struct NodeGroupMap {
    /// All groups, in canonical display order.
    pub groups: Vec<NodeGroup>,
    /// Lookup: node ID -> index into `groups`.
    pub node_to_group: HashMap<NodeId, usize>,
    /// The perfect NG mode used to build this map.
    pub perfect_mode: PerfectNGMode,
    /// Jaccard threshold (only meaningful when mode is JaccardSimilarity).
    pub jaccard_threshold: f64,
}

impl NodeGroupMap {
    /// Build the node group map from a merged network.
    pub fn from_merged(_merged: &MergedNetwork, _monitor: &dyn ProgressMonitor) -> Self {
        // TODO: Implement node group classification
        //
        // Algorithm (see NodeGroupMap.java):
        //
        // 1. For each node in the merged network:
        //    a. Determine its color from merged.node_colors
        //    b. Collect the set of EdgeTypes for all incident edges
        //    c. Sort edge types in canonical order
        //    d. Build NodeGroupTag from (color, sorted_edge_types)
        //
        // 2. Group nodes by their tag
        //
        // 3. Order groups canonically:
        //    a. Purple groups first, then Blue, then Red
        //    b. Within a color, order by edge type pattern
        //
        // 4. Within each group, order nodes (e.g., by degree or alphabetically)
        //
        todo!("Implement node group classification - see NodeGroupMap.java")
    }

    /// Compute the group ratio vector (fraction of nodes in each group).
    ///
    /// Used for NGS (Node Group Similarity) scoring.
    pub fn ratio_vector(&self) -> Vec<f64> {
        let total: usize = self.groups.iter().map(|g| g.members.len()).sum();
        if total == 0 {
            return vec![0.0; self.groups.len()];
        }
        self.groups
            .iter()
            .map(|g| g.members.len() as f64 / total as f64)
            .collect()
    }
}
