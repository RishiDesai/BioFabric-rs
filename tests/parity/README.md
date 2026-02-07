# BioFabric-rs Parity Test Plan

Exact byte-level parity between `biofabric-rs` and the original Java BioFabric.

## How it works

```
  Input network (.sif/.gw)
         │
         ├──► Java BioFabric (Docker) ──► golden .noa, .eda, .bif
         │
         └──► biofabric-rs (Rust)     ──► output .noa, .eda, .bif
                                                │
                                          byte-for-byte diff
```

**Golden generation**: `./tests/parity/generate_goldens.sh` builds BioFabric
from source inside Docker (Eclipse Temurin JDK 11) and runs each network
through the layout pipeline with various configurations. Outputs are `.noa`
(node order), `.eda` (link/edge order), and `.bif` (full session XML).

**Rust tests**: `cargo test --test parity_tests -- --include-ignored` loads
the same input, runs the same algorithm, and diffs output against goldens.

## Output formats being compared

| File | Format | What it validates |
|------|--------|-------------------|
| `output.noa` | `Node Row\nname = row\n...` | Node ordering algorithm |
| `output.eda` | `Link Column\nname (rel) name = col\n...` | Edge layout algorithm, shadow link placement |
| `output.bif` | Full XML session | Everything: colors, drain zones, column ranges, annotations |

## Test dimensions (124 test functions)

| Phase | What | Config | Tests | Functions |
|-------|------|--------|-------|-----------|
| P1 | Default layout, shadows ON | baseline | 17 | 51 (×3 formats) |
| P2 | Default layout, shadows OFF | `--no-shadows` | 17 | 51 (×3 formats) |
| P3 | Link grouping | `--link-groups` + `--group-mode` | 4 | 12 (×3 formats) |
| P4 | XML round-trip | load `.bif` → re-export | 10 | 10 (BIF only) |
| **Total** | | | **48** | **124** |

Each P1/P2/P3 test generates **three** independent test functions (NOA, EDA,
BIF) so agents can make incremental progress — get parsing right first (NOA
passes), then edge layout (EDA passes), then full XML export (BIF passes).

## Current test networks (21)

### SIF (12 files)

| File | Nodes | Edges | Why it's here |
|------|-------|-------|---------------|
| `single_node.sif` | 1 | 0 | Minimal network — single isolated node, no edges |
| `single_edge.sif` | 2 | 1 | Minimal connected network — 2 nodes, 1 edge |
| `triangle.sif` | 3 | 3 | Simplest cycle; baseline sanity check |
| `self_loop.sif` | 3 | 3 | Feedback edge `A pp A` — no shadow for self-loops |
| `isolated_nodes.sif` | 6 | 2 | Lone nodes X, Y, Z appended after connected nodes |
| `disconnected_components.sif` | 11 | 8 | 4 components — validates component ordering in BFS |
| `multi_relation.sif` | 5 | 6 | 3 relation types (pp, pd, gi) — validates relation-aware edge sorting |
| `dense_clique.sif` | 6 | 15 | Complete graph K6 — all equal degree, pure tie-breaking test |
| `linear_chain.sif` | 10 | 9 | Path graph — many degree-2 nodes, chain BFS ordering |
| `star-500.sif` | 501 | 500 | High-degree hub; stress test for column assignment |
| `ba2K.sif` | 2000 | ~12K | Power-law network; real-world degree distribution |
| `BINDhuman.sif` | 19333 | ~39K | Cytoscape protein interaction network; large-scale stress |

### GW (5 files)

| File | Nodes | Edges | Why it's here |
|------|-------|-------|---------------|
| `triangle.gw` | 3 | 3 | Same topology as triangle.sif; validates GW parser, default relation |
| `directed_triangle.gw` | 3 | 3 | Directed GW with named relations (activates, inhibits) |
| `RNorvegicus.gw` | ~1.5K | ~1.5K | Small real PPI network (SANA) |
| `CElegans.gw` | ~3K | ~5K | Medium real PPI network (SANA) |
| `yeast.gw` | ~2.4K | ~16K | Large real PPI network (SANA) |

