# BioFabric-rs Parity Test Plan

Exact byte-level parity between `biofabric-rs` and the original Java BioFabric.

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

**Golden generation**: `./tests/parity/generate_goldens.sh` builds BioFabric
and the AlignmentPlugin from source inside Docker (Eclipse Temurin JDK 11)
and runs each test case through the layout pipeline. Outputs are `.noa`
(node order), `.eda` (link/edge order), `.bif` (full session XML), and
`.scores` (alignment scoring metrics, when applicable).

**Rust tests**: `cargo test --test parity_tests --test analysis_tests -- --include-ignored`

## Output formats being compared

| File | Format | What it validates |
|------|--------|-------------------|
| `output.noa` | `Node Row\nname = row\n...` | Node ordering algorithm |
| `output.eda` | `Link Column\nname (rel) name = col\n...` | Edge layout, shadow link placement |
| `output.bif` | Full XML session | Everything: colors, drain zones, columns, annotations, plugin data |
| `output.scores` | `metric\tvalue` (tab-separated) | Alignment scoring metrics (EC, S3, ICS, NC, NGS, LGS, JS) |

## Test suite overview (381 test functions)

| Phase | What | Tests | Functions |
|-------|------|-------|-----------|
| P1 | Default layout, shadows ON | 17 | 51 |
| P2 | Default layout, shadows OFF | 17 | 51 |
| P3 | Link grouping (PER_NODE + PER_NETWORK) | 4 | 12 |
| P4 | XML round-trip | 10 | 10 |
| P5 | NodeSimilarity layout (resort + clustered) | 8 | 24 |
| P6 | HierDAG layout (pointUp true/false) | 6 | 18 |
| P7 | NodeCluster layout | 1 | 3 |
| P8 | ControlTop layout (4 mode combos) | 8 | 24 |
| P9 | SetLayout (BELONGS_TO + CONTAINS) | 4 | 12 |
| P10 | WorldBank layout | 2 | 6 |
| P11 | Fixed-order import (NOA/EDA input) | 4 | 12 |
| P12 | Subnetwork extraction | 3 | 9 |
| P13 | Analysis: cycle detection + Jaccard | 10 | 10 |
| P14 | Display option permutations | 8 | 24 |
| P15 | Alignment — toy networks (GROUP view) | 2 | 6 |
| P16 | Alignment — realistic PPI (GROUP view) | 5 | 15 |
| P17 | Alignment view variants (ORPHAN, CYCLE, NG modes) | 12 | 36 |
| — | Analysis: cycle detection | — | 7 |
| — | Analysis: Jaccard similarity | — | 5 |
| — | Analysis: subnetwork extraction | — | 3 |
| — | Analysis: connected components | — | 11 |
| — | Analysis: topological sort | — | 3 |
| — | Analysis: node degree | — | 9 |
| — | Analysis: first-neighbor expansion | — | 7 |
| — | Analysis: alignment scoring (all 7 metrics × 5 scenarios) | — | 35 |
| **Total** | | **121 manifest + 80 analysis** | **301 parity + 80 analysis = 381** |

Each parity test generates **three** independent test functions (NOA, EDA,
BIF) so agents can make incremental progress — get parsing right first (NOA
passes), then edge layout (EDA passes), then full XML export (BIF passes).

## Test input files (39 files)

### SIF networks (20 files)

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
| `star-500.sif` | 501 | 500 | Hub stress test |
| `ba2K.sif` | 2000 | ~12K | Power-law stress test |
| `BINDhuman.sif` | 19333 | ~39K | Large-scale stress test |
| `Yeast2KReduced.sif` | ~2.4K | ~16K | Alignment source (DesaiEtAl-2019) |
| `SC.sif` | ~6.6K | ~77K | Alignment target (DesaiEtAl-2019) |

### GW networks (7 files)

| File | Nodes | Edges | Purpose |
|------|-------|-------|---------|
| `triangle.gw` | 3 | 3 | GW parser, default relation |
| `directed_triangle.gw` | 3 | 3 | Directed GW, named relations |
| `RNorvegicus.gw` | ~1.5K | ~1.5K | Small real PPI (SANA) |
| `CElegans.gw` | ~3K | ~5K | Medium real PPI (SANA) |
| `yeast.gw` | ~2.4K | ~16K | Large real PPI (SANA) |
| `CaseStudy-IV-SmallYeast.gw` | 1004 | ~8.3K | Alignment source (DesaiEtAl-2019) |
| `CaseStudy-IV-LargeYeast.gw` | 1004 | ~10K | Alignment target (DesaiEtAl-2019) |

### Alignment files (7 files)

