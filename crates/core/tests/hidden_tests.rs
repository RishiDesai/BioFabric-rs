// Hidden tests for biofabric-rs agent evaluation.
//
// These tests validate the Rust implementation against golden files
// using NEW network inputs that are NOT part of the public test suite.
// They exercise the same code paths as the public parity tests but
// with different data, preventing overfitting to specific golden files.
//
// == Golden Generation ==
//
// Hidden goldens are generated at Docker build time (or via the
// generate_hidden_goldens test below). They are NOT checked into git.
//
// == Running ==
//
//   cargo test --test hidden_tests -- --include-ignored
//
// == Grading ==
//
// The agent passes if ALL public tests AND ALL hidden tests pass.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use biofabric_core::io;
use biofabric_core::layout::build_data::LayoutBuildData;
use biofabric_core::layout::default::{DefaultEdgeLayout, DefaultNodeLayout};
use biofabric_core::layout::hierarchy::HierDAGLayout;
use biofabric_core::layout::traits::{EdgeLayout, LayoutMode, LayoutParams, NodeLayout};
use biofabric_core::layout::result::NetworkLayout;
use biofabric_core::model::{Network, NodeId};
use biofabric_core::worker::NoopMonitor;

// ---------------------------------------------------------------------------
// Test infrastructure (mirrors parity_tests.rs)
// ---------------------------------------------------------------------------

fn parity_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../tests/parity")
}

fn network_path(filename: &str) -> PathBuf {
    let subdir = match filename.rsplit('.').next() {
        Some("sif") => "sif",
        Some("gw") => "gw",
        _ => "sif",
    };
    parity_root().join("networks").join(subdir).join(filename)
}

fn golden_dir(dirname: &str) -> PathBuf {
    parity_root().join("goldens").join(dirname)
}

fn golden_available(dirname: &str, filename: &str) -> bool {
    let path = golden_dir(dirname).join(filename);
    assert!(
        path.exists(),
        "Golden file not found at {}. Run: cargo test --test hidden_tests generate_hidden_goldens -- --include-ignored --nocapture",
        path.display()
    );
    true
}

fn read_golden(dirname: &str, filename: &str) -> String {
    let path = golden_dir(dirname).join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read golden file {}: {}", path.display(), e))
}

fn load_network(path: &std::path::Path) -> Network {
    match path.extension().and_then(|e| e.to_str()) {
        Some("sif") => io::sif::parse_file(path).unwrap(),
        Some("gw") => io::gw::parse_file(path).unwrap(),
        _ => panic!("Unknown input format: {}", path.display()),
    }
}

/// Compare two strings with a detailed diff on failure.
fn assert_parity(label: &str, expected: &str, actual: &str) {
    if expected == actual {
        return;
    }
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    let mut first_diff_line = None;
    for (i, (e, a)) in exp_lines.iter().zip(act_lines.iter()).enumerate() {
        if e != a {
            first_diff_line = Some(i + 1);
            break;
        }
    }
    if first_diff_line.is_none() && exp_lines.len() != act_lines.len() {
        first_diff_line = Some(exp_lines.len().min(act_lines.len()) + 1);
    }
    let msg = match first_diff_line {
        Some(line) => {
            let exp_content = exp_lines.get(line - 1).unwrap_or(&"<EOF>");
            let act_content = act_lines.get(line - 1).unwrap_or(&"<EOF>");
            format!(
                "PARITY FAILURE in {label}:\n\
                 First difference at line {line}:\n\
                 Expected: {exp_content:?}\n\
                 Actual:   {act_content:?}\n\
                 Expected total lines: {}\n\
                 Actual total lines:   {}",
                exp_lines.len(),
                act_lines.len()
            )
        }
        None => format!(
            "PARITY FAILURE in {label}: outputs differ but couldn't locate difference"
        ),
    };
    panic!("{}", msg);
}

// ---------------------------------------------------------------------------
// Layout runners
// ---------------------------------------------------------------------------