---

## What to test: full checklist

### Phase 1: Parsing + Default Layout (shadows ON)

These validate parsing, node ordering, edge layout, and export — the full
default pipeline. Each test checks NOA, EDA, and BIF independently.

#### SIF parser (`io::sif`)

| # | Test case | Edge case | Validated by |
|---|-----------|-----------|-------------|
| 1 | Basic tab-delimited 3-token lines | Happy path | `triangle.sif` |
| 2 | Space-delimited fallback (no tabs) | BINDhuman uses spaces | `BINDhuman.sif` |
| 3 | Self-loop line (`A pp A`) | No shadow created for feedback | `self_loop.sif` |
| 4 | Isolated node (1-token line) | Added to lone nodes set | `isolated_nodes.sif` |
| 5 | Multiple relation types on different edges | Each gets own `AugRelation` | `multi_relation.sif` |
| 6 | Duplicate edges (same src, rel, trg) | Removed during `preprocessLinks()` | `BINDhuman.sif` has dups |
| 7 | Single isolated node (no edges at all) | Entire network is one lone node | `single_node.sif` |
| 8 | Minimal connected (2 nodes, 1 edge) | Simplest non-trivial case | `single_edge.sif` |
| 9 | Dense all-to-all connectivity | Equal degree, pure lexicographic ordering | `dense_clique.sif` |
| 10 | Path graph (all degree 2 except endpoints) | Chain BFS, extensive tie-breaking | `linear_chain.sif` |
| 11 | Shadow link creation for every non-feedback edge | `src != trg` → shadow | All networks |

#### GW parser (`io::gw`)

| # | Test case | Edge case | Validated by |
|---|-----------|-----------|-------------|
| 1 | Undirected GW (`-2` header) | Default undirected | `triangle.gw` |
| 2 | Directed GW (`-1` header) | Direction preserved | `directed_triangle.gw` |
| 3 | Empty edge labels → `"default"` relation | `stripBrackets()` then fallback | `triangle.gw` |
| 4 | Named edge labels in brackets `\|{rel}\|` | Bracket stripping | `directed_triangle.gw` |
| 5 | 1-based node indexing | Parser must subtract/adjust | All GW files |
| 6 | Large GW file (18K lines) | Performance, correctness at scale | `yeast.gw` |

### Phase 2: Default Node Layout

The BFS node ordering algorithm. Tested via `.noa` output.

| # | Test case | Expected behavior | Validated by |
|---|-----------|-------------------|-------------|
| 1 | Start from highest-degree node | Degree = count of all links (including shadows) | `triangle.sif` (all degree 4) |
| 2 | Tie-breaking: lexicographic by node name | Within same degree, `TreeSet<NetNode>` ordering | `triangle.sif` → A, B, C |
| 3 | BFS adds neighbors in degree order | Higher-degree neighbors first | `multi_relation.sif` |
| 4 | Disconnected components: largest first | Component with highest-degree node goes first | `disconnected_components.sif` |
| 5 | Isolated nodes appended at end | Sorted lexicographically after all connected nodes | `isolated_nodes.sif` → X, Y, Z at end |
| 6 | Self-loop doesn't double-count degree | One link, one shadow (none), degree = 3 for A | `self_loop.sif` |
| 7 | Hub node always first | Degree 500 >> all others | `star-500.sif` |
| 8 | Large power-law network | BFS handles many components/nodes | `ba2K.sif` |
| 9 | Single node (no edges) | Just the one lone node | `single_node.sif` |
| 10 | Minimal edge (2 nodes) | Higher-degree node first (tie → lexicographic) | `single_edge.sif` |
| 11 | Dense clique (all same degree) | Pure lexicographic ordering | `dense_clique.sif` |
| 12 | Long chain (degree-2 interior) | Endpoint vs interior tie-breaking | `linear_chain.sif` |

### Phase 3: Default Edge Layout

The greedy column assignment algorithm. Tested via `.eda` output.

