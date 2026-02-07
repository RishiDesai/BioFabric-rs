# Remaining Parity Test Surface

Everything the Java BioFabric can do that is **not yet covered** by the
current 124-test parity suite. Organized by priority tier for CLI parity.

Current coverage: default pipeline (SIF/GW → DefaultLayout → NOA/EDA/BIF),
shadow toggle, link grouping, XML round-trip.

---

## Tier 1: Other Layout Algorithms (HIGH priority, ~60 tests)

The biggest gap. Java BioFabric has **7 layout algorithms**; we only test
`DefaultLayout`. Each produces a different node ordering and edge layout,
all of which must match byte-for-byte.

### 1a. NodeSimilarityLayout (`NodeSimilarityLayout.java`)

Reorders nodes by Jaccard similarity of their neighborhoods. Two modes:

| Mode | Java BuildMode | Description |
|------|---------------|-------------|
| Resort / Reorder | `REORDER_LAYOUT` | Shape matching via connection vectors. Iterative refinement. |
| Clustered | `CLUSTERED_LAYOUT` | Attribute-based clustering using similarity matrix. |

**GoldenGenerator changes needed:**
- Add `--layout similarity_resort` and `--layout similarity_clustered` flags
- Both modes require `ResortParams` — need to decide on parameter values or use defaults

**Networks to test:** `triangle.sif`, `multi_relation.sif`, `ba2K.sif`, `yeast.gw`

**Estimated tests:** 4 networks × 2 modes × 3 formats = **24 test functions**

---

### 1b. HierDAGLayout (`HierDAGLayout.java`)

Assigns nodes to hierarchical levels in a DAG (directed acyclic graph).
Has a `pointUp` parameter (top-down vs bottom-up orientation).

**GoldenGenerator changes needed:**
- Add `--layout hierdag` flag
- Add `--point-up true|false` flag

**New input files needed:**
- `dag_simple.sif` — small hand-crafted DAG (5-6 nodes)
- `dag_diamond.sif` — diamond-shaped DAG (4 nodes, converging paths)
- `dag_deep.sif` — deep linear DAG (tests many levels)

**Why new files:** HierDAG has a `criteriaMet()` check that rejects networks
with cycles. None of the current SIF test files are cycle-free DAGs
(triangle, clique, chain all have cycles or are undirected).

Example `dag_simple.sif`:
```
Root	pd	Child1
Root	pd	Child2
Child1	pd	Leaf1
Child1	pd	Leaf2
Child2	pd	Leaf3
```

**Estimated tests:** 3 DAG networks × 2 pointUp values × 3 formats = **18 test functions**

---

### 1c. NodeClusterLayout (`NodeClusterLayout.java`)

Clusters nodes by attributes read from a `.na` (node attribute) file.

**GoldenGenerator changes needed:**
- Add `--layout node_cluster` flag
- Add `--attribute-file <path>` flag to pass the `.na` file

**New input files needed:**
- Attribute files (`.na`) for existing networks that assign cluster labels
- Example `multi_relation_clusters.na`:
  ```
  ClusterAssignment
  A = group1
  B = group1
  C = group2
  D = group2
  E = group3
  ```

**Networks to test:** `multi_relation.sif` + attribute file, `ba2K.sif` + attribute file

**Estimated tests:** 2 networks × 3 formats = **6 test functions**

---

### 1d. ControlTopLayout (`ControlTopLayout.java`)

Pins "control" nodes at the top of the layout. Five control selection modes
and four target modes:

| Control Mode | Description |
|-------------|-------------|
| `FIXED_LIST` | User-specified node list |
| `HIGHEST_DEGREE` | Top-N highest degree nodes |
| `SELECTED` | Currently selected nodes (UI-driven) |

| Target Mode | Description |
|-------------|-------------|
| `ALL` | All other nodes below controls |
| `NEIGHBORS_ONLY` | Only neighbors of control nodes |

**GoldenGenerator changes needed:**
- Add `--layout control_top` flag
- Add `--control-nodes Node1,Node2,...` flag
- Add `--control-mode fixed_list|highest_degree` flag
- Add `--target-mode all|neighbors_only` flag

**Networks to test:** `star-500.sif` (hub as control), `multi_relation.sif`

**Estimated tests:** 2 networks × 2 control modes × 2 target modes × 3 formats = **24 test functions**

(Can reduce by picking the most important combinations.)

---

### 1e. SetLayout (`SetLayout.java`)

Bipartite / set membership layout. Requires specifying which nodes are
"in the set" and how links relate to set membership.

