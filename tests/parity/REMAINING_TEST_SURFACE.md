# Remaining Parity Test Surface

Status of the BioFabric-rs parity test suite against the full Java BioFabric
feature surface.

**Current totals:** 104 manifest entries, 256 parity test functions +
15 analysis test functions = **271 test functions** across 30 input files.

---

## What's COVERED

### Core pipeline (P1-P4) — 48 tests, 124 functions

| Phase | What | Tests | Functions |
|-------|------|-------|-----------|
| P1 | Default layout, shadows ON | 17 | 51 |
| P2 | Default layout, shadows OFF | 17 | 51 |
| P3 | Link grouping (PER_NODE + PER_NETWORK) | 4 | 12 |
| P4 | XML round-trip (load .bif → re-export) | 10 | 10 |

### Layout algorithms (P5-P10) — 29 tests, 87 functions

| Phase | Algorithm | Tests | Networks |
|-------|-----------|-------|----------|
| P5 | NodeSimilarity (resort) | 4 | triangle, multi_relation, ba2K, yeast |
| P5b | NodeSimilarity (clustered) | 4 | triangle, multi_relation, ba2K, yeast |
| P6 | HierDAG (pointUp true/false) | 6 | dag_simple, dag_diamond, dag_deep |
| P7 | NodeCluster | 1 | multi_relation + attribute file |
| P8 | ControlTop (4 mode combos) | 8 | star-500, multi_relation |
| P9 | SetLayout (BELONGS_TO + CONTAINS) | 4 | bipartite, star-500 |
| P10 | WorldBank | 2 | star-500, ba2K |

### Fixed-order import (P11) — 4 tests, 12 functions

| Test | Description |
|------|-------------|
| Fixed NOA × 3 | triangle_reversed, multi_relation_shuffled, ba2K_reversed |
| Fixed EDA × 1 | triangle_reversed |

### Analysis operations (P12-P13) — 13 tests, 24 functions

| Category | Tests | File |
|----------|-------|------|
| Subnetwork extraction | 3 | parity_tests.rs (NOA/EDA/BIF golden comparison) |
| Cycle detection | 7 | analysis_tests.rs (boolean comparison) |
| Jaccard similarity | 5 | analysis_tests.rs (floating-point comparison) |

### Display option permutations (P14) — 8 tests, 24 functions

| Option | Variants |
|--------|----------|
| minDrainZone | 0, 5 (× triangle + multi_relation) |
| nodeLighterLevel | 0.0, 1.0 (× triangle) |
| linkDarkerLevel | 0.0, 1.0 (× triangle) |

### Alignment (P15) — 2 tests, 6 functions

| Test | Description |
|------|-------------|
| align_perfect | 3-node networks with full 1:1 mapping |
| align_partial | 3-node networks with incomplete mapping |

**Note:** Using toy alignment networks. Realistic biological alignments to
be provided later.

---

## What's NOT COVERED

### 1. Image export (PNG/JPEG/TIFF) — ~6 tests

No pixel-level comparison tests. This covers the `render` CLI command
and Java's `ImageExporter` class.

| Format | Notes |
|--------|-------|
| PNG | Lossless, most important |
| JPEG | Lossy — needs tolerance-based comparison |
| TIFF | Lossless |

**Challenge:** Java AWT rendering may produce platform-dependent pixels.
May need tolerance-based comparison rather than exact byte parity.
Golden generation via `ImageGeneratorApplication` is partially supported
already (the `-pngExport` flag exists in Java).

**Needed:** Extend GoldenGenerator or use `ImageGeneratorApplication`
directly. Add resolution/dimension parameters. Compare with pixel tolerance.

---

### 2. Alignment scoring metrics — ~7 tests

The 2 alignment tests compare merged network NOA/EDA/BIF output but don't
validate the 7 individual quality scores computed by the alignment module:

| Score | Full Name |
|-------|-----------|
| EC | Edge Correctness |
| S3 | Symmetric Substructure Score |
| ICS | Induced Conserved Structure |
| NC | Node Correctness |
| NGS | Node-Group Score |
| LGS | Link-Group Score |
| JS | Jaccard Similarity |

**Needed:** Extend GoldenGenerator to output scoring results (text/JSON).
Add analysis tests that compare numeric values.

---

### 3. Alignment sub-features — ~4 tests

| Feature | Description |
|---------|-------------|
| Cycle detection in alignment mapping | Detect A→B→C→A cycles in the `.align` file |
| Orphan edge filtering | Remove edges where one endpoint has no alignment match |

**Needed:** Additional alignment test scenarios + analysis tests.