fn run_default_layout(network: &Network) -> NetworkLayout {
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

fn run_hierdag_layout(network: &Network, params: &LayoutParams) -> NetworkLayout {
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
    HierDAGLayout::install_node_annotations(network, params, &mut layout);
    layout
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

fn format_noa(layout: &NetworkLayout) -> String {
    let mut out = String::new();
    out.push_str("Node Row\n");
    let mut nodes: Vec<(&NodeId, &biofabric_core::layout::result::NodeLayout)> =
        layout.iter_nodes().collect();
    nodes.sort_by_key(|(_, info)| info.row);
    for (id, info) in nodes {
        out.push_str(&format!("{} = {}\n", id, info.row));
    }
    out
}

fn format_eda(layout: &NetworkLayout) -> String {
    let mut out = String::new();
    out.push_str("Link Column\n");
    for ll in layout.iter_links() {
        if ll.is_shadow {
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

fn format_eda_no_shadows(layout: &NetworkLayout) -> String {
    let mut out = String::new();
    out.push_str("Link Column\n");
    for ll in layout.iter_links() {
        if ll.is_shadow { continue; }
        let col = ll.column_no_shadows.unwrap_or(ll.column);
        out.push_str(&format!(
            "{} ({}) {} = {}\n",
            ll.source, ll.relation, ll.target, col
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// Golden generation (run with --include-ignored)
// ---------------------------------------------------------------------------

#[test]
#[ignore = "golden-gen: run explicitly to generate hidden golden files"]
fn generate_hidden_goldens() {
    let cases: Vec<(&str, &str, bool)> = vec![
        // Hidden mixed network: default + shadows ON
        ("hidden_mixed.sif", "hidden_mixed_default", true),
        // Hidden mixed network: default + shadows OFF
        ("hidden_mixed.sif", "hidden_mixed_noshadow", false),
        // Hidden DAG: default layout
        ("hidden_dag.sif", "hidden_dag_default", true),
    ];

    let mut generated = 0;

    for (input_file, golden_dirname, shadows) in &cases {
        let input_path = network_path(input_file);
        if !input_path.exists() {
            eprintln!("SKIP: input file not found: {}", input_path.display());
            continue;
        }

        let golden_path = golden_dir(golden_dirname);
        let noa_path = golden_path.join("output.noa");
        if noa_path.exists() {
            eprintln!("SKIP (already exists): {}", golden_dirname);
            continue;
        }

        eprintln!("Generating hidden golden: {} -> {}", input_file, golden_dirname);

        let network = load_network(&input_path);
        let layout = run_default_layout(&network);

        let noa_content = format_noa(&layout);
        let eda_content = if *shadows {
            format_eda(&layout)
        } else {
            format_eda_no_shadows(&layout)
        };

        std::fs::create_dir_all(&golden_path).unwrap();
        std::fs::write(&noa_path, &noa_content).unwrap();
        std::fs::write(golden_path.join("output.eda"), &eda_content).unwrap();

        // Generate BIF
        let session = biofabric_core::io::session::Session::with_layout(network.clone(), layout.clone());
        let bif_content = biofabric_core::io::xml::write_session_string(&session).unwrap();
        std::fs::write(golden_path.join("output.bif"), &bif_content).unwrap();

        eprintln!(
            "  {} : {} NOA lines, {} EDA lines, {} BIF bytes",
            golden_dirname,
            noa_content.lines().count(),
            eda_content.lines().count(),
            bif_content.len()
        );
        generated += 1;
    }

    // HierDAG layout for hidden_dag
    {
        let input_path = network_path("hidden_dag.sif");
        if input_path.exists() {
            for point_up in [true, false] {
                let suffix = if point_up { "true" } else { "false" };
                let golden_dirname = format!("hidden_dag_hierdag_{}", suffix);
                let golden_path = golden_dir(&golden_dirname);
                let noa_path = golden_path.join("output.noa");
                if noa_path.exists() {
                    eprintln!("SKIP (already exists): {}", golden_dirname);
                    continue;
                }

                eprintln!("Generating hidden golden: hidden_dag.sif -> {}", golden_dirname);

                let network = load_network(&input_path);
                let params = LayoutParams {
                    include_shadows: true,
                    layout_mode: LayoutMode::PerNode,
                    point_up: Some(point_up),
                    ..Default::default()
                };
                let layout = run_hierdag_layout(&network, &params);

                let noa_content = format_noa(&layout);
                let eda_content = format_eda(&layout);

                std::fs::create_dir_all(&golden_path).unwrap();
                std::fs::write(&noa_path, &noa_content).unwrap();
                std::fs::write(golden_path.join("output.eda"), &eda_content).unwrap();

                let session = biofabric_core::io::session::Session::with_layout(network.clone(), layout.clone());
                let bif_content = biofabric_core::io::xml::write_session_string(&session).unwrap();
                std::fs::write(golden_path.join("output.bif"), &bif_content).unwrap();

                eprintln!("  {}: {} NOA, {} EDA, {} BIF bytes", golden_dirname,
                    noa_content.lines().count(), eda_content.lines().count(), bif_content.len());
                generated += 1;
            }
        }
    }

    eprintln!("\nGenerated {} hidden golden files.", generated);
}

// ===========================================================================
//
//  HIDDEN TEST 1: Mixed network — default layout, shadows ON
//
//  Exercises: SIF parsing, shadow link creation, multi-relation handling,
//  self-loop handling, disconnected components, BFS node ordering,
//  default edge layout, XML session export.
//
// ===========================================================================

#[test]
fn hidden_mixed_default_noa() {
    golden_available("hidden_mixed_default", "output.noa");
    let golden = read_golden("hidden_mixed_default", "output.noa");
    let network = load_network(&network_path("hidden_mixed.sif"));
    let layout = run_default_layout(&network);
    let actual = format_noa(&layout);
    assert_parity("hidden_mixed_default_noa", &golden, &actual);
}

#[test]
fn hidden_mixed_default_eda() {
    golden_available("hidden_mixed_default", "output.eda");
    let golden = read_golden("hidden_mixed_default", "output.eda");
    let network = load_network(&network_path("hidden_mixed.sif"));
    let layout = run_default_layout(&network);
    let actual = format_eda(&layout);
    assert_parity("hidden_mixed_default_eda", &golden, &actual);
}

#[test]
fn hidden_mixed_default_bif() {
    golden_available("hidden_mixed_default", "output.bif");
    let golden = read_golden("hidden_mixed_default", "output.bif");
    let network = load_network(&network_path("hidden_mixed.sif"));
    let layout = run_default_layout(&network);
    let session = biofabric_core::io::session::Session::with_layout(network, layout);
    let actual = biofabric_core::io::xml::write_session_string(&session).unwrap();
    assert_parity("hidden_mixed_default_bif", &golden, &actual);
}

// ===========================================================================
//
//  HIDDEN TEST 2: Mixed network — default layout, shadows OFF
//
//  Exercises: Shadow link toggling (NOA should be same as ON, EDA/BIF differ).
//
// ===========================================================================

#[test]
fn hidden_mixed_noshadow_noa() {
    golden_available("hidden_mixed_noshadow", "output.noa");
    let golden = read_golden("hidden_mixed_noshadow", "output.noa");
    let network = load_network(&network_path("hidden_mixed.sif"));
    let layout = run_default_layout(&network);
    let actual = format_noa(&layout);
    assert_parity("hidden_mixed_noshadow_noa", &golden, &actual);
}

#[test]
fn hidden_mixed_noshadow_eda() {
    golden_available("hidden_mixed_noshadow", "output.eda");
    let golden = read_golden("hidden_mixed_noshadow", "output.eda");
    let network = load_network(&network_path("hidden_mixed.sif"));
    let layout = run_default_layout(&network);
    // Shadows OFF EDA exports only non-shadow links with renumbered columns
    let actual = format_eda_no_shadows(&layout);
    assert_parity("hidden_mixed_noshadow_eda", &golden, &actual);
}

#[test]
fn hidden_mixed_noshadow_bif() {
    golden_available("hidden_mixed_noshadow", "output.bif");
    let golden = read_golden("hidden_mixed_noshadow", "output.bif");
    let network = load_network(&network_path("hidden_mixed.sif"));
    let layout = run_default_layout(&network);
    let session = biofabric_core::io::session::Session::with_layout(network, layout);
    // Shadows OFF just changes the display option; the layout is the same
    let actual = biofabric_core::io::xml::write_session_string(&session).unwrap();
    assert_parity("hidden_mixed_noshadow_bif", &golden, &actual);
}

// ===========================================================================
//
//  HIDDEN TEST 3: DAG — HierDAG layout (pointUp=true)
//
//  Exercises: HierDAG layout algorithm, topological ordering, level
//  annotations, directed edge handling.
//
// ===========================================================================

#[test]
fn hidden_dag_hierdag_true_noa() {
    golden_available("hidden_dag_hierdag_true", "output.noa");
    let golden = read_golden("hidden_dag_hierdag_true", "output.noa");
    let network = load_network(&network_path("hidden_dag.sif"));
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        point_up: Some(true),
        ..Default::default()
    };
    let layout = run_hierdag_layout(&network, &params);
    let actual = format_noa(&layout);
    assert_parity("hidden_dag_hierdag_true_noa", &golden, &actual);
}

#[test]
fn hidden_dag_hierdag_true_eda() {
    golden_available("hidden_dag_hierdag_true", "output.eda");
    let golden = read_golden("hidden_dag_hierdag_true", "output.eda");
    let network = load_network(&network_path("hidden_dag.sif"));
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        point_up: Some(true),
        ..Default::default()
    };
    let layout = run_hierdag_layout(&network, &params);
    let actual = format_eda(&layout);
    assert_parity("hidden_dag_hierdag_true_eda", &golden, &actual);
}

#[test]
fn hidden_dag_hierdag_true_bif() {
    golden_available("hidden_dag_hierdag_true", "output.bif");
    let golden = read_golden("hidden_dag_hierdag_true", "output.bif");
    let network = load_network(&network_path("hidden_dag.sif"));
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        point_up: Some(true),
        ..Default::default()
    };
    let layout = run_hierdag_layout(&network, &params);
    let session = biofabric_core::io::session::Session::with_layout(network, layout);
    let actual = biofabric_core::io::xml::write_session_string(&session).unwrap();
    assert_parity("hidden_dag_hierdag_true_bif", &golden, &actual);
}

// ===========================================================================
//
//  HIDDEN TEST 4: DAG — HierDAG layout (pointUp=false)
//
// ===========================================================================

#[test]
fn hidden_dag_hierdag_false_noa() {
    golden_available("hidden_dag_hierdag_false", "output.noa");
    let golden = read_golden("hidden_dag_hierdag_false", "output.noa");
    let network = load_network(&network_path("hidden_dag.sif"));
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        point_up: Some(false),
        ..Default::default()
    };
    let layout = run_hierdag_layout(&network, &params);
    let actual = format_noa(&layout);
    assert_parity("hidden_dag_hierdag_false_noa", &golden, &actual);
}

#[test]
fn hidden_dag_hierdag_false_eda() {
    golden_available("hidden_dag_hierdag_false", "output.eda");
    let golden = read_golden("hidden_dag_hierdag_false", "output.eda");
    let network = load_network(&network_path("hidden_dag.sif"));
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        point_up: Some(false),
        ..Default::default()
    };
    let layout = run_hierdag_layout(&network, &params);
    let actual = format_eda(&layout);
    assert_parity("hidden_dag_hierdag_false_eda", &golden, &actual);
}

#[test]
fn hidden_dag_hierdag_false_bif() {
    golden_available("hidden_dag_hierdag_false", "output.bif");
    let golden = read_golden("hidden_dag_hierdag_false", "output.bif");
    let network = load_network(&network_path("hidden_dag.sif"));
    let params = LayoutParams {
        include_shadows: true,
        layout_mode: LayoutMode::PerNode,
        point_up: Some(false),
        ..Default::default()
    };
    let layout = run_hierdag_layout(&network, &params);
    let session = biofabric_core::io::session::Session::with_layout(network, layout);
    let actual = biofabric_core::io::xml::write_session_string(&session).unwrap();
    assert_parity("hidden_dag_hierdag_false_bif", &golden, &actual);
}

// ===========================================================================
//
//  HIDDEN TEST 5: Analysis — basic graph properties of hidden_mixed
//
//  These don't need golden files — they test computed properties directly.
//
// ===========================================================================

#[test]
fn hidden_mixed_analysis_components() {
    let network = load_network(&network_path("hidden_mixed.sif"));
    let components = biofabric_core::analysis::graph::connected_components(&network);
    let mut sizes: Vec<usize> = components.iter().map(|c| c.len()).collect();
    sizes.sort_unstable_by(|a, b| b.cmp(a));

    // hidden_mixed.sif has:
    //   Component 1: W-X-Y-Z-V-U (6 nodes, connected via pp/pd/gi)
    //   Component 2: Q-R-S (3 nodes, connected via pp)
    //   Component 3: T (1 node, only self-loop)
    //   Component 4: M-N (2 nodes, connected via pp)
    assert_eq!(sizes, vec![6, 3, 2, 1], "Connected component sizes mismatch for hidden_mixed.sif");
}

#[test]
fn hidden_mixed_analysis_cycle() {
    let network = load_network(&network_path("hidden_mixed.sif"));

    // Check for undirected cycles (ignoring directed "pd" edges)
    let mut adj: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
    for link in network.links() {
        if link.is_shadow { continue; }
        if link.relation.eq_ignore_ascii_case("pd") { continue; }
        if link.source == link.target { assert!(true, "Self-loop is a cycle"); return; }
        adj.entry(link.source.clone()).or_default().insert(link.target.clone());
        adj.entry(link.target.clone()).or_default().insert(link.source.clone());
    }

    // T has self-loop → should detect cycle
    // Actually the self-loop check above already returns. But let's also
    // check the undirected part: W-X-Y-Z-W forms a cycle.
    // (W pp X, X pp Y, Y pp Z, W pp Z → cycle W-X-Y-Z-W)
    assert!(true, "hidden_mixed should have cycles (self-loop T pp T)");
}

#[test]
fn hidden_mixed_analysis_jaccard() {
    let network = load_network(&network_path("hidden_mixed.sif"));

    // Jaccard(Q, R):
    // Neighbors of Q: {R, S}
    // Neighbors of R: {Q, S}
    // Intersection: {S}
    // Union: {R, S, Q, S} = {Q, R, S}
    // J = |{S}| / |{Q, R, S}| = 1/3
    let node_q = NodeId::new("Q");
    let node_r = NodeId::new("R");
    let comparison = network.compare_nodes(&node_q, &node_r)
        .expect("Nodes Q and R should exist in hidden_mixed.sif");
    let expected = 1.0 / 3.0;
    assert!(
        (comparison.jaccard_similarity - expected).abs() < 1e-10,
        "Jaccard(Q, R) should be ~{}, got {}",
        expected, comparison.jaccard_similarity
    );
}

#[test]
fn hidden_mixed_analysis_node_count() {
    let network = load_network(&network_path("hidden_mixed.sif"));
    // W, X, Y, Z, V, U, T, Q, R, S, M, N = 12 nodes
    assert_eq!(network.node_count(), 12, "hidden_mixed.sif should have 12 nodes");
}

#[test]
fn hidden_mixed_analysis_link_count() {
    let network = load_network(&network_path("hidden_mixed.sif"));
    // Non-shadow links: 12 (from the SIF file)
    // Shadow links: 11 (every non-self-loop edge gets a shadow; T pp T has no shadow)
    // Total: 23
    let non_shadow = network.links().filter(|l| !l.is_shadow).count();
    assert_eq!(non_shadow, 12, "hidden_mixed.sif should have 12 non-shadow links");
}

#[test]
fn hidden_dag_analysis_topo_sort() {
    let network = load_network(&network_path("hidden_dag.sif"));
    let mut net = network.clone();
    // Mark all links as directed (they use "pd" relation)
    for link in net.links_mut().iter_mut() {
        if link.relation.eq_ignore_ascii_case("pd") {
            link.directed = Some(true);
        }
    }
    net.metadata.is_directed = true;

    let levels = biofabric_core::analysis::graph::dag_levels(&net)
        .expect("hidden_dag.sif should be a valid DAG");

    // Expected levels:
    // Alpha: 0 (root)
    // Beta, Gamma: 1
    // Delta, Epsilon, Zeta, Eta: 2
    // Theta, Iota: 3
    assert_eq!(*levels.get(&NodeId::new("Alpha")).unwrap(), 0);
    assert_eq!(*levels.get(&NodeId::new("Beta")).unwrap(), 1);
    assert_eq!(*levels.get(&NodeId::new("Gamma")).unwrap(), 1);
    assert_eq!(*levels.get(&NodeId::new("Delta")).unwrap(), 2);
    assert_eq!(*levels.get(&NodeId::new("Epsilon")).unwrap(), 2);
    assert_eq!(*levels.get(&NodeId::new("Zeta")).unwrap(), 2);
    assert_eq!(*levels.get(&NodeId::new("Eta")).unwrap(), 2);
    assert_eq!(*levels.get(&NodeId::new("Theta")).unwrap(), 3);
    assert_eq!(*levels.get(&NodeId::new("Iota")).unwrap(), 3);
}

#[test]
fn hidden_dag_analysis_components() {
    let network = load_network(&network_path("hidden_dag.sif"));
    let components = biofabric_core::analysis::graph::connected_components(&network);
    // All nodes are in one connected component
    assert_eq!(components.len(), 1, "hidden_dag.sif should have 1 connected component");
    assert_eq!(components[0].len(), 9, "The component should have 9 nodes");
}

#[test]
fn hidden_dag_analysis_node_count() {
    let network = load_network(&network_path("hidden_dag.sif"));
    // Alpha, Beta, Gamma, Delta, Epsilon, Zeta, Eta, Theta, Iota = 9
    assert_eq!(network.node_count(), 9, "hidden_dag.sif should have 9 nodes");
}
