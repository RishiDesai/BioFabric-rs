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
through the default layout pipeline. Outputs are `.noa` (node order), `.eda`
(link/edge order), and `.bif` (full session XML).

**Rust tests**: `cargo test --test parity_tests -- --include-ignored` loads
the same input, runs the same algorithm, and diffs output against goldens.

## Output formats being compared

| File | Format | What it validates |
|------|--------|-------------------|
| `output.noa` | `Node Row\nname = row\n...` | Node ordering algorithm |
| `output.eda` | `Link Column\nname (rel) name = col\n...` | Edge layout algorithm, shadow link placement |
| `output.bif` | Full XML session | Everything: colors, drain zones, column ranges, annotations |

## Current test networks (13)

### SIF (8 files)

| File | Nodes | Edges | Why it's here |
|------|-------|-------|---------------|
| `triangle.sif` | 3 | 3 | Simplest cycle; baseline sanity check |
| `self_loop.sif` | 3 | 3 | Feedback edge `A pp A` — no shadow for self-loops |
| `isolated_nodes.sif` | 6 | 2 | Lone nodes X, Y, Z appended after connected nodes |
| `disconnected_components.sif` | 11 | 8 | 4 components — validates component ordering in BFS |
| `multi_relation.sif` | 5 | 6 | 3 relation types (pp, pd, gi) — validates relation-aware edge sorting |
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

### Phase 1: Parsing

These validate that loading a file produces the exact same internal network
representation (same nodes, same links, same relations, same shadow links).
Checked via `.noa` (correct node set) and `.eda` (correct link set with
correct shadow/non-shadow classification).

#### SIF parser (`io::sif`)

| # | Test case | Edge case | Validated by |
|---|-----------|-----------|-------------|
| 1 | Basic tab-delimited 3-token lines | Happy path | `triangle.sif` |
| 2 | Space-delimited fallback (no tabs) | BINDhuman uses spaces | `BINDhuman.sif` |
| 3 | Self-loop line (`A pp A`) | No shadow created for feedback | `self_loop.sif` |
| 4 | Isolated node (1-token line) | Added to lone nodes set | `isolated_nodes.sif` |
| 5 | Multiple relation types on different edges | Each gets own `AugRelation` | `multi_relation.sif` |
| 6 | Duplicate edges (same src, rel, trg) | Removed during `preprocessLinks()` | `BINDhuman.sif` has dups |
| 7 | Empty/blank lines between entries | Silently skipped | All files (add blank lines to test) |
| 8 | Quoted node names (`"node A"`) | `stripQuotes()` removes outer `"` | **NEEDS NEW NETWORK** |
| 9 | 2-token line (invalid) | Added to `badLines`, skipped | **NEEDS NEW NETWORK** |
| 10 | Shadow link creation for every non-feedback edge | `src != trg` → shadow | All networks |

#### GW parser (`io::gw`)

| # | Test case | Edge case | Validated by |
|---|-----------|-----------|-------------|
| 1 | Undirected GW (`-2` header) | Default undirected | `triangle.gw` |
| 2 | Directed GW (`-1` header) | Direction preserved | `directed_triangle.gw` |
| 3 | Empty edge labels → `"default"` relation | `stripBrackets()` then fallback | `triangle.gw` |
| 4 | Named edge labels in brackets `\|{rel}\|` | Bracket stripping | `directed_triangle.gw` |
| 5 | 1-based node indexing | Parser must subtract/adjust | All GW files |
| 6 | Lone nodes (declared but no edges) | Added to lone nodes set | Check SANA networks |
| 7 | Large GW file (18K lines) | Performance, correctness at scale | `yeast.gw` |

#### XML/BIF parser (`io::xml`) — round-trip

| # | Test case | Validated by |
|---|-----------|-------------|
| 1 | Load a `.bif` → re-export → byte-identical | Use golden `.bif` as input |
| 2 | All XML elements preserved (colors, nodes, links, annotations, groups) | Compare re-exported `.bif` |
| 3 | Character entity mapping (`&`, `<`, `>`, `"`, `'` in names) | **NEEDS NETWORK WITH SPECIAL CHARS** |

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

### Phase 3: Default Edge Layout

The greedy column assignment algorithm. Tested via `.eda` output.

