// ---------------------------------------------------------------------------
// Test configuration and runner functions
// ---------------------------------------------------------------------------

use crate::common::*;

// ---------------------------------------------------------------------------
// Test configuration
// ---------------------------------------------------------------------------

/// Configuration for a parity test case.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ParityConfig {
    pub input_file: &'static str,
    pub golden_dirname: &'static str,
    pub shadows: bool,
    pub link_groups: Option<&'static [&'static str]>,
    pub group_mode: Option<&'static str>,
    pub is_roundtrip: bool,
    // Layout algorithm (Phase A)
    pub layout: &'static str,               // "default", "similarity_resort", etc.
    pub point_up: Option<bool>,             // HierDAG
    pub control_mode: Option<&'static str>, // ControlTop
    pub target_mode: Option<&'static str>,  // ControlTop
    pub control_nodes: Option<&'static [&'static str]>, // ControlTop fixed list
    pub link_means: Option<&'static str>,   // SetLayout: "belongs_to" or "contains"
    pub attribute_file: Option<&'static str>, // NodeCluster: .na file
    pub cluster_order: Option<&'static str>, // NodeCluster: "link_size", "node_size", "name"
    pub cluster_placement: Option<&'static str>, // NodeCluster: "inline", "between"
    // NodeSimilarity custom parameters
    pub similarity_pass_count: Option<usize>,
    pub similarity_tolerance: Option<f64>,
    pub similarity_chain_length: Option<usize>,
    // Fixed-order import (Phase B)
    pub noa_file: Option<&'static str>,
    pub eda_file: Option<&'static str>,
    // Analysis (Phase C)
    pub extract_nodes: Option<&'static [&'static str]>,
    pub node_pair: Option<(&'static str, &'static str)>,
    // Display options (Phase E)
    pub min_drain_zone: Option<i32>,
    pub node_lighter: Option<f32>,
    pub link_darker: Option<f32>,
    // Alignment (Phase D)
    pub align_net2: Option<&'static str>,
    pub align_file: Option<&'static str>,
}

impl ParityConfig {
    pub fn has_custom_similarity(&self) -> bool {
        self.similarity_pass_count.is_some()
            || self.similarity_tolerance.is_some()
            || self.similarity_chain_length.is_some()
    }
}

/// Create a default ParityConfig (all extended fields set to None/defaults).
#[allow(dead_code)]
pub fn default_config(input: &'static str, golden: &'static str) -> ParityConfig {
    ParityConfig {
        input_file: input,
        golden_dirname: golden,
        shadows: true,
        link_groups: None,
        group_mode: None,
        is_roundtrip: false,
        layout: "default",
        point_up: None,
        control_mode: None,
        target_mode: None,
        control_nodes: None,
        link_means: None,
        attribute_file: None,
        cluster_order: None,
        cluster_placement: None,
        similarity_pass_count: None,
        similarity_tolerance: None,
        similarity_chain_length: None,
        noa_file: None,
        eda_file: None,
        extract_nodes: None,
        node_pair: None,
        min_drain_zone: None,
        node_lighter: None,
        link_darker: None,
        align_net2: None,
        align_file: None,
    }
}

// ---------------------------------------------------------------------------
// Enum parsing helpers for deep-variant config strings
// ---------------------------------------------------------------------------

pub fn parse_control_order(mode: Option<&str>) -> ControlOrder {
    match mode {
        Some("degree_only") => ControlOrder::DegreeOnly,
        Some("partial_order") => ControlOrder::PartialOrder,
        Some("fixed_list") => ControlOrder::FixedList,
        Some("intra_degree") => ControlOrder::IntraDegree,
        Some("median_target_degree") => ControlOrder::MedianTargetDegree,
        _ => ControlOrder::DegreeOnly,
    }
}

pub fn parse_target_order(mode: Option<&str>) -> TargetOrder {
    match mode {
        Some("target_degree") => TargetOrder::TargetDegree,
        Some("breadth_order") => TargetOrder::BreadthOrder,
        Some("gray_code") => TargetOrder::GrayCode,
        Some("degree_odometer") => TargetOrder::DegreeOdometer,
        _ => TargetOrder::TargetDegree,
    }
}

pub fn parse_cluster_order(order: Option<&str>) -> ClusterOrder {
    match order {
        Some("link_size") => ClusterOrder::LinkSize,
        Some("node_size") => ClusterOrder::NodeSize,
        Some("name") => ClusterOrder::Name,
        Some("breadth_first") => ClusterOrder::BreadthFirst,
        _ => ClusterOrder::Name, // default for backward compat
    }
}

pub fn parse_cluster_placement(placement: Option<&str>) -> InterClusterPlacement {
    match placement {
        Some("inline") => InterClusterPlacement::Inline,
        Some("between") => InterClusterPlacement::Between,
        _ => InterClusterPlacement::Between, // default for backward compat
    }
}

pub fn parse_set_semantics(s: Option<&str>) -> SetSemantics {
    match s {
        Some("belongs_to") => SetSemantics::BelongsTo,
        Some("contains") => SetSemantics::Contains,
        _ => SetSemantics::BelongsTo,
    }
}

// ---------------------------------------------------------------------------
// Layout runner functions
// ---------------------------------------------------------------------------

/// Load a network from a file based on extension.
pub fn load_network(path: &std::path::Path) -> Network {
    match path.extension().and_then(|e| e.to_str()) {
        Some("sif") => io::sif::parse_file(path).unwrap(),
        Some("gw") => io::gw::parse_file(path).unwrap(),
        _ => panic!("Unknown input format: {}", path.display()),
    }
}

/// Run default layout and get the NetworkLayout.
pub fn run_default_layout(network: &Network) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    let node_layout = DefaultNodeLayout::new();
    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap()
}

/// Run layout with a pre-determined node order (for fixed-order import).
///
/// Skips the node layout phase entirely — uses the given node_order as-is —
/// then runs the default edge layout.
pub fn run_layout_with_node_order(network: &Network, node_order: Vec<NodeId>) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap()
}

/// Run layout with link group configuration.
///
/// Uses the default BFS node layout, then runs the edge layout with
/// non-zero group ordinals so edges are grouped by relation type.
pub fn run_layout_with_groups(
    network: &Network,
    link_groups: &[String],
    mode: LayoutMode,
) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: mode,
        link_groups: Some(link_groups.to_vec()),
        ..Default::default()
    };

    // Node order is always computed with the default BFS algorithm;
    // link groups only affect the edge ordering.
    let node_layout = DefaultNodeLayout::new();
    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    let mut layout = edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap();

    // Install link group annotations (column range markers for each relation type).
    // Java does this automatically in DefaultEdgeLayout.installLinkAnnotations().
    DefaultEdgeLayout::install_link_annotations(&mut layout, link_groups, None);
    layout
}

/// Run a similarity layout (resort or clustered mode).
///
/// 1. Runs the NodeSimilarityLayout to get the refined node order
/// 2. Builds LayoutBuildData with the refined order
/// 3. Runs DefaultEdgeLayout and returns NetworkLayout
pub fn run_similarity_layout(network: &Network, mode: SimilarityMode) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    let similarity_layout = NodeSimilarityLayout::new(mode);
    let node_order = similarity_layout
        .layout_nodes(network, &params, &monitor)
        .unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    edge_layout
        .layout_edges(&mut build_data, &params, &monitor)
        .unwrap()
}

/// Run a similarity layout with custom parameters.
pub fn run_similarity_layout_custom(
    network: &Network,
    mode: SimilarityMode,
    pass_count: Option<usize>,
    tolerance: Option<f64>,
    chain_length: Option<usize>,
) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    let mut similarity_layout = NodeSimilarityLayout::new(mode);
    if let Some(pc) = pass_count {
        similarity_layout = similarity_layout.with_pass_count(pc);
    }
    if let Some(tol) = tolerance {
        similarity_layout = similarity_layout.with_tolerance(tol);
    }
    if let Some(cl) = chain_length {
        similarity_layout = similarity_layout.with_chain_length(cl);
    }
    let node_order = similarity_layout
        .layout_nodes(network, &params, &monitor)
        .unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    edge_layout
        .layout_edges(&mut build_data, &params, &monitor)
        .unwrap()
}

