//! Basic graph algorithms.
//!
//! This module provides fundamental graph traversal and analysis algorithms.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.analysis.GraphSearcher`

use crate::model::{Network, NodeId};
use std::collections::{HashSet, VecDeque};

/// Perform breadth-first search from a starting node.
///
/// Returns nodes in BFS order (visit order).
///
/// # Arguments
/// * `network` - The network to search
/// * `start` - Starting node ID
///
/// # Returns
/// Vector of node IDs in BFS visit order, or empty if start node doesn't exist.
///
/// # Example
/// ```rust,ignore
/// let order = bfs(&network, &NodeId::new("A"));
/// for node in order {
///     println!("Visited: {}", node);
/// }
/// ```
pub fn bfs(network: &Network, start: &NodeId) -> Vec<NodeId> {
    // TODO: Implement BFS
    //
    // See Java implementation in org.systemsbiology.biofabric.analysis.GraphSearcher
    //
    // Algorithm:
    // 1. Check that start node exists
    // 2. Initialize queue with start node
    // 3. Initialize visited set with start node
    // 4. While queue is not empty:
    //    a. Dequeue next node
    //    b. Add to result
    //    c. For each neighbor not in visited:
    //       - Add to visited
    //       - Enqueue
    // 5. Return result
    //
    // Key behaviors:
    // - Only visits nodes reachable from start
    // - Consistent ordering (sort neighbors before enqueueing for reproducibility)
    //
    todo!("Implement BFS - see GraphSearcher.java")
}

/// Perform depth-first search from a starting node.
///
/// Returns nodes in DFS order (visit order).
///
/// # Arguments
/// * `network` - The network to search
/// * `start` - Starting node ID
///
/// # Returns
/// Vector of node IDs in DFS visit order, or empty if start node doesn't exist.
pub fn dfs(network: &Network, start: &NodeId) -> Vec<NodeId> {
    // TODO: Implement DFS
    //
    // Algorithm:
    // 1. Check that start node exists
    // 2. Initialize stack with start node
    // 3. Initialize visited set
    // 4. While stack is not empty:
    //    a. Pop next node
    //    b. If not visited:
    //       - Mark as visited
    //       - Add to result
    //       - Push all unvisited neighbors onto stack
    // 5. Return result
    //
    // Key behaviors:
    // - Consistent ordering (sort neighbors for reproducibility)
    //
    todo!("Implement DFS")
}

/// Find all connected components in the network.
///
/// Returns a vector of components, where each component is a vector of node IDs.
/// Components are sorted by size (largest first), and nodes within each
/// component are in BFS order from the highest-degree node.
///
/// # Example
/// ```rust,ignore
/// let components = connected_components(&network);
/// println!("Found {} components", components.len());
/// println!("Largest component has {} nodes", components[0].len());
/// ```
pub fn connected_components(network: &Network) -> Vec<Vec<NodeId>> {
    // TODO: Implement connected components
    //
    // Algorithm:
    // 1. Initialize empty result
    // 2. Initialize set of all unvisited nodes
    // 3. While there are unvisited nodes:
    //    a. Find highest-degree unvisited node
    //    b. Run BFS from that node
    //    c. All visited nodes form one component
    //    d. Remove these nodes from unvisited set
    //    e. Add component to result
    // 4. Sort components by size (descending)
    // 5. Return result
    //
    // Key behaviors:
    // - Handles isolated nodes (each is its own component)
    // - Consistent ordering for reproducibility
    //
    todo!("Implement connected components")
}

/// Find the shortest path between two nodes.
///
/// Returns the path as a vector of node IDs (including start and end),
/// or None if no path exists.
///
/// # Arguments
/// * `network` - The network to search
/// * `start` - Starting node ID
/// * `end` - Ending node ID
///
/// # Returns
/// `Some(path)` if a path exists, `None` otherwise.
pub fn shortest_path(network: &Network, start: &NodeId, end: &NodeId) -> Option<Vec<NodeId>> {
    // TODO: Implement shortest path (BFS-based for unweighted graphs)
    //
    // Algorithm:
    // 1. BFS from start, tracking parent of each visited node
    // 2. When end is found, reconstruct path by following parents
    // 3. Return reversed path
    //
    todo!("Implement shortest path")
}

/// Get nodes within N hops of a starting node.
///
/// # Arguments
/// * `network` - The network to search
/// * `start` - Starting node ID  
/// * `hops` - Maximum number of hops (edges) from start
///
/// # Returns
/// Set of node IDs within the specified distance.
pub fn neighborhood(network: &Network, start: &NodeId, hops: usize) -> HashSet<NodeId> {
    // TODO: Implement neighborhood query
    //
    // Algorithm:
    // 1. BFS from start, but track depth
    // 2. Stop when depth exceeds hops
    // 3. Return all visited nodes
    //
    todo!("Implement neighborhood")
}

/// Find the node with highest degree in the network.
///
/// # Returns
/// The node ID with highest degree, or None if network is empty.
pub fn highest_degree_node(network: &Network) -> Option<NodeId> {
    network
        .node_ids()
        .max_by_key(|id| network.degree(id))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Link;

    fn create_test_network() -> Network {
        // A -- B -- C
        //      |
        //      D
        let mut network = Network::new();
        network.add_link(Link::new("A", "B", "r"));
        network.add_link(Link::new("B", "C", "r"));
        network.add_link(Link::new("B", "D", "r"));
        network
    }

    #[test]
    fn test_highest_degree_node() {
        let network = create_test_network();
        let highest = highest_degree_node(&network);
        assert_eq!(highest, Some(NodeId::new("B")));
    }

    // TODO: Enable tests once algorithms are implemented
    //
    // #[test]
    // fn test_bfs_order() {
    //     let network = create_test_network();
    //     let order = bfs(&network, &NodeId::new("B"));
    //     assert_eq!(order.len(), 4);
    //     assert_eq!(order[0], NodeId::new("B")); // Start node first
    // }
    //
    // #[test]
    // fn test_connected_components_single() {
    //     let network = create_test_network();
    //     let components = connected_components(&network);
    //     assert_eq!(components.len(), 1);
    //     assert_eq!(components[0].len(), 4);
    // }
    //
    // #[test]
    // fn test_connected_components_multiple() {
    //     let mut network = Network::new();
    //     network.add_link(Link::new("A", "B", "r"));
    //     network.add_link(Link::new("C", "D", "r"));
    //     network.add_lone_node("E");
    //     
    //     let components = connected_components(&network);
    //     assert_eq!(components.len(), 3);
    // }
}
