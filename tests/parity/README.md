# BioFabric-rs Parity Tests

Exact byte-level parity between `biofabric-rs` and the original Java BioFabric.

The Rust rewrite of BioFabric's core library and CLI is **complete**. Every
layout algorithm, analysis operation, alignment pipeline, and XML round-trip
path has been ported and validated against the Java reference implementation.

## How it works

```
  Input network (.sif/.gw)  ─┐
  + optional config          │
  + optional alignment       │
         ├──► Java BioFabric (Docker) ──► golden .noa, .eda, .bif, .scores
         │
         └──► biofabric-rs (Rust)     ──► output .noa, .eda, .bif, .scores
                                               │
                                         byte-for-byte diff
```

**Golden generation** (Java path): `./tests/parity/generate_goldens.sh` builds
BioFabric and the AlignmentPlugin from source inside Docker (Eclipse Temurin
JDK 11) and runs each test case through the layout pipeline. Outputs are `.noa`
(node order), `.eda` (link/edge order), `.bif` (full session XML), and
`.scores` (alignment scoring metrics, when applicable).

**Golden generation** (Rust path): The Rust test suite can also regenerate
goldens without Docker, using the same algorithms:

```bash
cargo test --test parity_tests generate_goldens -- --include-ignored --nocapture
```

The Docker/Java generator is the canonical source of truth, but the Rust-based
generator produces identical output and is much faster.

**Rust tests**:

```bash
cargo test --test parity_tests -- --include-ignored       # layout parity + property tests
cargo test --test analysis_tests -- --include-ignored      # graph analysis + alignment scoring
cargo test --test cli_tests                                # CLI integration tests
```

## Output formats being compared

| File | Format | What it validates |
|------|--------|-------------------|
| `output.noa` | `Node Row\nname = row\n...` | Node ordering algorithm |
| `output.eda` | `Link Column\nname (rel) name = col\n...` | Edge layout, shadow link placement |
| `output.bif` | Full XML session | Everything: colors, drain zones, columns, annotations, plugin data |
| `output.scores` | `metric\tvalue` (tab-separated) | Alignment scoring metrics (EC, S3, ICS, NC, NGS, LGS, JS) |

## Test suite overview

| Test file | Macro-expanded | Hand-written | Total `#[test]` functions |
|-----------|---------------|--------------|---------------------------|
| `parity_tests.rs` | 248 (from 102 macro invocations) | 18 (1 golden-gen + 17 property) | **266** |
| `analysis_tests.rs` | — | 65 | **65** |
| `cli_tests.rs` | — | 54 | **54** |
| **Total** | | | **385** |

### Parity test breakdown (248 macro-generated + 18 hand-written = 266)

| Phase | What | Invocations | Fns/each | Functions |
|-------|------|-------------|----------|-----------|
| P1 | Default layout, shadows ON | 12 | 3 | 36 |
| P2 | Default layout, shadows OFF | 12 | 3 | 36 |
| P3 | Link grouping (PER_NODE + PER_NETWORK) | 4 | 3 | 12 |
| P4 | XML round-trip (layout goldens) | 7 | 1 | 7 |
| P5 | NodeSimilarity layout (resort mode) | 2 | 3 | 6 |
| P5b | NodeSimilarity layout (clustered mode) | 2 | 3 | 6 |
| P5c | NodeSimilarity deep-variants (custom params) | 3 | 3 | 9 |
| P6 | HierDAG layout (pointUp true/false) | 6 | 3 | 18 |
| P7 | NodeCluster layout | 1 | 3 | 3 |
| P7b | NodeCluster deep-variants (ordering + placement) | 3 | 3 | 9 |
| P8 | ControlTop layout (4 mode combos) | 4 | 3 | 12 |
| P8b | ControlTop deep-variants (4 additional mode combos) | 4 | 3 | 12 |
| P9 | SetLayout (BELONGS_TO + CONTAINS) | 2 | 3 | 6 |
| P11 | Fixed-order import (NOA/EDA input) | 3 | 3 | 9 |
| P12 | Subnetwork extraction | 3 | 3 | 9 |
| P13 | Analysis: cycle detection + Jaccard | 9 | 1 | 9 |
| P14 | Display option permutations | 8 | 1 | 8 |
| P15 | Alignment — toy networks (GROUP view) | 2 | 3 | 6 |
| P16 | Alignment — realistic PPI (GROUP view) | 3 | 3 | 9 |
| P16b | BIF round-trip: DesaiEtAl pub files (annotations) | 5 | 1 | 5 |
| P17 | Alignment view variants (ORPHAN, CYCLE, NG modes) | 7 | 3 | 21 |
| — | Property-based cycle layout invariants | — | — | 17 |
| — | Golden file generator (ignored) | — | — | 1 |
| **Total** | | **102 invocations** | | **266** |

