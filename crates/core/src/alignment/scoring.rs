//! Alignment quality scoring metrics.
//!
//! Computes topological and evaluation measures for a network alignment.
//!
//! ## Topological Measures (no reference alignment needed)
//!
//! - **EC** (Edge Coverage): `covered / (covered + induced_G1)`
//! - **S3** (Symmetric Substructure Score): `covered / (covered + induced_G1 + induced_G2)`
//! - **ICS** (Induced Conserved Substructure): `covered / (covered + induced_G2)`
//!
//! ## Evaluation Measures (require a known-correct "perfect" alignment)
//!
//! - **NC** (Node Correctness): fraction of correctly aligned nodes
//! - **NGS** (Node Group Similarity): angular similarity of node group ratio vectors
//! - **LGS** (Link Group Similarity): angular similarity of link group ratio vectors
//! - **JS** (Jaccard Similarity): average Jaccard similarity of aligned node neighborhoods
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.NetworkAlignmentScorer`
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.JaccardSimilarity`

use super::merge::MergedNetwork;
use crate::io::align::AlignmentMap;
use crate::worker::ProgressMonitor;
use serde::{Deserialize, Serialize};

/// All computed alignment quality scores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlignmentScores {
    // -- Topological measures --
    /// Edge Coverage: covered / (covered + induced_G1).
    pub ec: f64,
    /// Symmetric Substructure Score: covered / (covered + induced_G1 + induced_G2).
    pub s3: f64,
    /// Induced Conserved Substructure: covered / (covered + induced_G2).
    pub ics: f64,

    // -- Evaluation measures (only populated if a perfect alignment is provided) --
    /// Node Correctness: fraction of nodes aligned to their correct partner.
    pub nc: Option<f64>,
    /// Node Group Similarity: angular similarity of node group ratio vectors.
    pub ngs: Option<f64>,
    /// Link Group Similarity: angular similarity of link group ratio vectors.
    pub lgs: Option<f64>,
    /// Jaccard Similarity: average Jaccard similarity of aligned neighborhoods.
    pub js: Option<f64>,
}

impl AlignmentScores {
    /// Compute topological scores from a merged network.
    ///
    /// These metrics only require the merged network (no reference alignment).
    pub fn topological(_merged: &MergedNetwork, _monitor: &dyn ProgressMonitor) -> Self {
        // TODO: Implement topological scoring
        //
        // 1. Count edges by type using merged.count_by_edge_type()
        // 2. Compute EC = covered / (covered + induced_g1)
        // 3. Compute S3 = covered / (covered + induced_g1 + induced_g2)
        // 4. Compute ICS = covered / (covered + induced_g2)
        //
        // See NetworkAlignmentScorer.java: calcTopologicalMeasures()
        //
        todo!("Implement topological scoring")
    }

    /// Compute evaluation scores by comparing to a known-correct alignment.
    ///
    /// Requires both the merged network and the perfect (reference) alignment.
    pub fn with_evaluation(
        _merged: &MergedNetwork,
        _perfect_alignment: &AlignmentMap,
        _monitor: &dyn ProgressMonitor,
    ) -> Self {
        // TODO: Implement evaluation scoring
        //
        // 1. Compute NC: compare each aligned pair to the perfect alignment
        // 2. Compute NGS: build node group vectors for main and perfect, compute angular similarity
        // 3. Compute LGS: build link group vectors for main and perfect, compute angular similarity
        // 4. Compute JS: for each aligned node pair, compute Jaccard similarity of neighborhoods
        //
        // See NetworkAlignmentScorer.java: calcNodeCorrectness(), calcGroupSimilarity()
        // See JaccardSimilarity.java
        //
        todo!("Implement evaluation scoring")
    }
}

/// Compute Jaccard similarity between two sets.
///
/// Returns a value in `[0.0, 1.0]`. Two empty sets are considered identical (1.0).
pub fn jaccard_similarity<T: Eq + std::hash::Hash>(
    set_a: &std::collections::HashSet<T>,
    set_b: &std::collections::HashSet<T>,
) -> f64 {
    if set_a.is_empty() && set_b.is_empty() {
        return 1.0;
    }
    let intersection = set_a.intersection(set_b).count();
    let union = set_a.union(set_b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Compute angular (cosine) similarity between two vectors.
///
/// Returns a value in `[0.0, 1.0]`.
pub fn angular_similarity(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    let cosine = (dot / (mag_a * mag_b)).clamp(-1.0, 1.0);
    1.0 - cosine.acos() / std::f64::consts::FRAC_PI_2
}
