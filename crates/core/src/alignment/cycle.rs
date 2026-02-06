//! Alignment cycle and path detection.
//!
//! When an alignment is imperfect, following the mapping from G1→G2 and back
//! can reveal **cycles** (closed loops of misalignment) and **paths** (open
//! chains ending at unaligned nodes).
//!
//! ## The 9 Cases (from the Java implementation)
//!
//! The Java `AlignCycleLayout` classifies every alignment structure into
//! exactly one of 9 cases:
//!
//! | Case | Structure | Description |
//! |------|-----------|-------------|
//! | 1 | `b` | Correctly unaligned blue (G1-only) node |
//! | 2 | `r` | Correctly unaligned red (G2-only) node |
//! | 3 | `(p)` | Correctly aligned purple singleton (1-cycle) |
//! | 4 | `p` | Incorrectly aligned purple singleton path |
//! | 5 | `r → b` | Incorrect 2-node path (one red, one blue) |
//! | 6 | `r → p → … → p` | Incorrect path: red + N purple nodes |
//! | 7 | `p → … → p → b` | Incorrect path: N purple nodes + blue |
//! | 8 | `r → p → … → p → b` | Incorrect path: red + N purple + blue |
//! | 9 | `(p → … → p)` | Incorrect N-purple cycle (N > 1) |
//!
//! Cases 1–3 are **correct** alignments; cases 4–9 are **incorrect**.
//! A perfect (reference) alignment is required to distinguish case 3
//! (correct singleton) from case 4 (incorrect singleton).
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.AlignCycleLayout`
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.AlignCycleRelation`

use crate::io::align::AlignmentMap;
use crate::model::NodeId;
use crate::worker::ProgressMonitor;

/// Classification of an alignment cycle or path.
///
/// Each variant corresponds to one of the 9 cases from the Java
/// `AlignCycleLayout` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CycleCase {
    /// Case 1: Correctly unaligned blue (G1-only) node.
    ///
    /// This node exists in G1 but not in G2, and the perfect alignment
    /// confirms it should not be aligned.
    CorrectlyUnalignedBlue,

    /// Case 2: Correctly unaligned red (G2-only) node.
    ///
    /// This node exists in G2 but not in G1, and the perfect alignment
    /// confirms it should not be aligned.
    CorrectlyUnalignedRed,

    /// Case 3: Correctly aligned purple singleton (1-cycle).
    ///
    /// This G1 node maps to a G2 node, and the perfect alignment agrees.
    /// `main_alignment[g1] == perfect_alignment[g1]`.
    CorrectSingleton,

    /// Case 4: Incorrectly aligned purple singleton path.
    ///
    /// A single aligned node whose mapping disagrees with the perfect
    /// alignment. In cycle terms, this is a path of length 1.
    IncorrectSingleton,

    /// Case 5: Incorrect 2-node path — one red node and one blue node.
    ///
    /// `r → b`: a G2-only node that should have been aligned to a G1-only
    /// node (or vice versa).
    PathRedBlue,

    /// Case 6: Incorrect path starting with red, followed by N purple nodes.
    ///
    /// `r → p → … → p`: the red endpoint is a G2 node that should have been
    /// aligned differently; the chain of purple nodes are all misaligned.
    PathRedPurple,

    /// Case 7: Incorrect path with N purple nodes ending in blue.
    ///
    /// `p → … → p → b`: the blue endpoint is a G1 node that should have been
    /// aligned; the chain of purple nodes are all misaligned.
    PathPurpleBlue,

    /// Case 8: Incorrect path: red + N purple + blue.
    ///
    /// `r → p → … → p → b`: the most general path form with both
    /// unaligned endpoints and a chain of misaligned purple nodes.
    PathRedPurpleBlue,

    /// Case 9: Incorrect cycle of N purple nodes (N > 1).
    ///
    /// `(p → … → p)`: all nodes are aligned but form a closed cycle of
    /// misalignment — following the mapping loops back to the start.
    IncorrectCycle,
}