| File | Mappings | Description |
|------|----------|-------------|
| `test_perfect.align` | 3 | Toy: full 1:1 mapping |
| `test_partial.align` | 2 | Toy: incomplete mapping |
| `casestudy_iv.align` | 1004 | Case Study IV: SmallYeast ↔ LargeYeast |
| `yeast_sc_perfect.align` | 2379 | Known ground-truth alignment |
| `yeast_sc_s3_pure.align` | 2379 | Pure structural objective (S3=1.0) |
| `yeast_sc_importance_pure.align` | 2379 | Pure biological objective (importance=1.0) |
| `yeast_sc_s3_050.align` | 2379 | Blended (S3=0.05, importance=0.95) |

### Other input files (5 files)

| File | Format | Purpose |
|------|--------|---------|
| `multi_relation_clusters.na` | Node attribute | Cluster assignments for NodeClusterLayout |
| `triangle_reversed.noa` | Node order | Fixed node ordering (reversed) |
| `multi_relation_shuffled.noa` | Node order | Fixed node ordering (shuffled) |
| `ba2K_reversed.noa` | Node order | Fixed node ordering (reversed, large) |
| `triangle_reversed.eda` | Edge order | Fixed edge ordering (reversed) |

---

## Alignment coverage detail

The AlignmentPlugin is tested comprehensively across all 3 view types,
both PerfectNG modes, and the cycle view shadow toggle:

### View types

| ViewType | What it does | Tests |
|----------|-------------|-------|
| `GROUP` | Standard alignment layout with 7 link groups and node color classification | 7 baseline + 4 NG variants |
| `ORPHAN` | Filters to unaligned edges + 1-hop context (OrphanEdgeLayout) | 3 tests |
| `CYCLE` | Highlights mis-alignments via cycle/path detection (AlignCycleLayout) | 3 tests (incl. shadows OFF) |

### PerfectNG modes

| Mode | Node groups | Tests |
|------|-------------|-------|
| `NONE` | 40 standard groups | All baseline GROUP tests |
| `NODE_CORRECTNESS` | 76 groups (split by correctness) | 4 tests |
| `JACCARD_SIMILARITY` | 76 groups (split by JS threshold) | 1 test (threshold=0.75) |

### Scoring metrics (35 tests)

All 7 alignment quality metrics are tested across 5 alignment scenarios
(7 × 5 = 35 test functions in `analysis_tests.rs`):

| Metric | Requires perfect alignment |
|--------|---------------------------|
| EC (Edge Coverage) | No |
| S3 (Symmetric Substructure Score) | No |
| ICS (Induced Conserved Structure) | No |
| NC (Node Correctness) | Yes |
| NGS (Node-Group Similarity) | Yes |
| LGS (Link-Group Similarity) | Yes |
| JS (Jaccard Similarity) | Yes |

Scenarios: `casestudy_iv`, `yeast_sc_perfect`, `yeast_sc_s3_pure`,
`yeast_sc_importance_pure`, `yeast_sc_s3_050`. Each produces an
`output.scores` file (tab-separated `metric\tvalue` lines).

The 4 Yeast↔SC variants use the **same two networks** with **different
alignment files** + `--perfect-align` reference, testing how alignment
quality affects all 7 metrics and the merged network layout.

### Alignment data source

Realistic alignment test data from:

> Desai et al. (2019) "Network Alignment Visualization"
> BMC Bioinformatics. DOI: 10.1186/s12859-019-2878-3

- **Case Study IV**: SmallYeast ↔ LargeYeast (1004 nodes, GW format)
- **Case Studies I-III**: Yeast2KReduced ↔ SC (2379/6600 nodes, SIF format)
  with 4 alignment quality variants (perfect, pure S3, pure importance, blend)

---

## Deterministic behavior reference

These are the exact rules that must match Java for byte-level parity:

### Node ordering (DefaultLayout)

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

## Remaining test surface

Features not yet covered by parity tests (~32 tests / ~68 functions):

| Category | Est. tests | Notes |
|----------|-----------|-------|
| Image export (PNG/JPEG/TIFF) | ~6 | Pixel-level comparison; Java AWT may produce platform-dependent output. Needs tolerance-based comparison. |
| Alignment sub-features | ~4 | Cycle detection in `.align` file mappings; orphan edge filtering as a standalone analysis op. |
| DefaultLayout custom start nodes | ~3 | `DefaultParams.startNodes` override; needs `--start-nodes` flag in GoldenGenerator. |
| Algorithm parameter deep-variants | ~15 | ControlTop: `CTRL_INTRA_DEGREE_ONLY`, `CTRL_MEDIAN_TARGET_DEGREE`, `GRAY_CODE`, `NODE_DEGREE_ODOMETER_SOURCE`. NodeCluster: ordering (`LINK_SIZE`/`NODE_SIZE`/`NAME`), placement (`INLINE`/`BETWEEN`). NodeSimilarity: custom pass count, cosine distance, chain length, tolerance. SetLayout: `BOTH_IN_SET`. |
| Populated annotations | ~2 | All goldens produce empty `<nodeAnnotations>`/`<linkAnnotations>`. Need a BIF with manual annotations. |
| GZIP session handling | ~2 | Loading/saving `.bif.gz` compressed sessions. |

