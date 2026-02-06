//! Jaccard similarity stubs.
//!
//! Parity with Java `JaccardSimilarity`.

use crate::model::{Network, NodeId};

/// Jaccard similarity helper (stub).
pub struct JaccardSimilarity;

impl JaccardSimilarity {
    pub fn score(_network: &Network, _a: &NodeId, _b: &NodeId) -> f64 {
        // TODO: Implement Jaccard similarity of neighbor sets.
        todo!("Implement Jaccard similarity")
    }
}