| # | Test case | Expected behavior | Validated by |
|---|-----------|-------------------|-------------|
| 1 | Links to already-placed nodes come first | Process nodes in row order, links to prior rows first | `triangle.sif` |
| 2 | Shadow links ordered separately | Shadows sorted by bottom-row (not top-row) | `triangle.sif` `.eda` |
| 3 | Self-loop gets column but no shadow column | `A (pp) A = 0`, no `A shdw(pp) A` line | `self_loop.sif` |
| 4 | Multi-relation: direction→relation→tag ordering | `pd` before `pp` before `gi`? Check actual order | `multi_relation.sif` |
| 5 | Column numbers are sequential (0, 1, 2, ...) | No gaps in column assignment | All networks |
| 6 | Shadow column is always assigned | Every link has a shadow column (even non-shadows) | Verified in `.bif` XML |
| 7 | Non-shadow column only for non-shadow links | Shadow-only links have no `column` attr | Verified in `.bif` XML |
| 8 | Comparator: undirected < down-directed < up-directed | Within same node pair, direction ordering | `multi_relation.sif` |

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

### Phase 6: Shadow Link Toggle

Same network with shadows ON vs OFF. Requires extending the golden generator
to support a `--no-shadows` flag (sets `FabricDisplayOptions.setDisplayShadows(false)`).

| # | Test case | Expected behavior |
|---|-----------|-------------------|
| 1 | Triangle shadows OFF | `.eda` has 3 lines (no shadow links), `.bif` has different column ranges |
| 2 | Self-loop shadows OFF | Self-loop still has no shadow regardless |
| 3 | Star-500 shadows OFF | 500 columns instead of ~1000 |

**Status**: Not yet implemented — needs golden generator extension.

### Phase 7: Link Grouping

Same network with different link grouping configurations. Requires extending
the golden generator to accept `--link-groups` and `--group-mode` flags.

| # | Test case | Expected behavior |
|---|-----------|-------------------|
| 1 | multi_relation PER_NODE_MODE groups=[pp,pd,gi] | Links at each node sorted by group order |
| 2 | multi_relation PER_NETWORK_MODE groups=[pp,pd,gi] | Global group ordering as primary sort key |
| 3 | Link group annotations generated | `<linkAnnotations>` section populated |

**Status**: Not yet implemented — needs golden generator extension.

### Phase 8: Advanced Layouts

Each layout algorithm gets tested when implemented. Requires extending the
golden generator to accept `--layout` flags and set up the appropriate
`BuildData`.

| Layout | Networks to test | Notes |
|--------|-----------------|-------|
| `NodeSimilarityLayout` | triangle, ba2K | Jaccard similarity ordering |
| `HierDAGLayout` | **needs DAG network** | Requires DAG structure |
| `NodeClusterLayout` | **needs cluster file** | Requires attribute file input |
| `ControlTopLayout` | star-500 | 5 control ordering modes × 4 target modes |
| `SetLayout` | **needs bipartite network** | Requires bipartite structure |
| `WorldBankLayout` | star-500 | Hub-spoke grouping |

**Status**: Not yet implemented — advanced layouts are future work.

### Phase 9: Network Alignment

Alignment-specific features. Requires alignment plugin and `.align` files.

| # | Test case |
|---|-----------|
| 1 | Merge two networks with alignment mapping |
| 2 | Node coloring (purple/blue/red classification) |
| 3 | Edge type classification (7 types) |
| 4 | Alignment scores (EC, S3, ICS, NC, NGS, LGS, JS) |
| 5 | Cycle detection |
| 6 | Orphan edge filtering |

**Status**: Future work — depends on alignment module implementation.

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

## How to regenerate goldens

```bash
./tests/parity/generate_goldens.sh              # all networks (~10s)
./tests/parity/generate_goldens.sh triangle      # just one
./tests/parity/generate_goldens.sh --rebuild     # force Docker rebuild
```

## Implementation priority

| Priority | What | First test to pass |
|----------|------|--------------------|
| P0 | SIF parser | `sif_triangle` NOA check |
| P0 | GW parser | `gw_triangle` NOA check |
| P1 | DefaultNodeLayout (BFS) | `sif_triangle` NOA parity |
| P1 | DefaultEdgeLayout (greedy) | `sif_triangle` EDA parity |
| P2 | NOA/EDA export format | `sif_triangle` NOA + EDA exact match |
| P3 | XML session writer | `sif_triangle` BIF parity |
| P3 | Color generator (32-color palette) | Part of BIF parity |
| P3 | Drain zone computation | Part of BIF parity |
| P4 | XML session reader (round-trip) | Load `.bif` → re-export → identical |
| P5 | Shadow toggle | Needs golden generator extension |
| P5 | Link grouping | Needs golden generator extension |
| P6 | Advanced layouts | Per-algorithm implementation |
| P7 | Alignment features | Depends on alignment module |