/// Run HierDAG layout and get the NetworkLayout.
pub fn run_hierdag_layout(network: &Network, params: &LayoutParams) -> NetworkLayout {
    let monitor = NoopMonitor;

    let node_layout = HierDAGLayout::new();
    let node_order = node_layout.layout_nodes(network, params, &monitor).unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    let mut layout = edge_layout.layout_edges(&mut build_data, params, &monitor).unwrap();

    // Install node annotations for hierarchy levels
    HierDAGLayout::install_node_annotations(network, params, &mut layout);

    layout
}

/// Run WorldBank layout and get the NetworkLayout.
pub fn run_world_bank_layout(network: &Network) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    let node_layout = WorldBankLayout::new();
    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap()
}

/// Run the SetLayout with the given semantics.
pub fn run_set_layout(network: &Network, semantics: SetSemantics) -> NetworkLayout {
    use biofabric_core::model::Annotation;
    use std::collections::BTreeSet;

    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    let node_layout = SetLayout::new().with_semantics(semantics);
    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();

    // Re-extract the set/member structure to build annotations.
    // Identify set nodes and member nodes based on semantics.
    let mut elems_per_set: std::collections::HashMap<NodeId, BTreeSet<NodeId>> = std::collections::HashMap::new();
    let mut sets_per_elem: std::collections::HashMap<NodeId, BTreeSet<NodeId>> = std::collections::HashMap::new();
    for link in network.links() {
        if link.is_shadow { continue; }
        let (set_node, member_node) = match semantics {
            SetSemantics::BelongsTo => (&link.target, &link.source),
            SetSemantics::Contains => (&link.source, &link.target),
        };
        elems_per_set.entry(set_node.clone()).or_default().insert(member_node.clone());
        sets_per_elem.entry(member_node.clone()).or_default().insert(set_node.clone());
    }

    // The node_order is: sets first (ordered by cardinality desc, name asc), then members.
    let set_nodes: BTreeSet<NodeId> = elems_per_set.keys().cloned().collect();
    let member_order: Vec<NodeId> = node_order.iter()
        .filter(|n| !set_nodes.contains(n))
        .cloned()
        .collect();
    // Ordered set list (as they appear in node_order)
    let set_list: Vec<NodeId> = node_order.iter()
        .filter(|n| set_nodes.contains(n))
        .cloned()
        .collect();

    // Java's SetLayout marks all links as directed (set membership has
    // an inherent direction). Clone the network and set directed=true.
    let mut directed_network = network.clone();
    for link in directed_network.links_mut() {
        link.directed = Some(true);
    }

    let has_shadows = directed_network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        directed_network,
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    let mut layout = edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap();

    // Java's SetLayout writes directed="true" for all links in the BIF.
    for ll in layout.iter_links_mut() {
        ll.directed = Some(true);
    }

    // --- Build annotations ---
    // Collect link layout info grouped by set and member.
    // Non-shadow links are grouped by the set node they connect to.
    // Shadow links are grouped by the member node they originate from.

    // For non-shadow links: group by set → (min_col, max_col)
    let mut set_col_ranges: std::collections::HashMap<NodeId, (usize, usize)> = std::collections::HashMap::new();
    // For shadow links: group by member → (min_col, max_col)
    let mut member_shadow_col_ranges: std::collections::HashMap<NodeId, (usize, usize)> = std::collections::HashMap::new();

    for ll in layout.iter_links() {
        if !ll.is_shadow {
            // Determine which node is the set node.
            let set_node = if set_nodes.contains(&ll.source) {
                &ll.source
            } else if set_nodes.contains(&ll.target) {
                &ll.target
            } else {
                continue;
            };
            let e = set_col_ranges.entry(set_node.clone()).or_insert((ll.column, ll.column));
            e.0 = e.0.min(ll.column);
            e.1 = e.1.max(ll.column);
        } else {
            // Shadow links: group by the member node.
            // In the shadow, source/target are swapped relative to the original.
            // The shadow's source is the original target (set), shadow's target is the original source (member).
            let member_node = if sets_per_elem.contains_key(&ll.source) {
                &ll.source
            } else if sets_per_elem.contains_key(&ll.target) {
                &ll.target
            } else {
                continue;
            };
            let e = member_shadow_col_ranges.entry(member_node.clone()).or_insert((ll.column, ll.column));
            e.0 = e.0.min(ll.column);
            e.1 = e.1.max(ll.column);
        }
    }

    // Node annotations: each member gets annotated with "&"-joined set names
    let mut node_annots = biofabric_core::model::AnnotationSet::new();
    for member in &member_order {
        if let Some(member_sets) = sets_per_elem.get(member) {
            // Sort sets in set_list order
            let tag: Vec<String> = set_list.iter()
                .filter(|s| member_sets.contains(*s))
                .map(|s| s.as_str().to_string())
                .collect();
            if !tag.is_empty() {
                let row = layout.get_node(member).map(|nl| nl.row).unwrap_or(0);
                node_annots.add(Annotation::new(tag.join("&"), row, row, 0, ""));
            }
        }
    }
    layout.node_annotations = node_annots;

    // Link annotations (non-shadow): grouped by set in set_list order
    let mut link_annots = biofabric_core::model::AnnotationSet::new();
    for set_node in &set_list {
        if set_col_ranges.contains_key(set_node) {
            // Use column_no_shadows values for linkAnnotations
            // Find the min/max column_no_shadows for this set's non-shadow links
            let mut ns_start = usize::MAX;
            let mut ns_end = 0;
            for ll in layout.iter_links() {
                if !ll.is_shadow {
                    let sn = if set_nodes.contains(&ll.source) { &ll.source }
                             else if set_nodes.contains(&ll.target) { &ll.target }
                             else { continue };
                    if sn == set_node {
                        if let Some(ns_col) = ll.column_no_shadows {
                            ns_start = ns_start.min(ns_col);
                            ns_end = ns_end.max(ns_col);
                        }
                    }
                }
            }
            if ns_start != usize::MAX {
                link_annots.add(Annotation::new(set_node.as_str(), ns_start, ns_end, 0, ""));
            }
        }
    }
    layout.link_annotations_no_shadows = link_annots;

    // Shadow link annotations: set groups (non-shadow columns) + member groups (shadow columns)
    let mut shadow_annots = biofabric_core::model::AnnotationSet::new();
    // First, set groups (using shadow column ranges for non-shadow links)
    for set_node in &set_list {
        if let Some(&(start, end)) = set_col_ranges.get(set_node) {
            shadow_annots.add(Annotation::new(set_node.as_str(), start, end, 0, ""));
        }
    }
    // Then, member groups (shadow columns) with "&"-joined set tag
    for member in &member_order {
        if let Some(&(start, end)) = member_shadow_col_ranges.get(member) {
            if let Some(member_sets) = sets_per_elem.get(member) {
                let tag: Vec<String> = set_list.iter()
                    .filter(|s| member_sets.contains(*s))
                    .map(|s| s.as_str().to_string())
                    .collect();
                if !tag.is_empty() {
                    shadow_annots.add(Annotation::new(tag.join("&"), start, end, 0, ""));
                }
            }
        }
    }
    layout.link_annotations = shadow_annots;

    layout
}

/// Run the ControlTop layout with the given control and target ordering modes.
pub fn run_control_top_layout(
    network: &Network,
    ctrl_mode: ControlOrder,
    targ_mode: TargetOrder,
    control_nodes_explicit: Option<&[&str]>,
) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    // Build the ControlTopLayout
    let control_nodes: Vec<NodeId> = control_nodes_explicit
        .map(|names| names.iter().map(|n| NodeId::new(*n)).collect())
        .unwrap_or_default();

    let node_layout = ControlTopLayout::new(control_nodes)
        .with_control_order(ctrl_mode)
        .with_target_order(targ_mode);

    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = DefaultEdgeLayout::new();
    edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap()
}