Three `LinkMeans` semantics:
- `SOURCE_IN_SET` — source node of each link is in the set
- `TARGET_IN_SET` — target node is in the set
- `BOTH_IN_SET` — both endpoints are in the set

**GoldenGenerator changes needed:**
- Add `--layout set` flag
- Add `--set-nodes Node1,Node2,...` flag
- Add `--link-means source_in_set|target_in_set|both_in_set` flag

**New input files needed:**
- `bipartite.sif` — a proper bipartite graph, e.g.:
  ```
  L1	pp	R1
  L1	pp	R2
  L2	pp	R1
  L2	pp	R3
  L3	pp	R2
  L3	pp	R3
  ```

**Networks to test:** `bipartite.sif`, `star-500.sif` (hub as one partition)

**Estimated tests:** 2 networks × 3 link-means modes × 3 formats = **18 test functions**

---

### 1f. WorldBankLayout (`WorldBankLayout.java`)

Hub-spoke grouping pattern. Groups nodes around their highest-connectivity
hub. No special input requirements.

**GoldenGenerator changes needed:**
- Add `--layout world_bank` flag

**Networks to test:** `star-500.sif`, `ba2K.sif`

**Estimated tests:** 2 networks × 3 formats = **6 test functions**

---

### Summary: Layout Algorithm Tests

| Algorithm | Networks | Variants | Test Functions |
|-----------|----------|----------|----------------|
| NodeSimilarity (resort) | 4 | 1 | 12 |
| NodeSimilarity (clustered) | 4 | 1 | 12 |
| HierDAG | 3 | 2 (pointUp) | 18 |
| NodeCluster | 2 | 1 | 6 |
| ControlTop | 2 | 4 (mode combos) | 24 |
| SetLayout | 2 | 3 (link-means) | 18 |
| WorldBank | 2 | 1 | 6 |
| **Total** | | | **~96** |

---

## Tier 2: Fixed-Order Import (HIGH priority, ~12 tests)

Java BioFabric can load `.noa` and `.eda` files to impose a **fixed** node
or edge ordering, bypassing the layout algorithm. This tests the
`NODE_ATTRIB_LAYOUT` and `LINK_ATTRIB_LAYOUT` build modes.

### 2a. Fixed node order from NOA file

Load a `.noa` file → apply as fixed node ordering → recompute edge layout.

**How it works in Java:**
- Parse NOA: `nodeName = rowNumber` lines
- Use the specified row order instead of BFS
- Run edge layout normally against the fixed node order

**GoldenGenerator changes needed:**
- Add `--layout fixed_node_order` flag
- Add `--noa-file <path>` flag
- Generator loads the network, applies the fixed node order, runs edge layout

**Test cases:**
1. Load `triangle.sif` + a hand-crafted NOA that reverses the default order → different EDA, different BIF
2. Load `multi_relation.sif` + shuffled NOA
3. Load `ba2K.sif` + custom NOA (reverse alphabetical)

**Estimated tests:** 3 networks × 3 formats = **9 test functions**

### 2b. Fixed link order from EDA file

Load an `.eda` file → apply as fixed edge ordering → preserve node order.

**Test cases:**
1. Load `triangle.sif` + a hand-crafted EDA that reverses column order

**Estimated tests:** 1 network × 3 formats = **3 test functions**

---

## Tier 3: Network Analysis Operations (MEDIUM priority, ~18 tests)

These are query/analysis operations that produce structured output.
For parity, the Rust CLI should produce identical text output.

### 3a. Cycle Detection (`CycleFinder.java`)

DFS-based cycle detection. Returns boolean `hasACycle()`.

**Test approach:** Run cycle detection on various networks, compare output
(true/false) against Java reference.

| Network | Has Cycle? |
|---------|-----------|
| `triangle.sif` | Yes |
| `self_loop.sif` | Yes |
| `linear_chain.sif` | No (path graph, no cycles in undirected sense — depends on interpretation) |
| `dag_simple.sif` | No (new DAG file needed anyway for HierDAG tests) |
| `disconnected_components.sif` | Depends on component structure |

**Test format:** Text output comparison (simpler than BIF, just a boolean or JSON)

**Estimated tests:** 5 networks = **5 test functions**

### 3b. Node Comparison / Jaccard Similarity

Compare neighborhoods of two nodes using Jaccard similarity.

**Test approach:** For each network, pick a pair of nodes, compute Jaccard,
compare floating-point result against Java reference.

**Estimated tests:** 3 networks × 1-2 pairs = **5 test functions**

### 3c. Subnetwork Extraction