| # | Test case | Expected behavior | Validated by |
|---|-----------|-------------------|-------------|
| 1 | Links to already-placed nodes come first | Process nodes in row order, links to prior rows first | `triangle.sif` |
| 2 | Shadow links ordered separately | Shadows sorted by bottom-row (not top-row) | `triangle.sif` `.eda` |
| 3 | Self-loop gets column but no shadow column | `A (pp) A = 0`, no `A shdw(pp) A` line | `self_loop.sif` |
| 4 | Multi-relation: direction→relation→tag ordering | `pd` before `pp` before `gi`? Check actual order | `multi_relation.sif` |
| 5 | Column numbers are sequential (0, 1, 2, ...) | No gaps in column assignment | All networks |
| 6 | Comparator: undirected < down-directed < up-directed | Within same node pair, direction ordering | `multi_relation.sif` |
| 7 | Single edge: one column, one shadow column | Minimal case | `single_edge.sif` |
| 8 | Dense clique: 15 edges + 15 shadows = 30 columns | All edges laid out correctly | `dense_clique.sif` |

### Phase 4: Session XML Export

The `.bif` file. This is the most comprehensive check — if the `.bif`
matches byte-for-byte, everything matches.

| # | Test case | What it validates |
|---|-----------|-------------------|
| 1 | `<colors>` section: 32-color palette for nodes and links | Exact RGB values, exact color names |
| 2 | `<displayOptions />` | Default display options output |
| 3 | `<node>` elements: name, nid, row, minCol, maxCol, minColSha, maxColSha, color | All node layout attributes |
| 4 | `<drainZones>` and `<drainZonesShadow>` | Drain zone computation correctness |
| 5 | `<link>` elements: srcID, trgID, rel, directed, shadow, column, shadowCol, srcRow, trgRow, color | All link layout attributes |
| 6 | Color assignment: `row % 32` for nodes, `shadowCol % 32` for links | Deterministic cycling |
| 7 | Node ID (`nid`) assignment | Sequential integer IDs from `UniqueLabeller` |
| 8 | Empty annotations sections | `<nodeAnnotations>`, `<linkAnnotations>`, `<shadowLinkAnnotations>` present but empty |
| 9 | Empty plugin data | `<plugInDataSets>` present but empty |
| 10 | XML indentation (2-space indent) | Exact whitespace matching |
| 11 | Character entity escaping in relation names | `CharacterEntityMapper.mapEntities()` |
| 12 | Attribute ordering within elements | Must match Java's `PrintWriter` output order exactly |

### Phase 5: NOA/EDA Export Format

| # | Test case | What it validates |
|---|-----------|-------------------|
| 1 | NOA header is exactly `Node Row` | First line format |
| 2 | NOA body: `nodeName = rowNumber` (space-equals-space) | Exact format |
| 3 | NOA nodes ordered by row number (ascending) | `TreeSet` iteration on row keys |
| 4 | EDA header is exactly `Link Column` | First line format |
| 5 | EDA body: `src (rel) trg = col` for non-shadow | `FabricLink.toEOAString()` format |
| 6 | EDA body: `src shdw(rel) trg = col` for shadow | Shadow prefix format |
| 7 | EDA links ordered by column number (ascending) | `TreeMap` iteration on column keys |

### Phase 6: Shadow Link Toggle (P2 tests)

Same network with shadows OFF. Validates the `SHADOW_LINK_CHANGE` rebuild.

| # | Test case | Expected behavior |
|---|-----------|-------------------|
| 1 | Single node, shadows OFF | Trivially identical to shadows ON (no edges) |
| 2 | Single edge, shadows OFF | 1 column instead of 2 |
| 3 | Triangle, shadows OFF | 3 links, no shadow columns |
| 4 | Self-loop, shadows OFF | Self-loop has no shadow regardless |
| 5 | Star-500, shadows OFF | 500 columns instead of ~1000 |
| 6 | BA-2K, shadows OFF | Large-scale column reduction |
| 7 | All GW networks, shadows OFF | GW parser + shadow toggle integration |