/// Run the NodeCluster layout with explicit ordering and placement modes.
pub fn run_node_cluster_layout_with_params(
    network: &Network,
    attribute_file: &str,
    order: ClusterOrder,
    placement: InterClusterPlacement,
) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    // Load and parse the attribute file
    let na_path = parity_root().join("networks").join("na").join(attribute_file);
    assert!(na_path.exists(), "Attribute file not found: {}", na_path.display());
    let content = std::fs::read_to_string(&na_path)
        .unwrap_or_else(|e| panic!("Failed to read attribute file {}: {}", na_path.display(), e));

    let mut assignments: HashMap<NodeId, String> = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "ClusterAssignment" || trimmed == "NodeCluster" {
            continue;
        }
        if let Some((name, cluster)) = trimmed.split_once(" = ") {
            assignments.insert(NodeId::new(name.trim()), cluster.trim().to_string());
        }
    }

    let node_layout = NodeClusterLayout::new(assignments.clone())
        .with_order(order)
        .with_inter_cluster(placement);

    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();
    let node_order_copy = node_order.clone();

    let has_shadows = network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        network.clone(),
        node_order,
        has_shadows,
        params.layout_mode,
    );

    let edge_layout = NodeClusterEdgeLayout {
        params: biofabric_core::layout::cluster::ClusterLayoutParams {
            cluster_order: order,
            inter_cluster: placement,
        },
        assignments: assignments.clone(),
    };
    let mut layout = edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap();

    // Populate cluster assignments on the layout for BIF export
    layout.cluster_assignments = assignments.clone().into_iter().collect();

    // Build cluster node annotations: contiguous row ranges per cluster
    // Walk the node order and collect (cluster_name → (min_row, max_row))
    {
        use biofabric_core::model::Annotation;
        let mut cluster_ranges: Vec<(String, usize, usize)> = Vec::new();
        let mut current_cluster: Option<String> = None;
        for (row, node_id) in node_order_copy.iter().enumerate() {
            let cluster = assignments.get(node_id).cloned();
            if let Some(ref c) = cluster {
                if let Some(ref cc) = current_cluster {
                    if cc == c {
                        // Extend current range
                        if let Some(last) = cluster_ranges.last_mut() {
                            last.2 = row;
                        }
                    } else {
                        // New cluster
                        cluster_ranges.push((c.clone(), row, row));
                        current_cluster = Some(c.clone());
                    }
                } else {
                    cluster_ranges.push((c.clone(), row, row));
                    current_cluster = Some(c.clone());
                }
            } else {
                current_cluster = None;
            }
        }
        for (name, start, end) in cluster_ranges {
            layout.node_annotations.add(Annotation::new(&name, start, end, 0, ""));
        }

        // Build link cluster annotations (both shadow and non-shadow)
        // For each link, the cluster is determined by the anchor node (source for
        // non-shadow, target for shadow — but in Rust shadow links have swapped
        // source/target, so it's always source).
        //
        // An inter-cluster link (where source and target are in different clusters)
        // does NOT get an annotation.
        let _node_to_row: HashMap<&NodeId, usize> = node_order_copy
            .iter()
            .enumerate()
            .map(|(row, id)| (id, row))
            .collect();

        let mut shadow_annots: Vec<(String, usize, usize)> = Vec::new();
        let mut nonshadow_annots: Vec<(String, usize, usize)> = Vec::new();

        for ll in layout.iter_links() {
            let src_cluster = assignments.get(&ll.source);
            let tgt_cluster = assignments.get(&ll.target);

            // Only annotate intra-cluster links (both endpoints in the same cluster)
            let cluster = match (src_cluster, tgt_cluster) {
                (Some(sc), Some(tc)) if sc == tc => sc.clone(),
                _ => continue,
            };

            // Shadow annotation (uses ll.column = shadow column)
            if let Some(last) = shadow_annots.last_mut() {
                if last.0 == cluster {
                    last.2 = ll.column;
                } else {
                    shadow_annots.push((cluster.clone(), ll.column, ll.column));
                }
            } else {
                shadow_annots.push((cluster.clone(), ll.column, ll.column));
            }

            // Non-shadow annotation (only for non-shadow links)
            if !ll.is_shadow {
                if let Some(col_ns) = ll.column_no_shadows {
                    if let Some(last) = nonshadow_annots.last_mut() {
                        if last.0 == cluster {
                            last.2 = col_ns;
                        } else {
                            nonshadow_annots.push((cluster.clone(), col_ns, col_ns));
                        }
                    } else {
                        nonshadow_annots.push((cluster.clone(), col_ns, col_ns));
                    }
                }
            }
        }

        // link_annotations_no_shadows → written as <linkAnnotations> (non-shadow columns)
        for (name, start, end) in nonshadow_annots {
            layout.link_annotations_no_shadows.add(Annotation::new(&name, start, end, 0, ""));
        }
        // link_annotations → written as <shadowLinkAnnotations> (shadow columns)
        for (name, start, end) in shadow_annots {
            layout.link_annotations.add(Annotation::new(&name, start, end, 0, ""));
        }
    }

    layout
}

// ---------------------------------------------------------------------------
// NOA/EDA file parsing and path helpers
// ---------------------------------------------------------------------------

/// Parse a NOA-format file ("Node Row" header, "name = row" lines).
///
/// These files use the BioFabric NOA export format:
/// ```text
/// Node Row
/// C = 0
/// B = 1
/// A = 2
/// ```
///
/// Returns node IDs sorted by ascending row number.
pub fn parse_noa_format_file(path: &std::path::Path) -> Vec<NodeId> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read NOA file {}: {}", path.display(), e));
    let mut entries: Vec<(String, usize)> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "Node Row" {
            continue;
        }
        if let Some((name, row_str)) = trimmed.rsplit_once(" = ") {
            let row: usize = row_str.parse()
                .unwrap_or_else(|e| panic!("Invalid row number in NOA file: '{}': {}", trimmed, e));
            entries.push((name.to_string(), row));
        }
    }
    entries.sort_by_key(|(_, row)| *row);
    entries.into_iter().map(|(name, _)| NodeId::new(name)).collect()
}

/// Path to a NOA file for fixed-order import.
pub fn noa_file_path(filename: &str) -> PathBuf {
    parity_root().join("networks").join("noa").join(filename)
}

/// Path to an EDA file for fixed link order import.
pub fn eda_file_path(filename: &str) -> PathBuf {
    parity_root().join("networks").join("eda").join(filename)
}

/// Parse an EDA format file and return the link ordering.
///
/// The EDA file format specifies a column for each link:
///   - Non-shadow: `SOURCE (RELATION) TARGET = COLUMN`
///   - Shadow: `SOURCE shdw(RELATION) TARGET = COLUMN`
///
/// Returns a Vec of (source, target, relation, is_shadow, column) sorted by column.
pub fn parse_eda_format_file(path: &std::path::Path) -> Vec<(String, String, String, bool, usize)> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read EDA file {}: {}", path.display(), e));
    let mut entries: Vec<(String, String, String, bool, usize)> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "Link Column" {
            continue;
        }
        if let Some((link_spec, col_str)) = trimmed.rsplit_once(" = ") {
            let col: usize = col_str.parse()
                .unwrap_or_else(|e| panic!("Invalid column in EDA: '{}': {}", trimmed, e));

            // Parse link spec: "A (pp) B" or "A shdw(pp) B"
            let is_shadow = link_spec.contains(" shdw(");
            if is_shadow {
                // Format: "SOURCE shdw(RELATION) TARGET"
                if let Some((source, rest)) = link_spec.split_once(" shdw(") {
                    if let Some((relation, target)) = rest.split_once(") ") {
                        entries.push((source.to_string(), target.to_string(), relation.to_string(), true, col));
                    }
                }
            } else {
                // Format: "SOURCE (RELATION) TARGET"
                if let Some((source, rest)) = link_spec.split_once(" (") {
                    if let Some((relation, target)) = rest.split_once(") ") {
                        entries.push((source.to_string(), target.to_string(), relation.to_string(), false, col));
                    }
                }
            }
        }
    }
    entries.sort_by_key(|(_, _, _, _, col)| *col);
    entries
}