Extract a subnetwork from a set of selected nodes. The output is a new
BioFabricNetwork with only the selected nodes and their induced edges.

**Test approach:**
1. Load network
2. Select a subset of nodes
3. Extract subnetwork
4. Export NOA/EDA/BIF for the subnetwork
5. Compare against golden

**GoldenGenerator changes needed:**
- Add `--extract-nodes Node1,Node2,...` flag
- After loading + layout, call `fillSubModel()` with selected nodes
- Export the submodel

**Estimated tests:** 3 networks × 3 formats = **9 test functions**

### 3d. Network Statistics / Info

Node count, link count (with/without shadows), lone node count, etc.

**Test approach:** JSON or text output comparison.

**Estimated tests:** **Already implicitly tested** via BIF parity. The BIF
contains all this data. Could add explicit unit tests if desired but
not strictly necessary for parity.

---

## Tier 4: Alignment / VISNAB Features (MEDIUM priority, ~27 tests)

The alignment module merges two networks using a node-to-node mapping and
produces a combined visualization with color-coded nodes and edges.

### 4a. Network Merge

| Input | Description |
|-------|-------------|
| Network A (`.sif` or `.gw`) | First network |
| Network B (`.sif` or `.gw`) | Second network |
| Alignment file (`.align`) | Node mapping: `nodeA nodeB` per line |

**Output:** Merged `BioFabricNetwork` with:
- Combined node set (matched + unmatched)
- Combined edge set with 7 type classifications
- Color assignments (purple/blue/red for nodes)

### 4b. Alignment Scoring

7 alignment quality metrics:

| Score | Full Name |
|-------|-----------|
| EC | Edge Correctness |
| S3 | Symmetric Substructure Score |
| ICS | Induced Conserved Structure |
| NC | Node Correctness |
| NGS | Node-Group Score |
| LGS | Link-Group Score |
| JS | Jaccard Similarity |

### 4c. Additional Alignment Features

| Feature | Description |
|---------|-------------|
| Cycle detection | Detect cycles in alignment mapping (A→B→C→A) |
| Orphan edge filtering | Remove edges where one endpoint is unmatched |
| Alignment layout | Custom layout algorithm for merged networks |

**New input files needed:**
- Two small networks (e.g., `align_net1.sif`, `align_net2.sif`)
- An alignment file mapping nodes between them (e.g., `test.align`)
- Ideally 2-3 alignment scenarios: perfect alignment, partial alignment, no alignment

**GoldenGenerator changes needed:**
- Major extension: accept `--align <net1> <net2> <align_file>` flags
- Load both networks, merge with alignment, export combined result

**Estimated tests:** 3 alignment scenarios × 3 formats + 3 scoring tests = **~12 test functions**
**Plus:** Cycle detection, orphan filtering = **~6 more**
**Plus:** Alignment layout variants = **~9 more**

**Total estimated:** **~27 test functions**

---

## Tier 5: Format Re-Export (LOW priority, ~9 tests)

