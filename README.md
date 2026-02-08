# BioFabric-rs

A Rust rewrite of [BioFabric](https://github.com/wjrl/BioFabric) and the [VISNAB](https://doi.org/10.1007/978-3-030-57173-3_4) alignment plugin. Provides both a CLI tool and an interactive web application for visualizing networks with millions of nodes and edges.

## Overview

BioFabric uses a unique visualization approach:

- **Nodes** are horizontal lines, each at a distinct row
- **Edges** are vertical line segments connecting two node rows
- **Node span** extends horizontally from its leftmost to rightmost incident edge

This eliminates the "hairball" problem of traditional node-link diagrams and scales to very large networks.

## Project Structure

Cargo workspace with three crates plus a TypeScript web frontend:

```
biofabric-rs/
├── crates/
│   ├── core/              # biofabric-core  — pure computation library
│   │   └── src/
│   │       ├── model/     # Network, Node, Link, Annotation
│   │       ├── io/        # Parsers: SIF, GW, .align, JSON, XML
│   │       ├── layout/    # 7 layout algorithms + traits
│   │       ├── alignment/ # Network alignment (native, not plugin)
│   │       ├── analysis/  # BFS, DFS, components, cycles
│   │       ├── render/    # Colors, bucket renderer, tiles
│   │       └── util/      # Quadtree, data helpers
│   ├── cli/               # biofabric-cli   — command-line tool
│   └── wasm/              # biofabric-wasm  — WebAssembly bindings
├── web/                   # TypeScript + Vite + WebGL2 frontend
│   └── src/
│       ├── renderer/      # WebGL2 instanced line rendering
│       ├── ui/            # App shell, controls, file loading, search
│       └── wasm/          # WASM bridge (typed API)
├── BioFabric/             # Reference: original Java implementation
├── AlignmentPlugin/       # Reference: VISNAB Java plugin
└── SANA/                  # Reference: network alignment tool
```

## Module Status

### Core Library

| Module | Status | Description |
|--------|--------|-------------|
| `model` | Complete | Network, Node, Link, Annotation types |
| `io::sif` | Complete | SIF file parser/writer |
| `io::gw` | Complete | GW (LEDA) file parser/writer |
| `io::align` | Complete | .align file parser |
| `io::json` | Complete | JSON import/export |
| `io::xml` | Complete | BioFabric XML session format |
| `io::order` | Complete | Node/link order import/export |
| `io::attribute` | Stub | Node attribute loader (TSV) |
| `layout::default` | Complete | Default BFS node layout + edge layout |
| `layout::similarity` | Stub | Node similarity (Jaccard) layout |
| `layout::hierarchy` | Complete | Hierarchical DAG layout |
| `layout::cluster` | Stub | Node cluster layout |
| `layout::control_top` | Complete | Control-top layout |
| `layout::set` | Stub | Set membership layout |
| `layout::world_bank` | Stub | Hub-spoke layout |
| `alignment::types` | Complete | NodeColor, EdgeType, MergedNodeId |
| `alignment::merge` | Complete | Network merging via alignment |
| `alignment::scoring` | Complete | EC, S3, ICS, NC, NGS, LGS, JS |
| `alignment::groups` | Complete | Node group classification |
| `alignment::cycle` | Complete | Alignment cycle/path detection |
| `alignment::layout` | Complete | GROUP, ORPHAN, CYCLE layout modes |
| `analysis::graph` | Complete | BFS, DFS, connected components |
| `analysis::cycle` | Complete | Cycle detection in directed graphs |
| `render::color` | Complete | Color palette generation |
| `render::gpu_data` | Partial | Render extraction (extraction stub) |
| `export::image` | Partial | Image export (background-only) |
| `util::quadtree` | Stub | Spatial indexing |

### CLI

All subcommands are wired and functional. Run `biofabric --help` for full usage.

```
biofabric layout        <input>  [--algorithm default|similarity|...] [-o layout.json]
biofabric render        <input>  [--width 1920] [--height 0] [-o output.png]
biofabric info          <input>  [--all] [--format text|json]
biofabric convert       <input>  --format sif|gw|json|xml [-o output]
biofabric align         <g1> <g2> <align> [--layout group|orphan|cycle] [--score] [--json]
biofabric compare       <input>  <nodeA> <nodeB> [--format text|json]
biofabric extract       <input>  --node <center> [--hops N] [-o output]
biofabric export-order  <input>  [--what nodes|links] [-o output]
biofabric search        <input>  <pattern> [--regex] [-i] [--target nodes|relations|both]
```

Global flag: `--quiet` suppresses non-essential stderr output.

### Web App

TypeScript + Vite + WebGL2, consuming `biofabric-wasm` for all computation.

## Building

```bash
# Build all Rust crates
cargo build

# Run tests
cargo test

# Build WASM (requires wasm-pack)
cd web && npm run wasm:build

# Run web dev server
cd web && npm install && npm run dev
```

## References

- Original Java BioFabric: https://github.com/wjrl/BioFabric
- Original Alignment plugin: https://github.com/wjrl/AlignmentPlugin
- BioFabric paper: Longabaugh WJR. Combing the hairball with BioFabric: a new approach for visualization of large networks. BMC Bioinformatics. 2012.
- VISNAB paper: Desai, R.M., Longabaugh, W.J.R., Hayes, W.B. (2021). BioFabric Visualization of Network Alignments. Springer, Cham. https://doi.org/10.1007/978-3-030-57173-3_4

## License

LGPL-2.1
