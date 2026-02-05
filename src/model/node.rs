//! Node representation in BioFabric.
//!
//! In BioFabric visualization:
//! - Each node is assigned to a specific row (y-coordinate)
//! - The node spans horizontally from its first to last incident edge

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a node.
///
/// This is a wrapper around a string name. In the Java implementation,
/// nodes have both a numeric ID and a display name. For simplicity,
/// we use the name as the ID directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create a new NodeId from a string.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the name/ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A node in the BioFabric network.
///
/// Corresponds to `FabricNode` in the Java implementation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,
}

impl Node {
    /// Create a new node with the given ID.
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self { id: id.into() }
    }

    /// Get the node's name (same as ID).
    pub fn name(&self) -> &str {
        self.id.as_str()
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new("test_node");
        assert_eq!(node.name(), "test_node");
        assert_eq!(node.id.as_str(), "test_node");
    }

    #[test]
    fn test_node_equality() {
        let node1 = Node::new("A");
        let node2 = Node::new("A");
        let node3 = Node::new("B");

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_nodeid_ordering() {
        let a = NodeId::new("A");
        let b = NodeId::new("B");
        let a2 = NodeId::new("A");

        assert!(a < b);
        assert_eq!(a, a2);
    }
}
