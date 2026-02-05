//! Link (edge) representation in BioFabric.
//!
//! In BioFabric visualization:
//! - Each link is assigned to a specific column (x-coordinate)
//! - The link appears as a vertical line segment connecting two node rows

use super::NodeId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A link (edge) between two nodes in the network.
///
/// Corresponds to `FabricLink` in the Java implementation.
///
/// ## Shadow Links
///
/// BioFabric uses "shadow links" to show edges twice - once at each endpoint's
/// natural position. This helps reveal local structure. A link and its shadow
/// share the same source, target, and relation but `is_shadow` differs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Link {
    /// Source node ID.
    pub source: NodeId,

    /// Target node ID.
    pub target: NodeId,

    /// Relationship type or label for this link.
    /// In SIF files, this is the middle column (e.g., "activates", "inhibits").
    pub relation: String,

    /// Whether this link is directed.
    /// `None` means directionality hasn't been determined yet.
    pub directed: Option<bool>,

    /// Whether this is a shadow link.
    ///
    /// Shadow links are duplicates that appear at the "other end" of an edge
    /// to improve visualization of local structure.
    pub is_shadow: bool,
}

impl Link {
    /// Create a new link.
    pub fn new(
        source: impl Into<NodeId>,
        target: impl Into<NodeId>,
        relation: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            relation: relation.into(),
            directed: None,
            is_shadow: false,
        }
    }

    /// Create a new link with shadow status.
    pub fn with_shadow(
        source: impl Into<NodeId>,
        target: impl Into<NodeId>,
        relation: impl Into<String>,
        is_shadow: bool,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            relation: relation.into(),
            directed: None,
            is_shadow,
        }
    }

    /// Check if this is a self-loop (feedback link).
    pub fn is_feedback(&self) -> bool {
        self.source == self.target
    }

    /// Create a flipped version of this link (swap source and target).
    ///
    /// # Panics
    /// Panics if this is a feedback link (self-loop).
    pub fn flipped(&self) -> Self {
        assert!(
            !self.is_feedback(),
            "Cannot flip a feedback link (self-loop)"
        );
        Self {
            source: self.target.clone(),
            target: self.source.clone(),
            relation: self.relation.clone(),
            directed: self.directed,
            is_shadow: self.is_shadow,
        }
    }

    /// Create the shadow version of this link.
    pub fn to_shadow(&self) -> Self {
        Self {
            source: self.source.clone(),
            target: self.target.clone(),
            relation: self.relation.clone(),
            directed: self.directed,
            is_shadow: true,
        }
    }

    /// Check if this link and another form a shadow pair.
    ///
    /// A shadow pair consists of two links that are identical except
    /// one has `is_shadow = true` and the other has `is_shadow = false`.
    pub fn is_shadow_pair(&self, other: &Link) -> bool {
        self.source == other.source
            && self.target == other.target
            && self.relation.to_lowercase() == other.relation.to_lowercase()
            && self.directed == other.directed
            && self.is_shadow != other.is_shadow
    }

    /// Check if this link is synonymous with another.
    ///
    /// Two links are synonymous if they represent the same undirected edge,
    /// possibly with source and target swapped.
    pub fn is_synonymous(&self, other: &Link) -> bool {
        if self == other {
            return true;
        }

        // If either is directed, they can't be synonymous unless equal
        if self.directed == Some(true) || other.directed == Some(true) {
            return false;
        }

        // Check if same relation and shadow status
        if self.relation.to_lowercase() != other.relation.to_lowercase() {
            return false;
        }
        if self.is_shadow != other.is_shadow {
            return false;
        }

        // Check if nodes are swapped
        self.source == other.target && self.target == other.source
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arrow = match self.directed {
            Some(true) => "->",
            Some(false) => "--",
            None => "-?-",
        };
        let shadow = if self.is_shadow { " (shadow)" } else { "" };
        write!(
            f,
            "{} {} {} [{}]{}",
            self.source, arrow, self.target, self.relation, shadow
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let link = Link::new("A", "B", "activates");
        assert_eq!(link.source.as_str(), "A");
        assert_eq!(link.target.as_str(), "B");
        assert_eq!(link.relation, "activates");
        assert!(!link.is_shadow);
        assert!(link.directed.is_none());
    }

    #[test]
    fn test_feedback_detection() {
        let normal = Link::new("A", "B", "rel");
        let feedback = Link::new("A", "A", "rel");

        assert!(!normal.is_feedback());
        assert!(feedback.is_feedback());
    }

    #[test]
    fn test_link_flip() {
        let link = Link::new("A", "B", "rel");
        let flipped = link.flipped();

        assert_eq!(flipped.source.as_str(), "B");
        assert_eq!(flipped.target.as_str(), "A");
        assert_eq!(flipped.relation, "rel");
    }

    #[test]
    #[should_panic(expected = "Cannot flip a feedback link")]
    fn test_flip_feedback_panics() {
        let feedback = Link::new("A", "A", "rel");
        let _ = feedback.flipped();
    }

    #[test]
    fn test_shadow_pair() {
        let link = Link::new("A", "B", "rel");
        let shadow = link.to_shadow();

        assert!(link.is_shadow_pair(&shadow));
        assert!(!link.is_shadow_pair(&link));
    }

    #[test]
    fn test_synonymous() {
        let link1 = Link::new("A", "B", "rel");
        let link2 = Link::new("B", "A", "rel");

        assert!(link1.is_synonymous(&link2));
    }
}
