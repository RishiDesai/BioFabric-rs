//! Node similarity layout algorithm.
//!
//! This layout orders nodes by their structural similarity, grouping nodes
//! with similar neighborhoods together.
//!
//! ## Algorithm
//!
//! Uses Jaccard similarity of node neighborhoods:
//!
//! ```text
//! similarity(A, B) = |neighbors(A) ∩ neighbors(B)| / |neighbors(A) ∪ neighbors(B)|
//! ```
//!
//! Starting from the highest-degree node, each subsequent node is chosen as
//! the unvisited node most similar to the previously placed node.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.layouts.NodeSimilarityLayout`
//! - Jaccard index: <https://en.wikipedia.org/wiki/Jaccard_index>

use super::traits::{LayoutParams, LayoutResult, NodeLayout};
use crate::model::{Network, NodeId};

/// Node similarity layout algorithm.
///
/// Orders nodes by Jaccard similarity of their neighborhoods, creating
/// visual clusters of structurally similar nodes.
///
/// ## Example
///
/// ```rust,ignore
/// let layout = NodeSimilarityLayout::new();
/// let order = layout.layout_nodes(&network, &Default::default())?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct NodeSimilarityLayout;

impl NodeSimilarityLayout {
    /// Create a new node similarity layout.
    pub fn new() -> Self {
        Self
    }
}

impl NodeLayout for NodeSimilarityLayout {
    fn layout_nodes(
        &self,
        network: &Network,
        params: &LayoutParams,
    ) -> LayoutResult<Vec<NodeId>> {
        // TODO: Implement node similarity layout
        //
        // See Java implementation in org.systemsbiology.biofabric.layouts.NodeSimilarityLayout
        //
        // Algorithm:
        // 1. Precompute neighbor sets for all nodes
        // 2. Start with highest-degree node (or params.start_node)
        // 3. Mark it as visited, add to result
        // 4. While there are unvisited nodes:
        //    a. For each unvisited node, compute Jaccard similarity with last placed node
        //    b. Choose the unvisited node with highest similarity
        //    c. Tie-breaker: higher degree, then lexicographic
        //    d. Add chosen node to result, mark as visited
        // 5. Handle disconnected components: when no similar node found,
        //    start new component from highest-degree unvisited node
        //
        // Jaccard similarity:
        //   J(A,B) = |N(A) ∩ N(B)| / |N(A) ∪ N(B)|
        //   where N(X) is the set of neighbors of X
        //
        // Key behaviors to match:
        // - Nodes with identical neighborhoods cluster together
        // - Handles disconnected components
        // - Consistent ordering for reproducibility
        //
        todo!("Implement node similarity layout - see BioFabric NodeSimilarityLayout.java")
    }

    fn name(&self) -> &'static str {
        "Node Similarity (Jaccard)"
    }
}

/// Compute Jaccard similarity between two sets.
///
/// Returns a value between 0.0 (no overlap) and 1.0 (identical sets).
fn jaccard_similarity<T: Eq + std::hash::Hash>(
    set_a: &std::collections::HashSet<T>,
    set_b: &std::collections::HashSet<T>,
) -> f64 {
    if set_a.is_empty() && set_b.is_empty() {
        return 1.0; // Both empty = identical
    }

    let intersection = set_a.intersection(set_b).count();
    let union = set_a.union(set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_jaccard_identical() {
        let a: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let b: HashSet<i32> = [1, 2, 3].into_iter().collect();
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_disjoint() {
        let a: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let b: HashSet<i32> = [4, 5, 6].into_iter().collect();
        assert!((jaccard_similarity(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_partial() {
        let a: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let b: HashSet<i32> = [2, 3, 4].into_iter().collect();
        // Intersection: {2, 3} = 2
        // Union: {1, 2, 3, 4} = 4
        // Jaccard: 2/4 = 0.5
        assert!((jaccard_similarity(&a, &b) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_jaccard_empty() {
        let a: HashSet<i32> = HashSet::new();
        let b: HashSet<i32> = HashSet::new();
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < 0.001);
    }
}