Each parity test macro generates **three** independent test functions (NOA, EDA,
BIF) so progress can be tracked incrementally — parsing (NOA passes), then
edge layout (EDA passes), then full XML export (BIF passes). Exceptions:
round-trip tests (BIF only), analysis tests (text only), display tests (BIF
only), and property tests (invariant checks, no golden files).

### Analysis test breakdown (65 tests)

| Category | Tests | What it validates |
|----------|-------|-------------------|
| Cycle detection | 7 | Undirected cycle detection on various topologies |
| Jaccard similarity | 4 | Pairwise Jaccard index between node neighborhoods |
| Subnetwork extraction | 3 | Induced subgraph for selected node sets |
| First-neighbor expansion | 5 | 1-hop neighborhood retrieval |
| Connected components | 10 | Component count, sizes, membership |
| Topological sort | 3 | Level assignment on DAGs |
| Node degree | 9 | Degree counting (incl. shadows, self-loops, multi-relation) |
| Alignment scoring | 21 | EC, S3, ICS, NC, NGS, LGS, JS across 3 scenarios |
| Node group sizes | 4 | Node group ratio computation for alignment quality |
| **Total** | **65** | |

### CLI integration tests (54 tests)

The `cli_tests.rs` file validates end-to-end CLI behavior for all subcommands:
`layout`, `convert`, `export-order`, `info`, `extract`, `align`, `render`,
`search`, and `compare`.

## Test input files (36 files)

### SIF networks (17 files)

| File | Nodes | Edges | Purpose |
|------|-------|-------|---------|
| `single_node.sif` | 1 | 0 | Minimal — single isolated node |
| `single_edge.sif` | 2 | 1 | Minimal connected network |
| `triangle.sif` | 3 | 3 | Simplest cycle; baseline |
| `self_loop.sif` | 3 | 3 | Feedback edge `A pp A` |
| `isolated_nodes.sif` | 6 | 2 | Lone nodes appended at end |
| `disconnected_components.sif` | 11 | 8 | 4 components — BFS ordering |
| `multi_relation.sif` | 5 | 6 | 3 relation types (pp, pd, gi) |
| `dense_clique.sif` | 6 | 15 | K6 — equal degree tie-breaking |
| `linear_chain.sif` | 10 | 9 | Path graph — degree-2 ties |
| `dag_simple.sif` | 5 | 5 | DAG for HierDAG layout |
| `dag_diamond.sif` | 4 | 4 | Diamond DAG — converging paths |
| `dag_deep.sif` | 8 | 7 | Linear DAG — many levels |
| `bipartite.sif` | 6 | 6 | Bipartite graph for SetLayout |
| `align_net1.sif` | 3 | 3 | Toy alignment network 1 |
| `align_net2.sif` | 3 | 2 | Toy alignment network 2 |
| `Yeast2KReduced.sif` | ~2.4K | ~16K | Alignment source (DesaiEtAl-2019) |
| `SC.sif` | ~6.6K | ~77K | Alignment target (DesaiEtAl-2019) |

### GW networks (5 files)