Not relevant for CLI parity (safe to skip): zoom/navigation/scrolling,
print/PDF export, plugin system, dialog interaction, tour display,
background file reading, mouse-over views, browser URL templates.

---

## How to add a new test

1. Drop the network file into `tests/parity/networks/sif/`, `gw/`, or `align/`
2. Run `./tests/parity/generate_goldens.sh <name>` to generate goldens
3. Add the appropriate `parity_test*!` entry in `crates/core/tests/parity_tests.rs`
4. Add a `[[test]]` entry in `tests/parity/test_manifest.toml`

## How to regenerate goldens

```bash
./tests/parity/generate_goldens.sh              # all variants
./tests/parity/generate_goldens.sh triangle      # just one network
./tests/parity/generate_goldens.sh --rebuild     # force Docker rebuild

# Filter by phase:
./tests/parity/generate_goldens.sh --shadows-only
./tests/parity/generate_goldens.sh --no-shadows-only
./tests/parity/generate_goldens.sh --groups-only
./tests/parity/generate_goldens.sh --layouts-only
./tests/parity/generate_goldens.sh --fixed-only
./tests/parity/generate_goldens.sh --analysis-only
./tests/parity/generate_goldens.sh --display-only
./tests/parity/generate_goldens.sh --align-only
```

## Running specific test subsets

```bash
# All tests (381 functions)
cargo test --test parity_tests -- --include-ignored
cargo test --test analysis_tests -- --include-ignored

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
cargo test --test parity_tests -- --include-ignored world_bank     # P10
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

# Analysis tests
cargo test --test analysis_tests -- --include-ignored cycle        # Cycle detection
cargo test --test analysis_tests -- --include-ignored jaccard      # Jaccard similarity
cargo test --test analysis_tests -- --include-ignored extract      # Subnetwork extraction
cargo test --test analysis_tests -- --include-ignored components   # Connected components
cargo test --test analysis_tests -- --include-ignored toposort     # Topological sort
cargo test --test analysis_tests -- --include-ignored degree       # Node degree
cargo test --test analysis_tests -- --include-ignored first_neigh  # First-neighbor expansion
cargo test --test analysis_tests -- --include-ignored align_score  # Alignment scoring

# By input format
cargo test --test parity_tests -- --include-ignored sif_
cargo test --test parity_tests -- --include-ignored gw_
```

## Implementation priority

| Priority | What | First test to pass |
|----------|------|--------------------|
| P0 | SIF parser | `sif_triangle_noa` |
| P0 | GW parser | `gw_triangle_noa` |
| P1 | DefaultNodeLayout (BFS) | `sif_triangle_noa` parity |
| P1 | DefaultEdgeLayout (greedy) | `sif_triangle_eda` parity |
| P2 | NOA/EDA export format | `sif_triangle_noa` + `sif_triangle_eda` exact match |
| P3 | XML session writer + colors + drain zones | `sif_triangle_bif` parity |
| P4 | Shadow toggle (SHADOW_LINK_CHANGE) | `sif_triangle_noshadow_eda` |
| P5 | Link grouping | `sif_multi_relation_pernode_eda` |
| P6 | XML session reader (round-trip) | `roundtrip_sif_triangle_bif` |
| P7 | NodeSimilarityLayout | `sif_triangle_similarity_resort_noa` |
| P8 | HierDAGLayout | `sif_dag_simple_hierdag_true_noa` |
| P9 | NodeClusterLayout | `sif_multi_relation_node_cluster_noa` |
| P10 | ControlTopLayout | `sif_star500_controltop_degree_target_noa` |
| P11 | SetLayout | `sif_bipartite_set_belongs_to_noa` |
| P12 | WorldBankLayout | `sif_star500_world_bank_noa` |
| P13 | Fixed-order import | `sif_triangle_fixed_noa_noa` |
| P14 | Subnetwork extraction | `sif_triangle_extract_AB_noa` |
| P15 | Cycle detection + Jaccard + first-neighbors | `cycle_triangle`, `jaccard_triangle_ab` |
| P16 | Display options | `sif_triangle_drain0_bif` |
| P17 | Alignment merge (GROUP view) | `align_perfect_noa` |
| P18 | Alignment scoring (EC/S3/ICS/NC/NGS/LGS/JS) | `align_score_casestudy_iv_ec` |
| P19 | Alignment ORPHAN view | `align_casestudy_iv_orphan_noa` |
| P20 | Alignment CYCLE view | `align_casestudy_iv_cycle_noa` |
| P21 | Alignment PerfectNG modes | `align_yeast_sc_perfect_ng_nc_noa` |
| P22 | Realistic alignment at scale | `align_yeast_sc_perfect_noa` |
