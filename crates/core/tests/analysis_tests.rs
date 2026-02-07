// Analysis operation tests for biofabric-rs.
//
// These tests validate network analysis operations (cycle detection, Jaccard
// similarity, subnetwork extraction) against reference values from the Java
// BioFabric implementation.
//
// Unlike parity tests which compare full NOA/EDA/BIF output, analysis tests
// compare simpler text or numeric outputs. They are still stubbed (#[ignore])
// pending implementation of the analysis module.
//
// == Running ==
//
//   cargo test --test analysis_tests -- --include-ignored

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Test infrastructure
// ---------------------------------------------------------------------------

/// Root of the parity test data.
fn parity_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../../tests/parity")
}

/// Path to a test network input file.
fn network_path(filename: &str) -> PathBuf {
    let subdir = match filename.rsplit('.').next() {
        Some("sif") => "sif",
        Some("gw") => "gw",
        _ => "sif",
    };
    parity_root().join("networks").join(subdir).join(filename)
}

// ---------------------------------------------------------------------------
// Cycle detection tests
// ---------------------------------------------------------------------------

/// Test that cycle detection produces the expected boolean result.
#[allow(dead_code)]
fn run_cycle_test(input_file: &str, _expected_has_cycle: bool) {
    let input = network_path(input_file);
    assert!(input.exists(), "Input file not found: {}", input.display());

    // TODO: Load the network
    // let network = biofabric_core::io::sif::parse_file(&input).unwrap();

    // TODO: Run cycle detection
    // let has_cycle = biofabric_core::analysis::cycle_finder::has_cycle(&network);

    // TODO: Compare
    // assert_eq!(has_cycle, expected_has_cycle,
    //     "Cycle detection mismatch for {}: expected {}, got {}",
    //     input_file, expected_has_cycle, has_cycle);

    eprintln!("  Cycle detection: STUB (implementation pending)");
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_triangle() {
    // Triangle is undirected; Java treats as having potential cycle
    run_cycle_test("triangle.sif", true);
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_self_loop() {
    // Self-loop (A→A) is a cycle
    run_cycle_test("self_loop.sif", true);
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_dag_simple() {
    // DAG has no cycles
    run_cycle_test("dag_simple.sif", false);
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_dag_diamond() {
    // Diamond DAG has no cycles
    run_cycle_test("dag_diamond.sif", false);
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_dag_deep() {
    // Linear DAG has no cycles
    run_cycle_test("dag_deep.sif", false);
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_linear_chain() {
    // Linear chain (undirected interpretation)
    run_cycle_test("linear_chain.sif", false);
}

#[test]
#[ignore = "analysis: enable after implementing cycle detection"]
fn cycle_disconnected_components() {
    run_cycle_test("disconnected_components.sif", true);
}

// ---------------------------------------------------------------------------
// Jaccard similarity tests
// ---------------------------------------------------------------------------

/// Test that Jaccard similarity computation produces the expected value.
#[allow(dead_code)]
fn run_jaccard_test(input_file: &str, _node_a: &str, _node_b: &str, _expected: f64) {
    let input = network_path(input_file);
    assert!(input.exists(), "Input file not found: {}", input.display());

    // TODO: Load the network
    // let network = biofabric_core::io::sif::parse_file(&input).unwrap();

    // TODO: Compute Jaccard similarity
    // let similarity = biofabric_core::analysis::jaccard_similarity(&network, node_a, node_b);

    // TODO: Compare within tolerance
    // assert!((similarity - expected).abs() < 1e-10,
    //     "Jaccard similarity mismatch for {}({}, {}): expected {}, got {}",
    //     input_file, node_a, node_b, expected, similarity);

    eprintln!("  Jaccard similarity: STUB (implementation pending)");
}

#[test]
#[ignore = "analysis: enable after implementing Jaccard similarity"]
fn jaccard_triangle_ab() {
    // In a triangle, A and B share neighbor C → J = |{C}| / |{B,C} ∪ {A,C}| = 1/2
    run_jaccard_test("triangle.sif", "A", "B", 0.5);
}

#[test]
#[ignore = "analysis: enable after implementing Jaccard similarity"]
fn jaccard_triangle_ac() {
    // Symmetric: same as A-B
    run_jaccard_test("triangle.sif", "A", "C", 0.5);
}

#[test]
#[ignore = "analysis: enable after implementing Jaccard similarity"]
fn jaccard_multi_relation_ad() {
    // A connects to {B,C,E}, D connects to {C,E} → shared={C,E}, union={B,C,E}
    run_jaccard_test("multi_relation.sif", "A", "D", 2.0 / 3.0);
}

#[test]
#[ignore = "analysis: enable after implementing Jaccard similarity"]
fn jaccard_star500_hub_leaf() {
    // Hub connects to all 500 leaves, leaf connects only to hub → J = 1/500
    run_jaccard_test("star-500.sif", "0", "1", 0.0);
}

#[test]
#[ignore = "analysis: enable after implementing Jaccard similarity"]
fn jaccard_dense_clique_ab() {
    // In K6, every node connects to 5 others. A and B share 4 neighbors.
    // J = |{C,D,E,F}| / |{B,C,D,E,F} ∪ {A,C,D,E,F}| = 4/6
    run_jaccard_test("dense_clique.sif", "A", "B", 4.0 / 6.0);
}

// ---------------------------------------------------------------------------
// Subnetwork extraction tests
// ---------------------------------------------------------------------------

/// Test that subnetwork extraction produces the expected induced subgraph.
#[allow(dead_code)]
fn run_extraction_test(
    input_file: &str,
    _nodes: &[&str],
    _expected_node_count: usize,
    _expected_edge_count: usize,
) {
    let input = network_path(input_file);
    assert!(input.exists(), "Input file not found: {}", input.display());

    // TODO: Load the network, extract subnetwork, verify counts
    eprintln!("  Subnetwork extraction: STUB (implementation pending)");
}

#[test]
#[ignore = "analysis: enable after implementing subnetwork extraction"]
fn extract_triangle_ab() {
    // Extract {A,B} from triangle: 2 nodes, 1 edge (A-B)
    run_extraction_test("triangle.sif", &["A", "B"], 2, 1);
}

#[test]
#[ignore = "analysis: enable after implementing subnetwork extraction"]
fn extract_multi_relation_ace() {
    // Extract {A,C,E} from multi_relation:
    // Edges: A-pd-C, A-gi-E → 2 edges
    run_extraction_test("multi_relation.sif", &["A", "C", "E"], 3, 2);
}

#[test]
#[ignore = "analysis: enable after implementing subnetwork extraction"]
fn extract_dense_clique_abc() {
    // Extract {A,B,C} from K6: complete subgraph K3 = 3 edges
    run_extraction_test("dense_clique.sif", &["A", "B", "C"], 3, 3);
}
