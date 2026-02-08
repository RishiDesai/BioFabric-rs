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
use std::collections::{BTreeSet, HashMap};

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
        if edge_types.is_empty() {
            return Self(format!("({}:0)", color_char));
        }
        let types_str: Vec<&str> = edge_types.iter().map(|t| t.short_code()).collect();
        Self(format!("({}:{})", color_char, types_str.join("/")))
    }
}

impl PartialOrd for NodeGroupTag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeGroupTag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Parse color priority: P=0, b=1, r=2 (matching Java's Purple < Blue < Red)
        fn color_priority(tag: &str) -> u8 {
            if tag.starts_with("(P:") {
                0
            } else if tag.starts_with("(b:") {
                1
            } else {
                2
            }
        }

        let cp_self = color_priority(&self.0);
        let cp_other = color_priority(&other.0);
        cp_self.cmp(&cp_other).then_with(|| self.0.cmp(&other.0))
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
    pub fn from_merged(merged: &MergedNetwork, _monitor: &dyn ProgressMonitor) -> Self {
        // Build adjacency: for each node, collect incident edge types
        let mut node_edge_types: HashMap<NodeId, BTreeSet<usize>> = HashMap::new();

        // Initialize all nodes
        for node_id in merged.node_colors.keys() {
            node_edge_types.entry(node_id.clone()).or_default();
        }

        // Collect edge types for each node from the merged network's links
        for (i, link) in merged.network.links_slice().iter().enumerate() {
            if link.is_shadow {
                continue; // Only count non-shadow links for group classification
            }
            let et = merged.edge_types[i];
            let et_idx = EdgeType::all().iter().position(|&e| e == et).unwrap_or(0);

            node_edge_types
                .entry(link.source.clone())
                .or_default()
                .insert(et_idx);
            node_edge_types
                .entry(link.target.clone())
                .or_default()
                .insert(et_idx);
        }

        // Group nodes by (color, edge_type_set) -> tag
        let mut tag_to_nodes: HashMap<NodeGroupTag, Vec<NodeId>> = HashMap::new();
        let mut tag_to_meta: HashMap<NodeGroupTag, (NodeColor, Vec<EdgeType>)> = HashMap::new();

        for (node_id, et_indices) in &node_edge_types {
            let color = merged.node_colors.get(node_id).copied().unwrap_or(NodeColor::Purple);
            let edge_types_sorted: Vec<EdgeType> = et_indices
                .iter()
                .map(|&idx| EdgeType::all()[idx])
                .collect();

            let tag = NodeGroupTag::from_parts(color, &edge_types_sorted);

            tag_to_nodes
                .entry(tag.clone())
                .or_default()
                .push(node_id.clone());
            tag_to_meta
                .entry(tag)
                .or_insert((color, edge_types_sorted));
        }

        // Sort groups canonically
        let mut sorted_tags: Vec<NodeGroupTag> = tag_to_nodes.keys().cloned().collect();
        sorted_tags.sort();

        // Build groups
        let mut groups = Vec::new();
        let mut node_to_group = HashMap::new();

        for (group_idx, tag) in sorted_tags.iter().enumerate() {
            let (color, edge_types) = tag_to_meta.get(tag).cloned().unwrap();
            let mut members = tag_to_nodes.remove(tag).unwrap_or_default();
            // Sort members alphabetically within each group
            members.sort();

            for node_id in &members {
                node_to_group.insert(node_id.clone(), group_idx);
            }

            groups.push(NodeGroup {
                tag: tag.clone(),
                color,
                edge_types,
                members,
            });
        }

        NodeGroupMap {
            groups,
            node_to_group,
            perfect_mode: PerfectNGMode::None,
            jaccard_threshold: DEFAULT_JACCARD_THRESHOLD,
        }
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

    /// Compute the link group ratio vector.
    ///
    /// For each edge type, counts the edges of that type and divides by total.
    /// Used for LGS (Link Group Similarity) scoring.
    pub fn link_ratio_vector(merged: &MergedNetwork) -> Vec<f64> {
        let all_types = EdgeType::all();
        let mut counts = vec![0usize; all_types.len()];
        let mut total = 0usize;

        for (i, link) in merged.network.links_slice().iter().enumerate() {
            if link.is_shadow {
                continue;
            }
            let et = merged.edge_types[i];
            if let Some(idx) = all_types.iter().position(|&e| e == et) {
                counts[idx] += 1;
                total += 1;
            }
        }

        if total == 0 {
            return vec![0.0; all_types.len()];
        }
        counts.iter().map(|&c| c as f64 / total as f64).collect()
    }
}