Test that loading a network and re-exporting it to SIF or GW format
produces output identical to the original (or to Java's export).

**Note:** Java `BioFabricNetwork` does NOT have `writeSIF()` or `writeGW()`
methods. SIF/GW export appears to be handled at a higher level
(`FileLoadFlowsImpl` or export actions). Need to verify whether the original
tool actually supports re-export to these formats from a loaded session.

If supported:

| Test | Description |
|------|-------------|
| SIF → load → export SIF | Round-trip SIF |
| GW → load → export GW | Round-trip GW |
| SIF → load → export GW | Cross-format |

**Estimated tests:** 3 scenarios × 3 networks = **~9 test functions**

---

## Tier 6: Display Option Permutations (LOW priority, ~12 tests)

Display options that affect the BIF output (colors, drain zones):

| Option | Default | What changes in BIF |
|--------|---------|---------------------|
| `minDrainZone` | 1 | Drain zone sizes in `<drainZones>` |
| `nodeLighterLevel` | 0.43 | Node RGB values in `<colors>` |
| `linkDarkerLevel` | 0.43 | Link RGB values in `<colors>` |
| `shadeNodes` | false | Node shading in `<displayOptions>` |

**GoldenGenerator changes needed:**
- Add `--min-drain-zone N` flag
- Add `--node-lighter F` / `--link-darker F` flags

**Test cases:**
- `minDrainZone = 0` (no drain zones)
- `minDrainZone = 5` (larger drain zones)
- Custom color levels (0.0 and 1.0 extremes)

**Estimated tests:** 4 options × 1-2 networks × BIF only = **~12 test functions**

---

## Tier 7: Image Export (LOW priority, ~6 tests)

PNG/JPEG/TIFF export with pixel-level or perceptual comparison.

| Format | Java Class | Notes |
|--------|-----------|-------|
| PNG | `ImageExporter` | Most common, lossless |
| JPEG | `ImageExporter` | Lossy — approximate comparison |
| TIFF | `ImageExporter` | Lossless |

**GoldenGenerator changes needed:**
- Already partially supported via `-pngExport` flag in `ImageGeneratorApplication`
- Need to extend for resolution/dimension parameters

**Test approach:** Export PNG at fixed resolution, compare pixel-by-pixel.
Note: Java AWT rendering may produce platform-dependent pixels.
May need to compare within a tolerance rather than exact byte parity.

**Estimated tests:** 2-3 networks × 2 formats = **~6 test functions**

---

## Master Summary

| Tier | Category | Est. Tests | Files Needed |
|------|----------|------------|--------------|
| 1 | Other layout algorithms | ~96 | DAG networks, attribute files, bipartite network |
| 2 | Fixed-order import (NOA/EDA input) | ~12 | Hand-crafted NOA/EDA files |
| 3 | Network analysis operations | ~18 | (use existing networks) |
| 4 | Alignment / VISNAB | ~27 | Paired networks + .align files |
| 5 | Format re-export (SIF/GW write) | ~9 | (use existing networks) |
| 6 | Display option permutations | ~12 | (use existing networks) |
| 7 | Image export | ~6 | (use existing networks) |
| **Total remaining** | | **~180** | |
| **Already covered** | | **124** | |
| **Grand total** | | **~304** | |

---

## New Input Files To Create

### DAG networks (for HierDAG layout)

```
# dag_simple.sif — basic 5-node DAG
Root	pd	Child1
Root	pd	Child2
Child1	pd	Leaf1
Child1	pd	Leaf2
Child2	pd	Leaf3

# dag_diamond.sif — 4-node diamond DAG
Top	pd	Left
Top	pd	Right
Left	pd	Bottom
Right	pd	Bottom

# dag_deep.sif — 8-node linear DAG
L0	pd	L1
L1	pd	L2
L2	pd	L3
L3	pd	L4
L4	pd	L5
L5	pd	L6
L6	pd	L7
```

### Bipartite network (for SetLayout)

```
# bipartite.sif
L1	pp	R1
L1	pp	R2
L2	pp	R1
L2	pp	R3
L3	pp	R2
L3	pp	R3
```

### Node attribute files (for NodeClusterLayout)

```
# multi_relation_clusters.na
ClusterAssignment
A = group1
B = group1
C = group2
D = group2
E = group3
```

### Alignment test files

```
# align_net1.sif
A1	pp	B1
B1	pp	C1
C1	pp	A1

# align_net2.sif
X1	pp	Y1
Y1	pp	Z1

# test.align — maps net1 nodes to net2 nodes
A1	X1
B1	Y1
C1	Z1
```

### Fixed-order NOA files (for fixed-order import)

```
# triangle_reversed.noa — reverse of default order
Node Row
C = 0
B = 1
A = 2
```

---

## Implementation Plan For Delegation

### Phase A: Layout algorithm tests (Tier 1)

1. Extend `GoldenGenerator.java` with `--layout` flag supporting all 7 algorithms
2. Add algorithm-specific flags (`--point-up`, `--control-nodes`, `--set-nodes`, etc.)
3. Create new input files (DAG networks, bipartite network, attribute files)
4. Run `generate_goldens.sh` with new algorithm variants
5. Add `parity_test_layout!` macro entries in `parity_tests.rs`
6. Add `[[test]]` entries in `test_manifest.toml`

### Phase B: Fixed-order import tests (Tier 2)

1. Create hand-crafted NOA/EDA files for a few networks
2. Extend `GoldenGenerator.java` with `--noa-file` / `--eda-file` flags
3. Generate goldens and add test entries

### Phase C: Analysis operation tests (Tier 3)

1. Create a separate test file `crates/core/tests/analysis_tests.rs`
2. Define expected outputs for cycle detection, Jaccard, subnetwork extraction
3. These compare JSON/text output (not BIF), so they're simpler

### Phase D: Alignment tests (Tier 4)

1. Create paired test networks and `.align` files
2. Major `GoldenGenerator.java` extension for alignment loading
3. Generate goldens for merged network output
4. Add alignment-specific test entries

### Phase E: Remaining tiers (5-7)

1. Format re-export tests (if supported by original tool)
2. Display option permutation tests
3. Image export tests (PNG pixel comparison)