---

### 4. First-neighbor expansion — ~3 tests

`getFirstNeighbors()` / `AddFirstNeighborsAction`. Takes a node, returns
its 1-hop neighborhood. Used for growing selections before subnetwork
extraction.

**Needed:** Analysis tests that verify neighbor sets match Java's output.

---

### 5. DefaultLayout with custom start nodes — ~3 tests

`DefaultLayout` accepts `DefaultParams.startNodes` to override the
"start from highest-degree node" heuristic. Configured via
`BreadthFirstLayoutDialog` in the UI.

**Needed:** Extend GoldenGenerator with `--start-nodes Node1,Node2,...`
flag. Add test entries for triangle and multi_relation with non-default
start nodes.

---

### 6. Algorithm parameter deep-variants — ~15 tests

Each layout algorithm has parameter sub-options not tested beyond defaults:

| Algorithm | Untested options |
|-----------|-----------------|
| ControlTop | 3 more control modes (`CTRL_PARTIAL_ORDER`, `CTRL_INTRA_DEGREE_ONLY`, `CTRL_MEDIAN_TARGET_DEGREE`); 2 more target modes (`GRAY_CODE`, `NODE_DEGREE_ODOMETER_SOURCE`) |
| NodeCluster | Cluster ordering (`LINK_SIZE`, `NODE_SIZE`, `NAME`); inter-link placement (`INLINE` vs `BETWEEN`); cluster layout (`BREADTH_CONN_FIRST` vs `BREADTH`) |
| NodeSimilarity | Resort: custom pass count, terminate-at-increase; Clustered: Cosine distance, chain length, tolerance |
| SetLayout | `BOTH_IN_SET` mode (third LinkMeans, in addition to `BELONGS_TO` and `CONTAINS` already tested) |

**Needed:** Extend GoldenGenerator with per-algorithm parameter flags.
These are real code paths in Java but low-probability configurations.

---

### 7. Populated annotations — ~2 tests

All 104 tests produce empty `<nodeAnnotations>` / `<linkAnnotations>` in
the BIF. No test verifies that populated annotations round-trip correctly.

**Note:** Link grouping tests with `showLinkGroupAnnotations=true` may
produce populated link annotations. Need to verify whether the existing
P3 or P5+ tests already cover this.

**Needed:** Verify existing goldens; if annotations are always empty, add
a test that loads a BIF with manual annotations.

---

### 8. GZIP session handling — ~2 tests

Loading/saving compressed `.bif.gz` sessions. Minor feature.

**Needed:** Create a gzipped golden BIF, test load → re-export.

---

### 9. Realistic alignment networks — ~4 tests

The current 2 alignment tests use 3-node toy networks. Real biological
alignment testing needs:
- Larger paired networks (100+ nodes)
- Partial alignment (many unmatched nodes)
- Multiple connected components
- Alignment scoring on realistic data

**Status:** User will provide these later.

---

## Summary

| Category | Status | Tests | Functions |
|----------|--------|-------|-----------|
| Core pipeline (parse/layout/export) | ✅ Done | 48 | 124 |
| All 7 layout algorithms | ✅ Done | 29 | 87 |
| Fixed-order import | ✅ Done | 4 | 12 |
| Analysis operations | ✅ Done | 15 | 24 |
| Display options | ✅ Done | 8 | 24 |
| Alignment (basic) | ✅ Done | 2 | 6 |
| **Covered total** | | **106** | **271** |
| | | | |
| Image export | ❌ Not started | ~6 | ~6 |
| Alignment scoring | ❌ Not started | ~7 | ~7 |
| Alignment sub-features | ❌ Not started | ~4 | ~4 |
| First-neighbor expansion | ❌ Not started | ~3 | ~3 |
| Custom start nodes | ❌ Not started | ~3 | ~9 |
| Algorithm param variants | ❌ Not started | ~15 | ~45 |
| Populated annotations | ❌ Not started | ~2 | ~2 |
| GZIP sessions | ❌ Not started | ~2 | ~2 |
| Realistic alignments | ⏳ Waiting on user | ~4 | ~12 |
| **Remaining total** | | **~46** | **~90** |
| | | | |
| **Grand total** | | **~152** | **~361** |

### What's not relevant for CLI parity (safe to skip)

- Zoom/navigation/scrolling (UI only)
- Print/PDF export (UI only)
- Plugin system (out of scope)
- Dialog interaction / selection mode (UI only)
- Tour display (UI only)
- Background file reading (performance, not correctness)
- Mouse-over views, browser URL templates (UI only)