/// Run layout with a fixed link order from an EDA file.
///
/// This replicates Java's LINK_ATTRIB_LAYOUT BuildMode:
/// 1. Run the default node layout (BFS order) to get node rows
/// 2. Apply the EDA-specified link ordering instead of the default edge layout
pub fn run_layout_with_link_order(network: &Network, eda_path: &std::path::Path) -> NetworkLayout {
    let monitor = NoopMonitor;
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        ..Default::default()
    };

    // Step 1: Default node layout (BFS)
    let node_layout = DefaultNodeLayout::new();
    let node_order = node_layout.layout_nodes(network, &params, &monitor).unwrap();

    // Build node-to-row map
    let node_to_row: HashMap<NodeId, usize> = node_order.iter().enumerate()
        .map(|(row, id)| (id.clone(), row))
        .collect();

    // Step 2: Parse EDA file
    let eda_entries = parse_eda_format_file(eda_path);

    // Step 3: Build layout from the fixed link order
    let mut layout = NetworkLayout::with_capacity(node_order.len(), eda_entries.len());
    layout.row_count = node_order.len();

    // Initialize node layouts
    for (i, node_id) in node_order.iter().enumerate() {
        let mut nl = biofabric_core::layout::result::NodeLayout::new(i, node_id.as_str());
        nl.color_index = i;
        layout.nodes.insert(node_id.clone(), nl);
    }

    // Build link layouts from EDA entries
    // The EDA specifies the SHADOW column order. Non-shadow links also have
    // a non-shadow column counter.
    let mut ns_col = 0usize; // non-shadow column counter
    for (source, target, relation, is_shadow, shadow_col) in &eda_entries {
        let src_id = NodeId::new(source.as_str());
        let tgt_id = NodeId::new(target.as_str());

        // In Rust, shadow links have FLIPPED source/target
        let (ll_src, ll_tgt) = if *is_shadow {
            (tgt_id.clone(), src_id.clone())
        } else {
            (src_id.clone(), tgt_id.clone())
        };

        let src_row = node_to_row[&ll_src];
        let tgt_row = node_to_row[&ll_tgt];

        let mut ll = biofabric_core::layout::result::LinkLayout::new(
            *shadow_col,
            ll_src.clone(),
            ll_tgt.clone(),
            src_row,
            tgt_row,
            relation.clone(),
            *is_shadow,
        );
        ll.color_index = *shadow_col;

        if !is_shadow {
            ll.column_no_shadows = Some(ns_col);
            // Update node non-shadow spans
            if let Some(nl) = layout.nodes.get_mut(&ll_src) {
                nl.update_span_no_shadows(ns_col);
            }
            if let Some(nl) = layout.nodes.get_mut(&ll_tgt) {
                nl.update_span_no_shadows(ns_col);
            }
            ns_col += 1;
        }

        // Update node shadow spans
        if let Some(nl) = layout.nodes.get_mut(&ll_src) {
            nl.update_span(*shadow_col);
        }
        if let Some(nl) = layout.nodes.get_mut(&ll_tgt) {
            nl.update_span(*shadow_col);
        }

        layout.links.push(ll);
    }

    layout.column_count = eda_entries.len();
    layout.column_count_no_shadows = ns_col;

    layout
}

// ---------------------------------------------------------------------------
// Alignment helpers
// ---------------------------------------------------------------------------

/// Path to an alignment file.
pub fn align_file_path(filename: &str) -> PathBuf {
    parity_root().join("networks").join("align").join(filename)
}

/// Run alignment layout: merge two networks using an alignment, then lay out.
pub fn alignment_perfect_file(align_file: &str) -> Option<&'static str> {
    match align_file {
        "test_perfect.align" => Some("test_perfect.align"),
        "test_partial.align" => None,
        "yeast_sc_s3_pure.align" => Some("yeast_sc_perfect.align"),
        "yeast_sc_perfect.align" => Some("yeast_sc_perfect.align"),
        "casestudy_iv.align" => Some("casestudy_iv.align"),
        _ => None,
    }
}

pub fn alignment_ng_mode(golden_dirname: &str) -> Option<PerfectNGMode> {
    if golden_dirname.contains("ng_nc") {
        Some(PerfectNGMode::NodeCorrectness)
    } else if golden_dirname.contains("ng_jacc") {
        Some(PerfectNGMode::JaccardSimilarity)
    } else {
        None
    }
}

pub fn alignment_use_node_groups(golden_dirname: &str) -> bool {
    golden_dirname.contains("cycle_noshadow")
}

pub struct AlignmentLayoutResult {
    pub network: Network,
    pub layout: NetworkLayout,
}

pub fn run_alignment_layout(
    cfg: &ParityConfig,
    g1: &Network,
    g2: &Network,
    align_file: &str,
    mode: AlignmentLayoutMode,
) -> AlignmentLayoutResult {
    let monitor = NoopMonitor;

    // Parse alignment file
    let align_path = align_file_path(align_file);
    assert!(
        align_path.exists(),
        "Alignment file not found: {}",
        align_path.display()
    );
    let alignment = biofabric_core::io::align::parse_file(&align_path).unwrap();

    let perfect_alignment = alignment_perfect_file(align_file).map(|perfect_file| {
        let perfect_path = align_file_path(perfect_file);
        assert!(
            perfect_path.exists(),
            "Perfect alignment file not found: {}",
            perfect_path.display()
        );
        biofabric_core::io::align::parse_file(&perfect_path).unwrap()
    });

    // Merge networks (include perfect alignment if available)
    let merged = MergedNetwork::from_alignment(
        g1,
        g2,
        &alignment,
        perfect_alignment.as_ref(),
        &monitor,
    )
    .unwrap();

    // Determine PerfectNG mode (if any)
    let ng_mode = alignment_ng_mode(cfg.golden_dirname);
    let jaccard_correct = if matches!(ng_mode, Some(PerfectNGMode::JaccardSimilarity)) {
        perfect_alignment.as_ref().map(|perfect| {
            JaccardSimilarity::correctness_map_with_alignments(
                g1,
                g2,
                &alignment,
                perfect,
                DEFAULT_JACCARD_THRESHOLD,
            )
        })
    } else {
        None
    };

    let groups = if let Some(mode) = ng_mode {
        NodeGroupMap::from_merged_with_mode(
            &merged,
            mode,
            jaccard_correct.as_ref(),
            DEFAULT_JACCARD_THRESHOLD,
            &monitor,
        )
    } else {
        NodeGroupMap::from_merged(&merged, &monitor)
    };

    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        link_groups: Some(
            biofabric_core::alignment::layout::AlignmentEdgeLayout::alignment_link_group_order(),
        ),
        ..Default::default()
    };

    // Node layout
    let mut node_layout = AlignmentNodeLayout::new(merged.clone(), mode).with_groups(groups.clone());
    if mode == AlignmentLayoutMode::Cycle {
        let cycles = AlignmentCycles::detect(&alignment, perfect_alignment.as_ref(), &monitor);
        node_layout = node_layout.with_cycles(cycles);
    }
    let node_order = node_layout
        .layout_nodes(&merged.network, &params, &monitor)
        .unwrap();

    // For orphan mode, use the filtered (orphan-only) subnetwork for edge layout.
    // In Java, OrphanEdgeLayout.process() mutates mergedLinks in-place so the
    // entire subsequent layout pipeline operates on the filtered subgraph.
    let layout_network = if mode == AlignmentLayoutMode::Orphan {
        OrphanFilter::filter(&merged).network
    } else {
        merged.network.clone()
    };

    // Edge layout
    let has_shadows = layout_network.has_shadows();
    let mut build_data = LayoutBuildData::new(
        layout_network.clone(),
        node_order,
        has_shadows,
        LayoutMode::PerNode,
    );

    // Java uses DefaultEdgeLayout (PerNode BFS, no link groups) for ORPHAN mode,
    // and AlignmentEdgeLayout (PerNetwork with link groups) for GROUP/CYCLE.
    let mut layout = if mode == AlignmentLayoutMode::Orphan {
        let orphan_params = LayoutParams {
            include_shadows: true,
            layout_mode: LayoutMode::PerNode,
            ..Default::default()  // no link_groups
        };
        let edge_layout = DefaultEdgeLayout::new();
        edge_layout.layout_edges(&mut build_data, &orphan_params, &monitor).unwrap()
    } else {
        let edge_layout = AlignmentEdgeLayout::new(mode);
        edge_layout.layout_edges(&mut build_data, &params, &monitor).unwrap()
    };

    // Install node annotations for group/cycle layouts
    if mode == AlignmentLayoutMode::Group || alignment_use_node_groups(cfg.golden_dirname) {
        layout.node_annotations = AlignmentNodeLayout::build_group_node_annotations(&groups);
    }

    AlignmentLayoutResult {
        network: layout_network,
        layout,
    }
}

// ---------------------------------------------------------------------------
// Subnetwork extraction
// ---------------------------------------------------------------------------

/// Extract a subnetwork from a loaded network using the specified node names.
///
/// Filters the network to include only the named nodes and edges where
/// both endpoints are in the set. Shadow links are included transitively
/// (if the original non-shadow link survives extraction, so does its shadow).
pub fn extract_subnetwork(network: &Network, node_names: &[&str]) -> Network {
    let node_ids: HashSet<NodeId> = node_names.iter().map(|n| NodeId::new(*n)).collect();
    network.extract_subnetwork(&node_ids)
}