impl CycleCase {
    /// All 9 cases in canonical order (matches Java ordering).
    pub fn all() -> &'static [CycleCase] {
        &[
            CycleCase::CorrectlyUnalignedBlue,
            CycleCase::CorrectlyUnalignedRed,
            CycleCase::CorrectSingleton,
            CycleCase::IncorrectSingleton,
            CycleCase::PathRedBlue,
            CycleCase::PathRedPurple,
            CycleCase::PathPurpleBlue,
            CycleCase::PathRedPurpleBlue,
            CycleCase::IncorrectCycle,
        ]
    }

    /// Zero-based index of this case (0–8).
    pub fn index(&self) -> usize {
        match self {
            CycleCase::CorrectlyUnalignedBlue => 0,
            CycleCase::CorrectlyUnalignedRed => 1,
            CycleCase::CorrectSingleton => 2,
            CycleCase::IncorrectSingleton => 3,
            CycleCase::PathRedBlue => 4,
            CycleCase::PathRedPurple => 5,
            CycleCase::PathPurpleBlue => 6,
            CycleCase::PathRedPurpleBlue => 7,
            CycleCase::IncorrectCycle => 8,
        }
    }

    /// Whether this case represents a correct alignment.
    pub fn is_correct(&self) -> bool {
        matches!(
            self,
            CycleCase::CorrectlyUnalignedBlue
                | CycleCase::CorrectlyUnalignedRed
                | CycleCase::CorrectSingleton
        )
    }

    /// Whether this case is a cycle (as opposed to a path or singleton).
    pub fn is_cycle(&self) -> bool {
        matches!(
            self,
            CycleCase::CorrectSingleton | CycleCase::IncorrectCycle
        )
    }
}

/// A single detected cycle or path in the alignment.
#[derive(Debug, Clone)]
pub struct AlignmentCyclePath {
    /// The case classification.
    pub case: CycleCase,

    /// Ordered list of node IDs forming the cycle/path.
    ///
    /// For cycles, the first and last node are the same. For paths, the
    /// endpoints are unaligned (blue/red) nodes.
    pub nodes: Vec<NodeId>,
}

/// All detected cycles and paths in an alignment.
#[derive(Debug, Clone)]
pub struct AlignmentCycles {
    /// All detected cycles/paths.
    pub entries: Vec<AlignmentCyclePath>,

    /// Count by case type (indexed by `CycleCase::index()`).
    pub case_counts: [usize; 9],
}

impl AlignmentCycles {
    /// Detect all cycles and paths in an alignment.
    ///
    /// # Arguments
    ///
    /// * `alignment` — The G1→G2 alignment mapping
    /// * `perfect` — Optional perfect (reference) alignment for correctness
    ///   classification. Without this, cases 1–3 cannot be distinguished from
    ///   cases 4+ and all singletons are treated as `IncorrectSingleton`.
    /// * `monitor` — Progress/cancellation monitor
    ///
    /// # Algorithm
    ///
    /// See `AlignCycleLayout.calcAlignPathsV2()` in the Java implementation:
    ///
    /// 1. **Normalize namespaces**: If G1 and G2 use the same node names,
    ///    use an identity map. Otherwise, use the inverse of the perfect
    ///    alignment to translate G2 names back to G1 names.
    ///
    /// 2. **Build bidirectional mapping**:
    ///    - Forward: G1_a → G2_b (from alignment)
    ///    - Reverse: G2_b → G1_c (inverted alignment, or from perfect)
    ///
    /// 3. **Trace chains**: For each unvisited G1 node, follow:
    ///    ```text
    ///    G1_a → G2_b → G1_c → G2_d → …
    ///    ```
    ///    until we either:
    ///    - Return to the start (cycle, case 3 or 9)
    ///    - Reach an unaligned endpoint (path, cases 5–8)
    ///    - Find a lone unaligned node (cases 1–2)
    ///
    /// 4. **Classify** each chain into one of the 9 cases.
    ///
    /// 5. **Collect** all cycles/paths and count by case.
    pub fn detect(
        _alignment: &AlignmentMap,
        _perfect: Option<&AlignmentMap>,
        _monitor: &dyn ProgressMonitor,
    ) -> Self {
        // TODO: Implement cycle/path detection
        //
        // See AlignCycleLayout.calcAlignPathsV2() in Java.
        //
        todo!("Implement alignment cycle detection")
    }

    /// Total number of cycles/paths detected.
    pub fn total(&self) -> usize {
        self.entries.len()
    }

    /// Count for a specific case.
    pub fn count(&self, case: CycleCase) -> usize {
        self.case_counts[case.index()]
    }

    /// Number of correctly aligned structures (cases 1–3).
    pub fn correct_count(&self) -> usize {
        self.case_counts[0] + self.case_counts[1] + self.case_counts[2]
    }

    /// Number of incorrectly aligned structures (cases 4–9).
    pub fn incorrect_count(&self) -> usize {
        self.case_counts[3..]
            .iter()
            .sum()
    }
}
