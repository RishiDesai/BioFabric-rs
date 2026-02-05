//! Network (graph) container for BioFabric.
//!
//! The `Network` struct holds nodes and links and provides methods for
//! querying and manipulating the graph structure.

use super::{Link, Node, NodeId};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io;
use std::path::Path;

/// A network (graph) containing nodes and links.
///
/// This is the primary data structure for BioFabric. It maintains:
/// - A set of nodes (vertices)
/// - A list of links (edges)
/// - Optional: isolated nodes (nodes with no edges)
///
/// Corresponds to `BioFabricNetwork` in the Java implementation,
/// but contains only the graph data, not the layout information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Network {
    /// All nodes in the network, indexed by their ID.
    /// Using IndexMap to preserve insertion order.
    #[serde(with = "indexmap::serde_seq")]
    nodes: IndexMap<NodeId, Node>,

    /// All links in the network.
    links: Vec<Link>,

    /// Nodes that have no incident edges ("lone" nodes).
    /// These are tracked separately because they won't appear
    /// in the links list.
    lone_nodes: IndexSet<NodeId>,
}

impl Network {
    /// Create a new empty network.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a network with pre-allocated capacity.
    pub fn with_capacity(node_capacity: usize, link_capacity: usize) -> Self {
        Self {
            nodes: IndexMap::with_capacity(node_capacity),
            links: Vec::with_capacity(link_capacity),
            lone_nodes: IndexSet::new(),
        }
    }

    // =========================================================================
    // Node operations
    // =========================================================================

    /// Add a node to the network.
    ///
    /// If a node with the same ID already exists, this is a no-op.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.entry(node.id.clone()).or_insert(node);
    }

    /// Add a node by ID, creating a Node if it doesn't exist.
    pub fn add_node_by_id(&mut self, id: impl Into<NodeId>) -> &Node {
        let id = id.into();
        self.nodes
            .entry(id.clone())
            .or_insert_with(|| Node::new(id))
    }

    /// Get a node by its ID.
    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Check if a node exists in the network.
    pub fn contains_node(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Iterate over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Iterate over all node IDs.
    pub fn node_ids(&self) -> impl Iterator<Item = &NodeId> {
        self.nodes.keys()
    }

    /// Add a lone node (a node with no edges).
    pub fn add_lone_node(&mut self, id: impl Into<NodeId>) {
        let id = id.into();
        self.add_node_by_id(id.clone());
        self.lone_nodes.insert(id);
    }

    /// Get lone nodes (nodes with no edges).
    pub fn lone_nodes(&self) -> &IndexSet<NodeId> {
        &self.lone_nodes
    }

    // =========================================================================
    // Link operations
    // =========================================================================

    /// Add a link to the network.
    ///
    /// This also ensures both endpoint nodes exist in the network.
    pub fn add_link(&mut self, link: Link) {
        // Ensure both nodes exist
        self.add_node_by_id(link.source.clone());
        self.add_node_by_id(link.target.clone());

        // Remove from lone nodes if they were there
        self.lone_nodes.shift_remove(&link.source);
        self.lone_nodes.shift_remove(&link.target);

        self.links.push(link);
    }

    /// Get the number of links.
    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Iterate over all links.
    pub fn links(&self) -> impl Iterator<Item = &Link> {
        self.links.iter()
    }

    /// Get links as a slice.
    pub fn links_slice(&self) -> &[Link] {
        &self.links
    }

    /// Get mutable access to links.
    pub fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }

    // =========================================================================
    // Query operations
    // =========================================================================

    /// Get all links incident to a node (as source or target).
    pub fn links_for_node(&self, node_id: &NodeId) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|link| &link.source == node_id || &link.target == node_id)
            .collect()
    }

    /// Get the degree of a node (number of incident edges).
    pub fn degree(&self, node_id: &NodeId) -> usize {
        self.links
            .iter()
            .filter(|link| &link.source == node_id || &link.target == node_id)
            .count()
    }

    /// Get neighbors of a node.
    pub fn neighbors(&self, node_id: &NodeId) -> HashSet<&NodeId> {
        self.links
            .iter()
            .filter_map(|link| {
                if &link.source == node_id {
                    Some(&link.target)
                } else if &link.target == node_id {
                    Some(&link.source)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all unique relation types in the network.
    pub fn relation_types(&self) -> HashSet<&str> {
        self.links.iter().map(|link| link.relation.as_str()).collect()
    }

    // =========================================================================
    // I/O operations
    // =========================================================================

    /// Serialize the network to JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a network from JSON.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// Write the network to a JSON file.
    pub fn to_json_file(&self, path: &Path) -> io::Result<()> {
        let json = self.to_json().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Read a network from a JSON file.
    pub fn from_json_file(path: &Path) -> io::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_json(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_creation() {
        let network = Network::new();
        assert_eq!(network.node_count(), 0);
        assert_eq!(network.link_count(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut network = Network::new();
        network.add_node(Node::new("A"));
        network.add_node(Node::new("B"));

        assert_eq!(network.node_count(), 2);
        assert!(network.contains_node(&NodeId::new("A")));
        assert!(network.contains_node(&NodeId::new("B")));
        assert!(!network.contains_node(&NodeId::new("C")));
    }

    #[test]
    fn test_add_link_creates_nodes() {
        let mut network = Network::new();
        network.add_link(Link::new("A", "B", "activates"));

        assert_eq!(network.node_count(), 2);
        assert_eq!(network.link_count(), 1);
        assert!(network.contains_node(&NodeId::new("A")));
        assert!(network.contains_node(&NodeId::new("B")));
    }

    #[test]
    fn test_degree() {
        let mut network = Network::new();
        network.add_link(Link::new("A", "B", "r1"));
        network.add_link(Link::new("A", "C", "r2"));
        network.add_link(Link::new("B", "C", "r3"));

        assert_eq!(network.degree(&NodeId::new("A")), 2);
        assert_eq!(network.degree(&NodeId::new("B")), 2);
        assert_eq!(network.degree(&NodeId::new("C")), 2);
    }

    #[test]
    fn test_neighbors() {
        let mut network = Network::new();
        network.add_link(Link::new("A", "B", "r1"));
        network.add_link(Link::new("A", "C", "r2"));

        let neighbors = network.neighbors(&NodeId::new("A"));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&NodeId::new("B")));
        assert!(neighbors.contains(&NodeId::new("C")));
    }

    #[test]
    fn test_lone_nodes() {
        let mut network = Network::new();
        network.add_lone_node("isolated");
        network.add_link(Link::new("A", "B", "r1"));

        assert_eq!(network.node_count(), 3);
        assert!(network.lone_nodes().contains(&NodeId::new("isolated")));
        assert!(!network.lone_nodes().contains(&NodeId::new("A")));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut network = Network::new();
        network.add_link(Link::new("A", "B", "activates"));
        network.add_link(Link::new("B", "C", "inhibits"));

        let json = network.to_json().unwrap();
        let restored = Network::from_json(&json).unwrap();

        assert_eq!(restored.node_count(), network.node_count());
        assert_eq!(restored.link_count(), network.link_count());
    }
}