/// Extract a submodel from a fully laid-out session, compressing rows and columns.
///
/// This replicates the Java `BioFabricNetwork.fillSubModel()` behavior:
/// 1. Keep only links where both endpoints are in the extract set
/// 2. Keep only nodes that are in the extract set
/// 3. Compress rows (renumber sequentially)
/// 4. Compress non-shadow and shadow columns independently
/// 5. Set each node's maxCol = maxLinkCol and maxColSha = maxShadLinkCol
///    (matching a Java HashMap key-type mismatch bug where nodes always get
///     the global max column as their rightmost column)
///
/// This is different from `extract_subnetwork` which extracts from the raw
/// network and re-runs the layout from scratch. This function preserves the
/// original layout ordering and just compresses it.
pub fn extract_submodel(
    network: &Network,
    layout: &NetworkLayout,
    node_names: &[&str],
) -> (Network, NetworkLayout) {
    use std::collections::BTreeSet;

    let extract_set: HashSet<NodeId> = node_names.iter().map(|n| NodeId::new(*n)).collect();

    // Collect sub-nodes and sub-links from the full layout
    // Sub-links: links where both source and target are in extract_set
    // For shadow links in Rust, source/target are FLIPPED from the original.
    // But for the extraction filter, we need ORIGINAL source/target.
    // In the Rust model, shadow links have flipped source/target, so we need
    // to check both orientations.
    let sub_links: Vec<&biofabric_core::layout::result::LinkLayout> = layout
        .links
        .iter()
        .filter(|ll| {
            // For non-shadow links: source and target are as-is
            // For shadow links: source and target are SWAPPED in Rust model
            // Java keeps original source/target for shadows
            // So for extraction filter, we check:
            //   non-shadow: source & target in set
            //   shadow: target (=original source) & source (=original target) in set
            // Either way, both endpoint IDs must be in the extract set
            extract_set.contains(&ll.source) && extract_set.contains(&ll.target)
        })
        .collect();

    // Gather full-layout column values we need to keep
    let mut need_rows = BTreeSet::new();
    let mut need_columns = BTreeSet::new(); // non-shadow
    let mut need_columns_shad = BTreeSet::new(); // shadow (all columns)

    // From nodes: add each node's min column from the full layout
    for (id, nl) in layout.iter_nodes() {
        if !extract_set.contains(id) {
            continue;
        }
        need_rows.insert(nl.row);
        if nl.has_edges_no_shadows() {
            need_columns.insert(nl.min_col_no_shadows);
        }
        if nl.has_edges() {
            need_columns_shad.insert(nl.min_col);
        }
    }

    // From links: add rows and columns
    for ll in &sub_links {
        need_rows.insert(ll.source_row);
        need_rows.insert(ll.target_row);
        if !ll.is_shadow {
            if let Some(col) = ll.column_no_shadows {
                need_columns.insert(col);
            }
        }
        need_columns_shad.insert(ll.column);
    }

    // Create compression maps (full-scale → mini-scale)
    let row_map: std::collections::BTreeMap<usize, usize> = need_rows
        .iter()
        .enumerate()
        .map(|(i, &r)| (r, i))
        .collect();
    let col_map: std::collections::BTreeMap<usize, usize> = need_columns
        .iter()
        .enumerate()
        .map(|(i, &c)| (c, i))
        .collect();
    let shad_col_map: std::collections::BTreeMap<usize, usize> = need_columns_shad
        .iter()
        .enumerate()
        .map(|(i, &c)| (c, i))
        .collect();

    // Build compressed links and track max columns
    let mut max_link_col: Option<usize> = None;
    let mut max_shad_link_col: Option<usize> = None;
    let mut compressed_links: Vec<biofabric_core::layout::result::LinkLayout> = Vec::new();

    for ll in &sub_links {
        let new_src_row = row_map[&ll.source_row];
        let new_tgt_row = row_map[&ll.target_row];
        let new_shad_col = shad_col_map[&ll.column];

        let new_col_no_shadows = if ll.is_shadow {
            None
        } else {
            let c = ll.column_no_shadows.map(|c| col_map[&c]);
            if let Some(col) = c {
                max_link_col = Some(max_link_col.map_or(col, |m: usize| m.max(col)));
            }
            c
        };

        max_shad_link_col = Some(
            max_shad_link_col.map_or(new_shad_col, |m: usize| m.max(new_shad_col)),
        );

        let mut new_ll = biofabric_core::layout::result::LinkLayout::new(
            new_shad_col,
            ll.source.clone(),
            ll.target.clone(),
            new_src_row,
            new_tgt_row,
            ll.relation.clone(),
            ll.is_shadow,
        );
        new_ll.column_no_shadows = new_col_no_shadows;
        new_ll.color_index = ll.color_index;
        compressed_links.push(new_ll);
    }

    // Java increments max columns by 1 (to represent "count" rather than "max index")
    let max_link_col = max_link_col.map_or(1, |m| m + 1);
    let max_shad_link_col = max_shad_link_col.map_or(1, |m| m + 1);

    // Build compressed nodes
    // Due to the Java NID/NetNode type mismatch bug in fillSubModel,
    // maxCol is always maxLinkCol and maxColSha is always maxShadLinkCol
    let mut compressed_nodes: indexmap::IndexMap<NodeId, biofabric_core::layout::result::NodeLayout> =
        indexmap::IndexMap::new();

    // Build a nid map from the FULL network's node order
    // (matching the Java UniqueLabeller sequential IDs)
    let nid_map: HashMap<NodeId, usize> = network
        .node_ids()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect();

    // Process nodes in row order
    let mut nodes_sorted: Vec<(&NodeId, &biofabric_core::layout::result::NodeLayout)> = layout
        .iter_nodes()
        .filter(|(id, _)| extract_set.contains(id))
        .collect();
    nodes_sorted.sort_by_key(|(_, nl)| nl.row);

    for (id, nl) in &nodes_sorted {
        let new_row = row_map[&nl.row];
        let mut new_nl = biofabric_core::layout::result::NodeLayout::new(new_row, &nl.name);

        // minCol from compression
        if nl.has_edges_no_shadows() {
            let min_col = col_map[&nl.min_col_no_shadows];
            new_nl.min_col_no_shadows = min_col;
            // maxCol = maxLinkCol (Java bug: NID/NetNode mismatch causes fallback)
            new_nl.max_col_no_shadows = max_link_col;
        }
        if nl.has_edges() {
            let min_col_sha = shad_col_map[&nl.min_col];
            new_nl.min_col = min_col_sha;
            // maxColSha = maxShadLinkCol (Java bug: same reason)
            new_nl.max_col = max_shad_link_col;
        }

        // Set nid from the FULL network's node enumeration order
        // This matches Java's UniqueLabeller sequential IDs
        new_nl.nid = nid_map.get(*id).copied();
        // Preserve original color: Java's extraction uses infoFull.colorKey
        // which is based on the ORIGINAL row, not the compressed row
        new_nl.color_index = nl.row; // Color is derived from original row index

        // Pre-compute drain zones using Java's fillSubModel scan behavior
        // Java scans BACKWARD from maxCol to minCol for plain drain zones,
        // and FORWARD from minColSha to maxColSha for shadow drain zones.
        // When maxCol > actual rightmost link column (due to the NID/NetNode bug),
        // the scan hits a gap immediately and produces no drain zone.

        // Plain (non-shadow) drain zones: scan from maxCol down to minCol
        {
            let mut dz: Option<(usize, usize)> = None;
            let range_max = new_nl.max_col_no_shadows;
            let range_min = new_nl.min_col_no_shadows;
            if range_max >= range_min && range_max != usize::MAX {
                let mut col = range_max as isize;
                while col >= range_min as isize {
                    let c = col as usize;
                    // Find a non-shadow link at this column
                    let link = compressed_links.iter().find(|ll| {
                        !ll.is_shadow && ll.column_no_shadows == Some(c)
                    });
                    match link {
                        None => break, // gap → stop
                        Some(ll) => {
                            // Check if this node is involved
                            if ll.source != **id && ll.target != **id {
                                break; // link doesn't involve this node
                            }
                            // For non-shadow: check if bottom row = node row AND top row != node row → break
                            let bottom = ll.source_row.max(ll.target_row);
                            let top = ll.source_row.min(ll.target_row);
                            if bottom == new_row && top != new_row {
                                break;
                            }
                            // If top row = node row, add to drain zone
                            if top == new_row {
                                dz = Some(match dz {
                                    None => (c, c),
                                    Some((min, max)) => (min.min(c), max.max(c)),
                                });
                            }
                        }
                    }
                    col -= 1;
                }
            }
            new_nl.plain_drain_zones = Some(dz.map_or(Vec::new(), |z| vec![z]));
        }

        // Shadow drain zones: scan FORWARD from minColSha to maxColSha
        {
            let mut dz: Option<(usize, usize)> = None;
            let range_min = new_nl.min_col;
            let range_max = new_nl.max_col;
            if range_max >= range_min && range_min != usize::MAX {
                for c in range_min..=range_max {
                    let link = compressed_links.iter().find(|ll| ll.column == c);
                    match link {
                        None => continue, // gap → skip (not break!)
                        Some(ll) => {
                            // If we have an active drain zone and this link doesn't involve the node, break
                            if dz.is_some() && ll.source != **id && ll.target != **id {
                                break;
                            }
                            let top = ll.source_row.min(ll.target_row);
                            let bottom = ll.source_row.max(ll.target_row);
                            // Shadow drain zone condition:
                            // (top == node_row && !isShadow) || (bottom == node_row && isShadow)
                            let qualifies = (top == new_row && !ll.is_shadow)
                                || (bottom == new_row && ll.is_shadow);
                            if qualifies {
                                dz = Some(match dz {
                                    None => (c, c),
                                    Some((min, max)) => (min.min(c), max.max(c)),
                                });
                            }
                        }
                    }
                }
            }
            new_nl.shadow_drain_zones = Some(dz.map_or(Vec::new(), |z| vec![z]));
        }

        compressed_nodes.insert((*id).clone(), new_nl);
    }

    // Build the compressed NetworkLayout
    let mut new_layout = NetworkLayout::new();
    new_layout.nodes = compressed_nodes;
    new_layout.links = compressed_links;
    new_layout.row_count = row_map.len();
    new_layout.column_count = max_shad_link_col;
    new_layout.column_count_no_shadows = max_link_col;

    // Build the extracted network
    let sub_network = extract_subnetwork(network, node_names);

    (sub_network, new_layout)
}

