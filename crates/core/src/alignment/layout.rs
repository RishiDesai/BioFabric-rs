//! Alignment-specific layout modes.
//!
//! Three layout modes are provided for visualizing network alignments:
//!
//! ## GROUP Layout
//!
//! Nodes are ordered by their [`NodeGroupMap`] classification. Within each
//! group, nodes are ordered by BFS. Group boundaries are marked with
//! color-coded annotations.
//!
//! ## ORPHAN Layout
//!
//! Filters the merged network to show only "orphan" edges (unaligned edges)
//! plus sufficient context. Uses the default BFS layout on the filtered
//! subgraph.
//!
//! ## CYCLE Layout
//!
//! Orders nodes to highlight alignment cycles and paths. Misaligned nodes
//! forming a cycle are placed in adjacent rows with alternating-color
//! annotations.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.NetworkAlignmentLayout`
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.AlignCycleLayout`
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.NetworkAlignmentEdgeLayout`
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.AlignCycleEdgeLayout`

use super::groups::NodeGroupMap;
use super::merge::MergedNetwork;
use super::cycle::AlignmentCycles;
use super::cycle_relation::CyclePositionMap;
use super::orphan::OrphanFilter;
use super::types::EdgeType;
use crate::layout::build_data::{CycleBound, LayoutBuildData};
use crate::layout::traits::{EdgeLayout, LayoutParams, LayoutResult, NodeLayout};
use crate::layout::result::NetworkLayout;
use crate::model::{Annotation, AnnotationSet, Network, NodeId};
use crate::worker::ProgressMonitor;
use std::collections::{HashMap, HashSet};

/// Which alignment layout mode to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlignmentLayoutMode {
    /// Group layout: order by node group classification.
    #[default]
    Group,
    /// Orphan layout: show unaligned edges with context.
    Orphan,
    /// Cycle layout: highlight alignment cycles/paths.
    Cycle,
}

/// Alignment-aware node layout.
///
/// Wraps a [`MergedNetwork`] and applies one of the three alignment layout
/// modes to produce a node ordering.
pub struct AlignmentNodeLayout {
    /// The merged alignment network.
    pub merged: MergedNetwork,
    /// Which layout mode to use.
    pub mode: AlignmentLayoutMode,
    /// Precomputed node groups (used by Group and Cycle modes).
    pub groups: Option<NodeGroupMap>,
    /// Precomputed cycles (used by Cycle mode).
    pub cycles: Option<AlignmentCycles>,
}

impl AlignmentNodeLayout {
    /// Create a new alignment node layout.
    pub fn new(merged: MergedNetwork, mode: AlignmentLayoutMode) -> Self {
        Self {
            merged,
            mode,
            groups: None,
            cycles: None,
        }
    }

    /// Attach precomputed node groups.
    pub fn with_groups(mut self, groups: NodeGroupMap) -> Self {
        self.groups = Some(groups);
        self
    }

    /// Attach precomputed cycle detection results.
    pub fn with_cycles(mut self, cycles: AlignmentCycles) -> Self {
        self.cycles = Some(cycles);
        self
    }
}

impl NodeLayout for AlignmentNodeLayout {
    fn layout_nodes(
        &self,
        _network: &Network,
        _params: &LayoutParams,
        _monitor: &dyn ProgressMonitor,
    ) -> LayoutResult<Vec<NodeId>> {
        // TODO: Implement alignment node layout
        //
        // Dispatch based on self.mode:
        //
        // GROUP mode (see NetworkAlignmentLayout.java):
        //   1. Build NodeGroupMap if not already provided
        //   2. For each group (in canonical order):
        //      a. BFS within the group's members
        //      b. Append to result
        //   3. Create group annotations with appropriate colors
        //
        // ORPHAN mode (see orphan::OrphanFilter):
        //   1. Use OrphanFilter::filter(&self.merged) to get filtered subnetwork
        //   2. The filter includes orphan edges + context nodes
        //   3. Apply DefaultNodeLayout to the filtered subgraph
        //
        // CYCLE mode (see AlignCycleLayout.java):
        //   1. Detect cycles if not already provided
        //   2. Order: correct singletons first, then cycles/paths
        //   3. Within each cycle/path, unroll nodes in chain order
        //   4. Create cycle/path annotations
        //
        todo!("Implement alignment node layout")
    }

    fn name(&self) -> &'static str {
        match self.mode {
            AlignmentLayoutMode::Group => "Alignment (Group)",
            AlignmentLayoutMode::Orphan => "Alignment (Orphan)",
            AlignmentLayoutMode::Cycle => "Alignment (Cycle)",
        }
    }
}

/// Alignment-aware edge layout.
///
/// Orders edges to reflect the alignment structure. In GROUP mode, edges
/// are ordered by edge type priority. In CYCLE mode, edges are ordered
/// to visually follow the cycle structure with per-cycle/path annotations.
///
/// ## Modes
///
/// - **GROUP**: Sets link group order to the 7 edge types. Within each type,
///   uses the default edge ordering. Link annotation colors are `null` (grayscale)
///   because node annotations already provide color.
///
/// - **CYCLE**: Extends the default edge layout with cycle-aware link annotations.
///   For each detected cycle/path, creates an annotation spanning its edges,
///   alternating Orange/Green. Correctly-aligned singletons are not annotated.
///   Link annotation colors for edge types are fixed: Purple (G12), PowderBlue
///   (G1A), Pink (G2A).
///
/// - **ORPHAN**: Uses the default edge layout on the filtered subgraph.
///
/// ## References
///
/// - Java: `NetworkAlignmentEdgeLayout` (GROUP mode)
/// - Java: `AlignCycleEdgeLayout` (CYCLE mode)
/// - Java: `DefaultEdgeLayout` (ORPHAN mode, base class)
pub struct AlignmentEdgeLayout {
    /// Which layout mode to use.
    pub mode: AlignmentLayoutMode,
}

impl AlignmentEdgeLayout {
    /// Create a new alignment edge layout.
    pub fn new(mode: AlignmentLayoutMode) -> Self {
        Self { mode }
    }

    /// Build the default link group order for alignment edge layouts.
    ///
    /// Corresponds to Java `AlignCycleEdgeLayout.preProcessEdges()` and
    /// `NetworkAlignmentEdgeLayout.preProcessEdges()` which both set the
    /// group order to the 7 edge type tags.
    pub fn alignment_link_group_order() -> Vec<String> {
        EdgeType::all()
            .iter()
            .map(|t| t.short_code().to_string())
            .collect()
    }

    /// Compute cycle-mode link annotations.
    ///
    /// Ported from `AlignCycleEdgeLayout.calcGroupLinkAnnotsCycle()`.
    ///
    /// Walks the placed links in column order. For each link, determines the
    /// "zone node" (top node for regular links, bottom node for shadow links).
    /// When the zone node changes and matches a cycle bound's start, a new
    /// annotation region begins. When it matches the bound's end, the
    /// annotation is finalized (with alternating Orange/Green colors).
    ///
    /// Correctly-aligned singletons (where `bound_start == bound_end` and
    /// `is_correct`) are skipped.
    pub fn calc_cycle_link_annots(
        link_layout: &NetworkLayout,
        cycle_bounds: &[CycleBound],
        node_to_row: &HashMap<NodeId, usize>,
        shadow: bool,
        use_node_groups: bool,
    ) -> AnnotationSet {
        // If node groups are active, defer to them — no link annotations.
        if use_node_groups {
            return AnnotationSet::new();
        }
        // If not showing shadows, cycle link annotations are not produced
        // (Java: "we need every node to appear on the diagonal").
        if !shadow {
            return AnnotationSet::new();
        }

        let mut annots = AnnotationSet::new();

        // Helper: for a link, the "zone node" is the node that determines
        // which cycle region the link belongs to. For regular links it's
        // the top (min-row) node; for shadow links it's the bottom (max-row) node.
        fn zone_node(ll: &crate::layout::result::LinkLayout) -> &NodeId {
            let src_row = ll.source_row;
            let tgt_row = ll.target_row;
            if ll.is_shadow {
                // Bottom node
                if src_row > tgt_row { &ll.source } else { &ll.target }
            } else {
                // Top node
                if src_row < tgt_row { &ll.source } else { &ll.target }
            }
        }

        let mut curr_zoner: Option<NodeId> = None;
        let mut curr_bound = CycleBound {
            bound_start: NodeId::new(""),
            bound_end: NodeId::new(""),
            is_correct: false,
            is_cycle: false,
        };
        let mut seen: HashSet<NodeId> = HashSet::new();
        let mut cycle_idx = 0usize;
        let mut start_pos = 0usize;
        let mut end_pos = 0usize;
        let mut count = 0usize;

        for ll in link_layout.links.iter() {
            if ll.is_shadow && !shadow {
                continue;
            }
            let zoner = zone_node(ll);

            if curr_zoner.as_ref() != Some(zoner) {
                // Zone changed — finalize the previous annotation if appropriate
                if let Some(ref prev) = curr_zoner {
                    if *prev == curr_bound.bound_end {
                        // Don't annotate correct singletons (start == end && correct)
                        if curr_bound.bound_start != curr_bound.bound_end || !curr_bound.is_correct
                        {
                            let color = if cycle_idx % 2 == 0 {
                                "Orange"
                            } else {
                                "Green"
                            };
                            let kind = if curr_bound.is_cycle {
                                "cycle "
                            } else {
                                "path "
                            };
                            annots.add(Annotation::new(
                                format!("{}{}", kind, cycle_idx),
                                start_pos,
                                end_pos,
                                0,
                                color,
                            ));
                            cycle_idx += 1;
                        }
                    }
                }

                curr_zoner = Some(zoner.clone());

                // Check if this zone node starts a new cycle bound
                for bound in cycle_bounds {
                    if !seen.contains(&bound.bound_start) && bound.bound_start == *zoner {
                        start_pos = count;
                        seen.insert(bound.bound_start.clone());
                        curr_bound = bound.clone();
                    }
                }
            }

            end_pos = count;
            count += 1;
        }

        // Close out the last pending annotation
        if curr_bound.bound_start != curr_bound.bound_end || !curr_bound.is_correct {
            if count > 0 {
                let color = if cycle_idx % 2 == 0 {
                    "Orange"
                } else {
                    "Green"
                };
                let kind = if curr_bound.is_cycle {
                    "cycle "
                } else {
                    "path "
                };
                annots.add(Annotation::new(
                    format!("{}{}", kind, cycle_idx),
                    start_pos,
                    end_pos,
                    0,
                    color,
                ));
            }
        }

        annots
    }

    /// Get the annotation color for an edge type in CYCLE mode.
    ///
    /// Ported from `AlignCycleEdgeLayout.getColor()`.
    ///
    /// - `"G12"` → Purple
    /// - `"G1A"` → PowderBlue
    /// - `"G2A"` → Pink
    pub fn cycle_edge_type_color(type_tag: &str) -> Option<&'static str> {
        match type_tag.trim() {
            "G12" => Some("Purple"),
            "G1A" => Some("PowderBlue"),
            "G2A" => Some("Pink"),
            _ => None,
        }
    }
}

impl EdgeLayout for AlignmentEdgeLayout {
    fn layout_edges(
        &self,
        _build_data: &mut LayoutBuildData,
        _params: &LayoutParams,
        _monitor: &dyn ProgressMonitor,
    ) -> LayoutResult<NetworkLayout> {
        // TODO: Implement alignment edge layout
        //
        // Shared pre-processing (all modes):
        //   1. Set link_group_order to alignment_link_group_order()
        //   2. Set layout_mode to PerNode
        //
        // GROUP mode (see NetworkAlignmentEdgeLayout.java):
        //   - Run default edge ordering with PerNode link groups
        //   - Override getColor to return null for link annotations (grayscale only)
        //   - Generate link annotations with installLinkAnnotations()
        //
        // CYCLE mode (see AlignCycleEdgeLayout.java):
        //   - Run default edge ordering with PerNode link groups
        //   - Override calcGroupLinkAnnots with calc_cycle_link_annots()
        //   - Use cycle_bounds from build_data.alignment_data
        //
        // ORPHAN mode:
        //   - Run default edge layout on the filtered subgraph
        //
        todo!("Implement alignment edge layout")
    }

    fn name(&self) -> &'static str {
        match self.mode {
            AlignmentLayoutMode::Group => "Alignment Edge Layout (Group)",
            AlignmentLayoutMode::Orphan => "Alignment Edge Layout (Orphan)",
            AlignmentLayoutMode::Cycle => "Alignment Edge Layout (Cycle)",
        }
    }
}