### Phase 7: Link Grouping (P3 tests)

Different link grouping modes. Tests `GROUP_PER_NODE_CHANGE` and
`GROUP_PER_NETWORK_CHANGE` rebuild paths.

| # | Test case | Expected behavior |
|---|-----------|-------------------|
| 1 | multi_relation PER_NODE groups=[pp,pd,gi] | Links at each node sorted by group order |
| 2 | multi_relation PER_NETWORK groups=[pp,pd,gi] | Global group ordering as primary sort key |
| 3 | directed_triangle PER_NODE groups=[activates,inhibits] | GW-based grouping |
| 4 | directed_triangle PER_NETWORK groups=[activates,inhibits] | GW + global grouping |

### Phase 8: XML Round-Trip (P4 tests)

Load a golden `.bif` file → re-export → compare against itself.
Tests the `io::xml` read + write pipeline in isolation.

| # | Test case | What it validates |
|---|-----------|-------------------|
| 1 | Small networks (triangle, self_loop, etc.) | Basic XML parsing + serialization |
| 2 | Medium networks (star-500, RNorvegicus) | Scale handling |
| 3 | Large networks (ba2K, CElegans) | Performance + correctness at scale |

### Future Phases

| Phase | What | Status |
|-------|------|--------|
| P9 | Advanced layouts (Similarity, HierDAG, Cluster, ControlTop, Set, WorldBank) | Needs new golden generation + DAG/bipartite networks |
| P10 | Alignment features (merge, scoring, cycle detection) | Depends on alignment module |
| P11 | Image export (PNG parity) | Pixel-level comparison |

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

---

## How to add a new test

1. Drop the network file into `tests/parity/networks/sif/` or `gw/`
2. Run `./tests/parity/generate_goldens.sh <name>` to generate goldens
3. Add a `parity_test!` entry in `crates/core/tests/parity_tests.rs`
4. Add a `[[test]]` entry in `tests/parity/test_manifest.toml`

For shadow-off variants, also add:
- A `parity_test_noshadow!` entry in the test file
- A corresponding `[[test]]` with `shadows = false` in the manifest

## How to regenerate goldens

```bash
./tests/parity/generate_goldens.sh              # all variants (~30s)
./tests/parity/generate_goldens.sh triangle      # just one network (all variants)
./tests/parity/generate_goldens.sh --rebuild     # force Docker rebuild
./tests/parity/generate_goldens.sh --shadows-only    # only shadows-on baseline
./tests/parity/generate_goldens.sh --no-shadows-only # only shadows-off variants
./tests/parity/generate_goldens.sh --groups-only     # only link grouping variants
```

## Running specific test subsets

```bash
# All tests (124 functions)
cargo test --test parity_tests -- --include-ignored

# By network
cargo test --test parity_tests -- --include-ignored triangle

# By format
cargo test --test parity_tests -- --include-ignored noa
cargo test --test parity_tests -- --include-ignored eda
cargo test --test parity_tests -- --include-ignored bif

# By phase
cargo test --test parity_tests -- --include-ignored noshadow    # P2 tests
cargo test --test parity_tests -- --include-ignored pernode      # P3 per-node grouping
cargo test --test parity_tests -- --include-ignored pernetwork   # P3 per-network grouping
cargo test --test parity_tests -- --include-ignored roundtrip    # P4 XML round-trip

# By format (SIF vs GW)
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
| P3 | XML session writer | `sif_triangle_bif` parity |
| P3 | Color generator (32-color palette) | Part of BIF parity |
| P3 | Drain zone computation | Part of BIF parity |
| P4 | Shadow toggle (SHADOW_LINK_CHANGE) | `sif_triangle_noshadow_eda` parity |
| P5 | Link grouping (GROUP_PER_NODE/NETWORK) | `sif_multi_relation_pernode_eda` parity |
| P6 | XML session reader (round-trip) | `roundtrip_sif_triangle_bif` parity |
| P7 | Advanced layouts | Per-algorithm implementation |
| P8 | Alignment features | Depends on alignment module |