// ---------------------------------------------------------------------------
// Format functions (NOA, EDA output)
// ---------------------------------------------------------------------------

/// Format a NetworkLayout as NOA string.
pub fn format_noa(layout: &NetworkLayout) -> String {
    let mut out = String::new();
    out.push_str("Node Row\n");

    // Collect nodes sorted by row
    let mut nodes: Vec<(&biofabric_core::model::NodeId, &biofabric_core::layout::result::NodeLayout)> =
        layout.iter_nodes().collect();
    nodes.sort_by_key(|(_, info)| info.row);

    for (id, info) in nodes {
        out.push_str(&format!("{} = {}\n", id, info.row));
    }

    out
}

/// Format a NetworkLayout as EDA string (shadows ON).
///
/// For shadow links, the Rust model stores flipped source/target (from to_shadow()),
/// but the Java golden files show the ORIGINAL (unflipped) source/target. So for
/// shadow links we display target as source and source as target.
pub fn format_eda(layout: &NetworkLayout) -> String {
    let mut out = String::new();
    out.push_str("Link Column\n");

    for ll in layout.iter_links() {
        if ll.is_shadow {
            // Shadow links are flipped in Rust model, display original order
            out.push_str(&format!(
                "{} shdw({}) {} = {}\n",
                ll.target, ll.relation, ll.source, ll.column
            ));
        } else {
            out.push_str(&format!(
                "{} ({}) {} = {}\n",
                ll.source, ll.relation, ll.target, ll.column
            ));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Misc helpers
// ---------------------------------------------------------------------------

pub fn parse_bool_from_text(text: &str) -> Option<bool> {
    for token in text
        .split(|c: char| !c.is_ascii_alphabetic())
        .filter(|t| !t.is_empty())
    {
        match token.to_lowercase().as_str() {
            "true" => return Some(true),
            "false" => return Some(false),
            _ => {}
        }
    }
    None
}

pub fn parse_float_from_text(text: &str) -> Option<f64> {
    for token in text.split_whitespace() {
        if let Ok(val) = token.parse::<f64>() {
            return Some(val);
        }
    }
    None
}

pub fn has_undirected_cycle(network: &Network) -> bool {
    let mut adj: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

    for link in network.links() {
        if link.is_shadow {
            continue;
        }
        if link.source == link.target {
            return true;
        }
        adj.entry(link.source.clone())
            .or_default()
            .insert(link.target.clone());
        adj.entry(link.target.clone())
            .or_default()
            .insert(link.source.clone());
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    for node in network.node_ids() {
        if visited.contains(node) {
            continue;
        }
        if dfs_undirected_cycle(node, None, &adj, &mut visited) {
            return true;
        }
    }
    false
}

fn dfs_undirected_cycle(
    node: &NodeId,
    parent: Option<&NodeId>,
    adj: &HashMap<NodeId, HashSet<NodeId>>,
    visited: &mut HashSet<NodeId>,
) -> bool {
    visited.insert(node.clone());
    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if Some(neighbor) == parent {
                continue;
            }
            if visited.contains(neighbor) {
                return true;
            }
            if dfs_undirected_cycle(neighbor, Some(node), adj, visited) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Test runner functions (one per output format)
// ---------------------------------------------------------------------------

pub fn run_analysis_test(cfg: &ParityConfig) {
    let input = network_path(cfg.input_file);
    assert!(input.exists(), "Input file not found: {}", input.display());

    golden_available(cfg.golden_dirname, "output.txt");
    let golden_txt = read_golden(cfg.golden_dirname, "output.txt");
    let network = load_network(&input);

    match cfg.layout {
        "analysis_cycle" => {
            let expected = parse_bool_from_text(&golden_txt).unwrap_or_else(|| {
                panic!(
                    "Failed to parse boolean from golden output for {}",
                    cfg.golden_dirname
                )
            });
            let actual = has_undirected_cycle(&network);
            assert_eq!(
                actual, expected,
                "Cycle detection mismatch for {}",
                cfg.golden_dirname
            );
        }
        "analysis_jaccard" => {
            let (a, b) = cfg
                .node_pair
                .expect("analysis_jaccard requires a node pair");
            let comparison = network
                .compare_nodes(&NodeId::new(a), &NodeId::new(b))
                .unwrap_or_else(|| {
                    panic!("Nodes not found for Jaccard test {} ({}, {})", cfg.golden_dirname, a, b)
                });
            let expected = parse_float_from_text(&golden_txt).unwrap_or_else(|| {
                panic!(
                    "Failed to parse float from golden output for {}",
                    cfg.golden_dirname
                )
            });
            let actual = comparison.jaccard_similarity;
            let tolerance = 1e-10;
            assert!(
                (actual - expected).abs() < tolerance,
                "Jaccard mismatch for {}: expected {}, got {}",
                cfg.golden_dirname,
                expected,
                actual
            );
        }
        _ => panic!("Unknown analysis layout: {}", cfg.layout),
    }
}

/// Run a NOA (node order) parity test.
///
/// Supports:
/// - `extract_nodes`: extract subnetwork before layout
/// - `noa_file`: use imported node order instead of BFS
/// - `link_groups`: node order is unchanged (groups only affect EDA)
#[allow(dead_code)]
pub fn run_noa_test(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.noa");

    let input = if cfg.is_roundtrip {
        golden_dir(cfg.golden_dirname).join("output.bif")
    } else {
        network_path(cfg.input_file)
    };
    assert!(input.exists(), "Input file not found: {}", input.display());

    let golden_noa = read_golden(cfg.golden_dirname, "output.noa");

    let mut network = load_network(&input);

    // P12: Subnetwork extraction — filter to specified nodes before layout
    if let Some(extract_nodes) = cfg.extract_nodes {
        network = extract_subnetwork(&network, extract_nodes);
    }

    // Alignment layout dispatch
    let layout = if cfg.layout == "alignment" {
        let net2_file = cfg.align_net2.expect("alignment tests require align_net2");
        let align_file = cfg.align_file.expect("alignment tests require align_file");
        let g2_path = network_path(net2_file);
        assert!(g2_path.exists(), "G2 network not found: {}", g2_path.display());
        let g2 = load_network(&g2_path);

        // Determine alignment mode from golden dirname
        let mode = if cfg.golden_dirname.contains("orphan") {
            AlignmentLayoutMode::Orphan
        } else if cfg.golden_dirname.contains("cycle") {
            AlignmentLayoutMode::Cycle
        } else {
            AlignmentLayoutMode::Group
        };

        run_alignment_layout(cfg, &network, &g2, align_file, mode).layout
    } else if cfg.layout == "hierdag" {
        // P6: HierDAG layout
        let mut params = LayoutParams {
            include_shadows: true,
            layout_mode: LayoutMode::PerNode,
            ..Default::default()
        };
        params.point_up = cfg.point_up;
        run_hierdag_layout(&network, &params)
    } else if cfg.layout == "world_bank" {
        // P10: WorldBank layout
        run_world_bank_layout(&network)
    } else if cfg.layout == "set" {
        let semantics = parse_set_semantics(cfg.link_means);
        run_set_layout(&network, semantics)
    } else if cfg.layout == "control_top" {
        let ctrl_mode = parse_control_order(cfg.control_mode);
        let targ_mode = parse_target_order(cfg.target_mode);
        run_control_top_layout(&network, ctrl_mode, targ_mode, cfg.control_nodes)
    } else if cfg.layout == "node_cluster" {
        let order = parse_cluster_order(cfg.cluster_order);
        let placement = parse_cluster_placement(cfg.cluster_placement);
        run_node_cluster_layout_with_params(&network, cfg.attribute_file.unwrap(), order, placement)
    } else if let Some(noa_file) = cfg.noa_file {
        let noa_path = noa_file_path(noa_file);
        assert!(noa_path.exists(), "NOA file not found: {}", noa_path.display());
        let node_order = parse_noa_format_file(&noa_path);
        run_layout_with_node_order(&network, node_order)
    } else {
        match cfg.layout {
            "similarity_resort" if cfg.has_custom_similarity() => {
                run_similarity_layout_custom(&network, SimilarityMode::Resort,
                    cfg.similarity_pass_count, cfg.similarity_tolerance, cfg.similarity_chain_length)
            }
            "similarity_clustered" if cfg.has_custom_similarity() => {
                run_similarity_layout_custom(&network, SimilarityMode::Clustered,
                    cfg.similarity_pass_count, cfg.similarity_tolerance, cfg.similarity_chain_length)
            }
            "similarity_resort" => run_similarity_layout(&network, SimilarityMode::Resort),
            "similarity_clustered" => run_similarity_layout(&network, SimilarityMode::Clustered),
            _ => {
                // P3 (link grouping) does NOT change node order — default BFS
                run_default_layout(&network)
            }
        }
    };

    let actual_noa = format_noa(&layout);
    assert_parity("NOA", &golden_noa, &actual_noa);
}

/// Run an EDA (edge/link order) parity test.
///
/// Supports:
/// - `extract_nodes`: extract subnetwork before layout
/// - `noa_file`: use imported node order, then default edge layout
/// - `link_groups` + `group_mode`: group edges by relation type
#[allow(dead_code)]
pub fn run_eda_test(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.eda");

    let input = if cfg.is_roundtrip {
        golden_dir(cfg.golden_dirname).join("output.bif")
    } else {
        network_path(cfg.input_file)
    };
    assert!(input.exists(), "Input file not found: {}", input.display());

    let golden_eda = read_golden(cfg.golden_dirname, "output.eda");

    let mut network = load_network(&input);

    // P12: Subnetwork extraction — filter to specified nodes before layout
    if let Some(extract_nodes) = cfg.extract_nodes {
        network = extract_subnetwork(&network, extract_nodes);
    }

    // Choose the appropriate layout strategy
    let layout = if cfg.layout == "alignment" {
        let net2_file = cfg.align_net2.expect("alignment tests require align_net2");
        let align_file = cfg.align_file.expect("alignment tests require align_file");
        let g2_path = network_path(net2_file);
        assert!(g2_path.exists(), "G2 network not found: {}", g2_path.display());
        let g2 = load_network(&g2_path);

        let mode = if cfg.golden_dirname.contains("orphan") {
            AlignmentLayoutMode::Orphan
        } else if cfg.golden_dirname.contains("cycle") {
            AlignmentLayoutMode::Cycle
        } else {
            AlignmentLayoutMode::Group
        };

        run_alignment_layout(cfg, &network, &g2, align_file, mode).layout
    } else if cfg.layout == "hierdag" {
        // P6: HierDAG layout
        let mut params = LayoutParams {
            include_shadows: true,
            layout_mode: LayoutMode::PerNode,
            ..Default::default()
        };
        params.point_up = cfg.point_up;
        run_hierdag_layout(&network, &params)
    } else if cfg.layout == "world_bank" {
        // P10: WorldBank layout
        run_world_bank_layout(&network)
    } else if cfg.layout == "set" {
        let semantics = parse_set_semantics(cfg.link_means);
        run_set_layout(&network, semantics)
    } else if cfg.layout == "control_top" {
        let ctrl_mode = parse_control_order(cfg.control_mode);
        let targ_mode = parse_target_order(cfg.target_mode);
        run_control_top_layout(&network, ctrl_mode, targ_mode, cfg.control_nodes)
    } else if cfg.layout == "node_cluster" {
        let order = parse_cluster_order(cfg.cluster_order);
        let placement = parse_cluster_placement(cfg.cluster_placement);
        run_node_cluster_layout_with_params(&network, cfg.attribute_file.unwrap(), order, placement)
    } else if let Some(noa_file) = cfg.noa_file {
        // P11: Fixed node-order import
        let noa_path = noa_file_path(noa_file);
        assert!(noa_path.exists(), "NOA file not found: {}", noa_path.display());
        let node_order = parse_noa_format_file(&noa_path);
        run_layout_with_node_order(&network, node_order)
    } else if let Some(eda_file) = cfg.eda_file {
        // P11: Fixed link-order import from EDA file
        let eda_path = eda_file_path(eda_file);
        assert!(eda_path.exists(), "EDA file not found: {}", eda_path.display());
        run_layout_with_link_order(&network, &eda_path)
    } else if let Some(link_groups) = cfg.link_groups {
        // P3: Link grouping — reorder edges by relation type
        let mode = match cfg.group_mode {
            Some("per_node") => LayoutMode::PerNode,
            Some("per_network") => LayoutMode::PerNetwork,
            _ => LayoutMode::PerNode,
        };
        let groups: Vec<String> = link_groups.iter().map(|s| s.to_string()).collect();
        run_layout_with_groups(&network, &groups, mode)
    } else {
        match cfg.layout {
            "similarity_resort" if cfg.has_custom_similarity() => {
                run_similarity_layout_custom(&network, SimilarityMode::Resort,
                    cfg.similarity_pass_count, cfg.similarity_tolerance, cfg.similarity_chain_length)
            }
            "similarity_clustered" if cfg.has_custom_similarity() => {
                run_similarity_layout_custom(&network, SimilarityMode::Clustered,
                    cfg.similarity_pass_count, cfg.similarity_tolerance, cfg.similarity_chain_length)
            }
            "similarity_resort" => run_similarity_layout(&network, SimilarityMode::Resort),
            "similarity_clustered" => run_similarity_layout(&network, SimilarityMode::Clustered),
            _ => run_default_layout(&network),
        }
    };

    // The Java GoldenGenerator always exports the full EDA (including shadow
    // links) regardless of the shadow display setting. The shadow toggle only
    // affects visual rendering, not the EDA export format.
    let actual_eda = format_eda(&layout);

    assert_parity("EDA", &golden_eda, &actual_eda);
}

/// Build the BIF session for a parity test, returning the exported BIF string.
/// Shared logic used by run_bif_test, run_bif_test_nid_normalized, etc.
#[allow(dead_code)]
pub fn produce_bif_string(cfg: &ParityConfig) -> String {
    let session = if cfg.is_roundtrip {
        // Round-trip: read the golden BIF file as a Session
        let bif_path = golden_dir(cfg.golden_dirname).join("output.bif");
        assert!(bif_path.exists(), "BIF file not found: {}", bif_path.display());
        biofabric_core::io::xml::read_session(&bif_path).unwrap()
    } else {
        let input = network_path(cfg.input_file);
        assert!(input.exists(), "Input file not found: {}", input.display());

        let network = load_network(&input);

        // P12: Subnetwork extraction — uses full layout + compress approach
        // (must be done after layout, not before)
        let (network, extract_layout) = if let Some(extract_nodes) = cfg.extract_nodes {
            // First do the full layout, then extract/compress
            let full_layout = run_default_layout(&network);
            let (sub_net, sub_layout) = extract_submodel(&network, &full_layout, extract_nodes);
            (sub_net, Some(sub_layout))
        } else {
            (network, None)
        };

        // If extraction already produced a layout, use it; otherwise compute
        let (layout, network) = if let Some(el) = extract_layout {
            (el, network)
        } else if cfg.layout == "alignment" {
            let net2_file = cfg.align_net2.expect("alignment tests require align_net2");
            let align_file = cfg.align_file.expect("alignment tests require align_file");
            let g2_path = network_path(net2_file);
            assert!(g2_path.exists(), "G2 network not found: {}", g2_path.display());
            let g2 = load_network(&g2_path);
            let mode = if cfg.golden_dirname.contains("orphan") {
                AlignmentLayoutMode::Orphan
            } else if cfg.golden_dirname.contains("cycle") {
                AlignmentLayoutMode::Cycle
            } else {
                AlignmentLayoutMode::Group
            };
            let aligned = run_alignment_layout(cfg, &network, &g2, align_file, mode);
            (aligned.layout, aligned.network)
        } else if cfg.layout == "hierdag" {
            let mut params = biofabric_core::layout::traits::LayoutParams {
                include_shadows: true,
                layout_mode: LayoutMode::PerNode,
                ..Default::default()
            };
            params.point_up = cfg.point_up;
            (run_hierdag_layout(&network, &params), network)
        } else if cfg.layout == "world_bank" {
            (run_world_bank_layout(&network), network)
        } else if cfg.layout == "set" {
            let semantics = parse_set_semantics(cfg.link_means);
            (run_set_layout(&network, semantics), network)
        } else if cfg.layout == "control_top" {
            let ctrl_mode = parse_control_order(cfg.control_mode);
            let targ_mode = parse_target_order(cfg.target_mode);
            (
                run_control_top_layout(&network, ctrl_mode, targ_mode, cfg.control_nodes),
                network,
            )
        } else if cfg.layout == "node_cluster" {
            let order = parse_cluster_order(cfg.cluster_order);
            let placement = parse_cluster_placement(cfg.cluster_placement);
            (
                run_node_cluster_layout_with_params(&network, cfg.attribute_file.unwrap(), order, placement),
                network,
            )
        } else if let Some(noa_file) = cfg.noa_file {
            let noa_path = noa_file_path(noa_file);
            assert!(noa_path.exists(), "NOA file not found: {}", noa_path.display());
            let node_order = parse_noa_format_file(&noa_path);
            (run_layout_with_node_order(&network, node_order), network)
        } else if let Some(eda_file) = cfg.eda_file {
            // P11: Fixed link-order import from EDA file
            let eda_path = eda_file_path(eda_file);
            assert!(eda_path.exists(), "EDA file not found: {}", eda_path.display());
            (run_layout_with_link_order(&network, &eda_path), network)
        } else if let Some(link_groups) = cfg.link_groups {
            let mode = match cfg.group_mode {
                Some("per_node") => LayoutMode::PerNode,
                Some("per_network") => LayoutMode::PerNetwork,
                _ => LayoutMode::PerNode,
            };
            let groups: Vec<String> = link_groups.iter().map(|s| s.to_string()).collect();
            (run_layout_with_groups(&network, &groups, mode), network)
        } else {
            match cfg.layout {
                "similarity_resort" if cfg.has_custom_similarity() => {
                    (run_similarity_layout_custom(&network, SimilarityMode::Resort,
                        cfg.similarity_pass_count, cfg.similarity_tolerance, cfg.similarity_chain_length), network)
                }
                "similarity_clustered" if cfg.has_custom_similarity() => {
                    (run_similarity_layout_custom(&network, SimilarityMode::Clustered,
                        cfg.similarity_pass_count, cfg.similarity_tolerance, cfg.similarity_chain_length), network)
                }
                "similarity_resort" => (run_similarity_layout(&network, SimilarityMode::Resort), network),
                "similarity_clustered" => {
                    (run_similarity_layout(&network, SimilarityMode::Clustered), network)
                }
                _ => (run_default_layout(&network), network),
            }
        };

        // Build session
        let mut session = biofabric_core::io::session::Session::with_layout(network, layout);

        // Apply display option overrides
        if let Some(dz) = cfg.min_drain_zone {
            session.display_options.min_drain_zone = dz;
        }
        if let Some(nl) = cfg.node_lighter {
            session.display_options.node_lighter_level = nl as f64;
        }
        if let Some(ld) = cfg.link_darker {
            session.display_options.link_darker_level = ld as f64;
        }

        // Alignment cycle view explicitly writes shadows="true" in Java goldens
        if cfg.layout == "alignment"
            && cfg.golden_dirname.contains("cycle")
            && !cfg.golden_dirname.contains("cycle_noshadow")
        {
            session.display_options.shadows_explicit = true;
        }

        session
    };

    // ---- Export BIF ----
    biofabric_core::io::xml::write_session_string(&session).unwrap()
}

/// Run a BIF (session XML) parity test — exact byte-for-byte comparison.
#[allow(dead_code)]
pub fn run_bif_test(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.bif");
    let golden_bif = read_golden(cfg.golden_dirname, "output.bif");
    let actual_bif = produce_bif_string(cfg);
    assert_parity("BIF", &golden_bif, &actual_bif);
}

/// Same as run_bif_test but strips nid="..." attributes before comparing.
#[allow(dead_code)]
pub fn run_bif_test_nid_normalized(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.bif");
    let golden_bif = read_golden(cfg.golden_dirname, "output.bif");
    let actual_bif = produce_bif_string(cfg);
    assert_parity_bif_nid_normalized("BIF", &golden_bif, &actual_bif);
}

/// Run NOA test with unordered (set-based) comparison.
/// Only used for alignment tests (cycle layout) where HashSet ordering varies.
#[allow(dead_code)]
pub fn run_noa_test_unordered(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.noa");
    let golden_noa = read_golden(cfg.golden_dirname, "output.noa");

    let input = network_path(cfg.input_file);
    let network = load_network(&input);

    let net2_file = cfg.align_net2.expect("unordered NOA tests require alignment");
    let align_file = cfg.align_file.expect("unordered NOA tests require alignment");
    let g2_path = network_path(net2_file);
    let g2 = load_network(&g2_path);
    let mode = if cfg.golden_dirname.contains("orphan") {
        AlignmentLayoutMode::Orphan
    } else if cfg.golden_dirname.contains("cycle") {
        AlignmentLayoutMode::Cycle
    } else {
        AlignmentLayoutMode::Group
    };
    let aligned = run_alignment_layout(cfg, &network, &g2, align_file, mode);

    let actual_noa = format_noa(&aligned.layout);
    assert_parity_unordered("NOA", &golden_noa, &actual_noa);
}

/// Run EDA test with unordered (set-based) comparison.
/// Only used for alignment tests (cycle layout) where HashSet ordering varies.
#[allow(dead_code)]
pub fn run_eda_test_unordered(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.eda");
    let golden_eda = read_golden(cfg.golden_dirname, "output.eda");

    let input = network_path(cfg.input_file);
    let network = load_network(&input);

    let net2_file = cfg.align_net2.expect("unordered EDA tests require alignment");
    let align_file = cfg.align_file.expect("unordered EDA tests require alignment");
    let g2_path = network_path(net2_file);
    let g2 = load_network(&g2_path);
    let mode = if cfg.golden_dirname.contains("orphan") {
        AlignmentLayoutMode::Orphan
    } else if cfg.golden_dirname.contains("cycle") {
        AlignmentLayoutMode::Cycle
    } else {
        AlignmentLayoutMode::Group
    };
    let aligned = run_alignment_layout(cfg, &network, &g2, align_file, mode);

    let actual_eda = format_eda(&aligned.layout);
    assert_parity_unordered("EDA", &golden_eda, &actual_eda);
}

/// Run BIF test with NID normalization and unordered node/link sections.
#[allow(dead_code)]
pub fn run_bif_test_unordered(cfg: &ParityConfig) {
    golden_available(cfg.golden_dirname, "output.bif");
    let golden_bif = read_golden(cfg.golden_dirname, "output.bif");
    let actual_bif = produce_bif_string(cfg);
    let expected_norm = normalize_bif_nids(&golden_bif);
    let actual_norm = normalize_bif_nids(&actual_bif);
    assert_parity_bif_sections_unordered("BIF", &expected_norm, &actual_norm);
}
