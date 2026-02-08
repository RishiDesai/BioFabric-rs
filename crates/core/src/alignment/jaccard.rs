//! Jaccard similarity for alignment scoring.
//!
//! Computes the average Jaccard similarity of aligned node neighborhoods
//! in the merged network. This implements the JS metric from the Java
//! `JaccardSimilarity` class.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.JaccardSimilarity`

use crate::alignment::merge::MergedNetwork;
use crate::alignment::types::NodeColor;
use crate::model::{Network, NodeId};
use std::collections::HashSet;

/// Jaccard similarity computation for alignment scoring.
pub struct JaccardSimilarity;

impl JaccardSimilarity {
    /// Compute Jaccard similarity of two nodes' neighbor sets in a network.
    pub fn score(network: &Network, a: &NodeId, b: &NodeId) -> f64 {
        let neighbors_a: HashSet<&NodeId> = network.neighbors(a);
        let neighbors_b: HashSet<&NodeId> = network.neighbors(b);

        if neighbors_a.is_empty() && neighbors_b.is_empty() {
            return 1.0;
        }

        let intersection = neighbors_a.intersection(&neighbors_b).count();
        let union = neighbors_a.union(&neighbors_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Compute the average Jaccard similarity across all aligned (purple) node pairs.
    ///
    /// For each aligned node in the merged network, computes the Jaccard similarity
    /// of its neighborhood in the merged network. The final score is the average
    /// across all aligned nodes.
    ///
    /// This implements the Java `JaccardSimilarity.jaccardSimilarityScore()`.
    pub fn average_score(merged: &MergedNetwork) -> f64 {
        let mut total_js = 0.0f64;
        let mut count = 0usize;

        // For each purple node, compute Jaccard similarity of its G1 and G2 neighborhoods
        for (node_id, &color) in &merged.node_colors {
            if color != NodeColor::Purple {
                continue;
            }

            // Get the original G1 and G2 names
            if let Some(origin) = merged.node_origins.get(node_id) {
                if origin.g1.is_some() && origin.g2.is_some() {
                    // Get neighbors in the merged network
                    let neighbors: HashSet<&NodeId> = merged.network.neighbors(node_id);

                    // Split neighbors into G1-sourced (blue/purple via G1) and G2-sourced (red/purple via G2)
                    // In the merged network, neighbors of a purple node that come from G1 edges
                    // are connected via P, pBp, pBb edge types.
                    // Neighbors from G2 edges are connected via P, pRp, pRr edge types.
                    //
                    // For Jaccard, we compare the G1-side neighbor set with the G2-side neighbor set
                    // of the aligned node.
                    //
                    // Java implementation: for each aligned pair (a, b), finds neighbors of a in G1
                    // and neighbors of b in G2, both mapped to merged IDs, then computes Jaccard.
                    //
                    // Since we only have the merged network, we reconstruct: neighbors connected
                    // by G1-type edges vs neighbors connected by G2-type edges.

                    let mut g1_neighbors: HashSet<NodeId> = HashSet::new();
                    let mut g2_neighbors: HashSet<NodeId> = HashSet::new();

                    for (i, link) in merged.network.links_slice().iter().enumerate() {
                        if link.is_shadow {
                            continue;
                        }
                        let is_incident = &link.source == node_id || &link.target == node_id;
                        if !is_incident {
                            continue;
                        }

                        let other = if &link.source == node_id {
                            &link.target
                        } else {
                            &link.source
                        };

                        if let Some(et) = merged.edge_types.get(i) {
                            if et.is_graph1() {
                                g1_neighbors.insert(other.clone());
                            }
                            if et.is_graph2() {
                                g2_neighbors.insert(other.clone());
                            }
                        }
                    }

                    let intersection = g1_neighbors.intersection(&g2_neighbors).count();
                    let union = g1_neighbors.union(&g2_neighbors).count();
                    let js = if union == 0 { 1.0 } else { intersection as f64 / union as f64 };
                    total_js += js;
                    count += 1;
                }
            }
        }

        if count == 0 {
            0.0
        } else {
            total_js / count as f64
        }
    }
}