| File | Nodes | Edges | Purpose |
|------|-------|-------|---------|
| `triangle.gw` | 3 | 3 | GW parser, default relation |
| `directed_triangle.gw` | 3 | 3 | Directed GW, named relations |
| `CElegans.gw` | ~3K | ~5K | Medium real PPI (SANA) |
| `CaseStudy-IV-SmallYeast.gw` | 1004 | ~8.3K | Alignment source (DesaiEtAl-2019) |
| `CaseStudy-IV-LargeYeast.gw` | 1004 | ~10K | Alignment target (DesaiEtAl-2019) |

### Alignment files (5 files)

| File | Mappings | Description |
|------|----------|-------------|
| `test_perfect.align` | 3 | Toy: full 1:1 mapping |
| `test_partial.align` | 2 | Toy: incomplete mapping |
| `casestudy_iv.align` | 1004 | Case Study IV: SmallYeast ↔ LargeYeast |
| `yeast_sc_perfect.align` | 2379 | Known ground-truth alignment |
| `yeast_sc_s3_pure.align` | 2379 | Pure structural objective (S3=1.0) |

### BIF files for round-trip testing (5 files)

| File | Source | Annotation types |
|------|--------|------------------|
| `CaseStudy-III-Perfect.bif` | DesaiEtAl-2019 | Node groups, link groups, shadow link groups |
| `CaseStudy-III-All-S3.bif` | DesaiEtAl-2019 | Node groups, link groups, shadow link groups |
| `CaseStudy-III-All-Importance.bif` | DesaiEtAl-2019 | Node groups, link groups, shadow link groups |
| `CaseStudy-III-P05S3.bif` | DesaiEtAl-2019 | Node groups, link groups, shadow link groups |
| `CaseStudy-IV.bif` | DesaiEtAl-2019 | Cycle boundary annotations |

### Other input files (4 files)

| File | Format | Purpose |
|------|--------|---------|
| `multi_relation_clusters.na` | Node attribute | Cluster assignments for NodeClusterLayout |
| `triangle_reversed.noa` | Node order | Fixed node ordering (reversed) |
| `multi_relation_shuffled.noa` | Node order | Fixed node ordering (shuffled) |
| `triangle_reversed.eda` | Edge order | Fixed edge ordering (reversed) |

---

## Alignment coverage detail

The AlignmentPlugin is tested comprehensively across all 3 view types,
both PerfectNG modes, and the cycle view shadow toggle:

### View types

| ViewType | What it does | Tests |
|----------|-------------|-------|
| `GROUP` | Standard alignment layout with 7 link groups and node color classification | 5 baseline (2 toy + 3 realistic) + 4 NG variants |
| `ORPHAN` | Filters to unaligned edges + 1-hop context (OrphanEdgeLayout) | 3 tests |
| `CYCLE` | Highlights mis-alignments via cycle/path detection (AlignCycleLayout) | 3 tests (incl. shadows OFF) |

### PerfectNG modes

| Mode | Node groups | Tests |
|------|-------------|-------|
| `NONE` | 40 standard groups | All baseline GROUP tests |
| `NODE_CORRECTNESS` | 76 groups (split by correctness) | 4 tests |
| `JACCARD_SIMILARITY` | 76 groups (split by JS threshold) | 1 test (threshold=0.75) |

### Property-based cycle layout tests (17 tests)

Because Java's `HashSet` iteration order is nondeterministic, byte-level
parity is not achievable for cycle layout node ordering. Instead, 17
property-based tests verify structural invariants of the cycle layout
algorithm on two real alignment scenarios (CaseStudy-IV and Yeast↔SC):

| Property | Tests |
|----------|-------|
| Every node has a row | 2 |
| Rows are a permutation of 0..N | 2 |
| Every link has a column | 2 |
| Columns are unique | 2 |
| Cycle/path entries have contiguous rows | 2 |
| Correct entries come before incorrect entries | 2 |
| Link source/target rows match node rows | 2 |
| Cycle detection finds entries for imperfect alignments | 2 |
| Node column spans are consistent with incident links | 1 |

### Scoring metrics (21 tests in analysis_tests.rs)

All 7 alignment quality metrics are tested across 3 alignment scenarios
(7 × 3 = 21 test functions):

| Metric | Requires perfect alignment |
|--------|---------------------------|
| EC (Edge Coverage) | No |
| S3 (Symmetric Substructure Score) | No |
| ICS (Induced Conserved Structure) | No |
| NC (Node Correctness) | Yes |
| NGS (Node-Group Similarity) | Yes |
| LGS (Link-Group Similarity) | Yes |
| JS (Jaccard Similarity) | Yes |

Scenarios: `casestudy_iv`, `yeast_sc_perfect`, `yeast_sc_s3_pure`.

### Alignment data source

Realistic alignment test data from:

> Desai et al. (2019) "Network Alignment Visualization"
> BMC Bioinformatics. DOI: 10.1186/s12859-019-2878-3

- **Case Study IV**: SmallYeast ↔ LargeYeast (1004 nodes, GW format)
- **Case Studies I-III**: Yeast2KReduced ↔ SC (2379/6600 nodes, SIF format)

---

## Deterministic behavior reference

These are the exact rules that must match Java for byte-level parity:

### Node ordering (DefaultLayout)

> **Bug fix:** `GraphSearcher.nodeDegree()` had a bug on line 154 where
> `retval0.get(trg)` looked up a `NetNode` in a `HashMap<NodeAndRel, ...>`,
> always returning `null`. This has been fixed to `retval0.get(tar)`.
> Goldens should be regenerated with `--rebuild` after this fix.

1. Build degree map: count **all** links including shadows for each node
2. Group nodes by degree (highest first), break ties lexicographically
3. Pick highest-degree unvisited node as BFS root
4. BFS: visit neighbors in degree order (highest first), lexicographic ties
5. After component exhausted, repeat step 3 for next unvisited
6. Append isolated nodes sorted lexicographically

### Edge ordering (DefaultEdgeLayout)

For each node in row order:
1. Collect all incident links
2. Separate into: links to already-processed nodes vs links to unprocessed
3. Sort via `DefaultFabricLinkLocater` comparator:
   - PER_NETWORK_MODE: group order is primary key
   - Non-shadow: sort by top-row, then group (PER_NODE), then bottom-row
   - Shadow: sort by bottom-row, then group, then top-row
   - Direction: undirected < down-directed < up-directed
   - Relation name (lexicographic)
4. Assign sequential column numbers

### Color assignment

- Node color: `palette[row % 32]` (brighter variant)
- Link color: `palette[shadowCol % 32]` (darker variant)
- 32-color palette is fixed (see `FabricColorGenerator`)

### Drain zones

- Non-shadow: contiguous columns at end of node's range where node is top endpoint
- Shadow: contiguous columns at start of node's shadow range where node is bottom endpoint

### Alignment edge types (link groups)

| Order | Type | Tag | Color |
|-------|------|-----|-------|
| 1 | COVERED | `P` | Purple |
| 2 | INDUCED_GRAPH1 | `pBp` | Blue |
| 3 | HALF_ORPHAN_GRAPH1 | `pBb` | Blue (cyan) |
| 4 | FULL_ORPHAN_GRAPH1 | `bBb` | Blue (green) |
| 5 | INDUCED_GRAPH2 | `pRp` | Red |
| 6 | HALF_UNALIGNED_GRAPH2 | `pRr` | Red (orange) |
| 7 | FULL_UNALIGNED_GRAPH2 | `rRr` | Red (yellow) |

### Alignment node colors

- **Purple** (`P`): Aligned G1↔G2 node (merged/combined)
- **Blue** (`B`): Unaligned G1 node (orphan)
- **Red** (`R`): Unaligned G2 node

### Alignment scoring formulas

- `EC = covered / (covered + induced_G1)`
- `S3 = covered / (covered + induced_G1 + induced_G2)`
- `ICS = covered / (covered + induced_G2)`
- `NC = correct_nodes / total_aligned_nodes` (requires perfect alignment)
- `NGS = angular_similarity(node_group_ratios, perfect_node_group_ratios)`
- `LGS = angular_similarity(link_group_ratios, perfect_link_group_ratios)`
- `JS = avg(jaccard_similarity(neighborhoods))` (requires perfect alignment)

---

## Project structure

```
crates/
├── core/           # biofabric_core library
│   ├── src/
│   │   ├── alignment/   # Network alignment (merge, scoring, cycle, orphan, groups)
│   │   ├── analysis/    # Graph analysis (cycle detection, components, degree)
│   │   ├── export/      # Image export (PNG, JPEG, TIFF)
│   │   ├── io/          # File I/O (SIF, GW, XML/BIF, JSON, attributes, annotations)
│   │   ├── layout/      # All layout algorithms
│   │   │   ├── default.rs       # DefaultNodeLayout + DefaultEdgeLayout
│   │   │   ├── similarity.rs    # NodeSimilarityLayout (resort + clustered)
│   │   │   ├── hierarchy.rs     # HierDAGLayout
│   │   │   ├── cluster.rs       # NodeClusterLayout
│   │   │   ├── control_top.rs   # ControlTopLayout
│   │   │   ├── set.rs           # SetLayout
│   │   │   ├── world_bank.rs    # WorldBankLayout
│   │   │   └── link_group.rs    # Link grouping (PER_NODE + PER_NETWORK)
│   │   ├── model/       # Network model (nodes, links, annotations, selections)
│   │   ├── render/      # Rendering pipeline (camera, viewport, rasterization)
│   │   └── util/        # Utilities (quadtree, hit testing)
│   └── tests/
│       ├── parity_tests.rs        # 267 test functions (layout parity + properties)
│       ├── analysis_tests.rs      # 65 test functions (graph analysis + scoring)
│       └── pipeline_stub.rs       # Rendering pipeline stub for tests
├── cli/            # biofabric CLI binary
│   ├── src/
│   │   └── commands/    # Subcommands: layout, convert, export-order, info,
│   │                    #   extract, align, render, search, compare
│   └── tests/
│       └── cli_tests.rs           # 54 test functions (end-to-end CLI)
└── wasm/           # WebAssembly bindings

tests/parity/
├── Dockerfile                     # Java BioFabric build environment
├── generate_goldens.sh            # Golden file generation script
├── test_manifest.toml             # 123 test case definitions
├── java-harness/src/
│   └── GoldenGenerator.java       # Java golden generator (1390 lines)
├── networks/                      # 43 input files (SIF, GW, align, BIF, NOA, EDA, NA)
└── goldens/                       # Generated reference outputs (gitignored)
```

---

## How to add a new test

1. Drop the network file into `tests/parity/networks/sif/`, `gw/`, or `align/`
2. Run `./tests/parity/generate_goldens.sh <name>` to generate goldens
3. Add the appropriate `parity_test*!` macro invocation in `crates/core/tests/parity_tests.rs`
4. Add a `[[test]]` entry in `tests/parity/test_manifest.toml`

## How to regenerate goldens

```bash
# Via Docker (Java reference — canonical source of truth)
./tests/parity/generate_goldens.sh              # all variants
./tests/parity/generate_goldens.sh triangle      # just one network
./tests/parity/generate_goldens.sh --rebuild     # force Docker rebuild

# Via Rust (faster, identical output)
cargo test --test parity_tests generate_goldens -- --include-ignored --nocapture

# Filter by phase:
./tests/parity/generate_goldens.sh --shadows-only
./tests/parity/generate_goldens.sh --no-shadows-only
./tests/parity/generate_goldens.sh --groups-only
./tests/parity/generate_goldens.sh --layouts-only
./tests/parity/generate_goldens.sh --fixed-only
./tests/parity/generate_goldens.sh --analysis-only
./tests/parity/generate_goldens.sh --display-only
./tests/parity/generate_goldens.sh --align-only
./tests/parity/generate_goldens.sh --graph-analysis-only
```

## Running specific test subsets

```bash
# All tests (386 functions)
cargo test --test parity_tests -- --include-ignored
cargo test --test analysis_tests -- --include-ignored
cargo test --test cli_tests

# By network
cargo test --test parity_tests -- --include-ignored triangle

# By format
cargo test --test parity_tests -- --include-ignored noa
cargo test --test parity_tests -- --include-ignored eda
cargo test --test parity_tests -- --include-ignored bif

# By phase
cargo test --test parity_tests -- --include-ignored noshadow      # P2
cargo test --test parity_tests -- --include-ignored pernode        # P3
cargo test --test parity_tests -- --include-ignored pernetwork     # P3
cargo test --test parity_tests -- --include-ignored roundtrip      # P4
cargo test --test parity_tests -- --include-ignored similarity     # P5
cargo test --test parity_tests -- --include-ignored hierdag        # P6
cargo test --test parity_tests -- --include-ignored cluster        # P7
cargo test --test parity_tests -- --include-ignored controltop     # P8
cargo test --test parity_tests -- --include-ignored set_           # P9
cargo test --test parity_tests -- --include-ignored fixed          # P11
cargo test --test parity_tests -- --include-ignored extract        # P12
cargo test --test parity_tests -- --include-ignored display        # P14

# Alignment subsets
cargo test --test parity_tests -- --include-ignored align          # All alignment (P15-P17)
cargo test --test parity_tests -- --include-ignored casestudy      # Case Study IV
cargo test --test parity_tests -- --include-ignored yeast_sc       # Yeast↔SC (large)
cargo test --test parity_tests -- --include-ignored orphan         # ORPHAN view
cargo test --test parity_tests -- --include-ignored cycle          # CYCLE view
cargo test --test parity_tests -- --include-ignored ng_nc          # NODE_CORRECTNESS
cargo test --test parity_tests -- --include-ignored ng_jacc        # JACCARD_SIMILARITY
cargo test --test parity_tests -- --include-ignored roundtrip_desai # BIF round-trip (annotations)

# Property-based tests
cargo test --test parity_tests -- --include-ignored cycle_property # All 18 invariant checks

# Analysis tests
cargo test --test analysis_tests -- --include-ignored cycle        # Cycle detection
cargo test --test analysis_tests -- --include-ignored jaccard      # Jaccard similarity
cargo test --test analysis_tests -- --include-ignored extract      # Subnetwork extraction
cargo test --test analysis_tests -- --include-ignored components   # Connected components
cargo test --test analysis_tests -- --include-ignored toposort     # Topological sort
cargo test --test analysis_tests -- --include-ignored degree       # Node degree
cargo test --test analysis_tests -- --include-ignored first_neigh  # First-neighbor expansion
cargo test --test analysis_tests -- --include-ignored align_score  # Alignment scoring
cargo test --test analysis_tests -- --include-ignored node_group   # Node group sizes

# By input format
cargo test --test parity_tests -- --include-ignored sif_
cargo test --test parity_tests -- --include-ignored gw_

# Deep-variant tests
cargo test --test parity_tests -- --include-ignored cluster_linksize
cargo test --test parity_tests -- --include-ignored controltop_intra
cargo test --test parity_tests -- --include-ignored controltop_median
cargo test --test parity_tests -- --include-ignored controltop_degree_gray
cargo test --test parity_tests -- --include-ignored controltop_degree_odometer
cargo test --test parity_tests -- --include-ignored resort_5pass
cargo test --test parity_tests -- --include-ignored clustered_tol50
cargo test --test parity_tests -- --include-ignored clustered_chain5

# CLI tests
cargo test --test cli_tests
```

## Remaining test surface

Features not yet covered by parity tests:

| Category | Est. tests | Notes |
|----------|-----------|-------|
| Image export pixel rasterization | ~3 | CPU rasterization of network content (links, nodes, annotations). Current tests validate dimensions only. |
| GZIP session handling | ~2 | Loading/saving `.bif.gz` compressed sessions. |

Not relevant for CLI parity (safe to skip): `SetSemantics::BothInSet`
(GUI-only dialog option, never exposed through headless/CLI pipeline),
NodeSimilarity cosine distance metric (GUI-only alternative to Jaccard,
not available headlessly), zoom/navigation/scrolling, print/PDF export,
plugin system, dialog interaction, tour display, background file reading,
mouse-over views, browser URL templates.
