# BioFabric-rs

A Rust implementation of [BioFabric](https://github.com/wjrl/BioFabric), a network visualization tool that represents nodes as horizontal lines and edges as vertical line segments.

## Overview

BioFabric uses a unique visualization approach that makes it possible to see large networks (100K+ nodes) while still being able to identify individual edges:

- **Nodes** are horizontal lines, each at a distinct row
- **Edges** are vertical line segments connecting two node rows
- **Node span** extends horizontally from its leftmost to rightmost incident edge

This approach eliminates the "hairball" problem of traditional node-link diagrams.

## Project Status

This is a work-in-progress port of the Java BioFabric to Rust. Current status:

| Module | Status | Description |
|--------|--------|-------------|
| `model` | ✅ Complete | Core types: Network, Node, Link |
| `io::sif` | 🔲 Stub | SIF file parser |
| `io::gw` | 🔲 Stub | GW (LEDA) file parser |
| `layout::default` | 🔲 Stub | Default BFS layout |
| `layout::similarity` | 🔲 Stub | Node similarity layout |
| `analysis::graph` | 🔲 Stub | BFS, DFS, connected components |

## Building

```bash
cargo build
cargo test
```

## Usage

```rust
use biofabric::{Network, io::sif, layout::{DefaultNodeLayout, DefaultEdgeLayout, NodeLayout, EdgeLayout}};
use std::path::Path;

// Load network from SIF file
let network = sif::parse_file(Path::new("network.sif"))?;

// Compute layout
let node_layout = DefaultNodeLayout::new();
let edge_layout = DefaultEdgeLayout::new();

let node_order = node_layout.layout_nodes(&network, &Default::default())?;
let layout = edge_layout.layout_edges(&network, &node_order, &Default::default())?;

// Access layout information
for (node_id, info) in layout.iter_nodes() {
    println!("Node {} at row {}, spans columns {}-{}", 
        node_id, info.row, info.min_col, info.max_col);
}
```

## Architecture

```
biofabric/
├── model/           # Core data structures
│   ├── node.rs      # Node and NodeId types
│   ├── link.rs      # Link (edge) type with shadow support
│   └── network.rs   # Network container
├── io/              # File format parsers
│   ├── sif.rs       # Simple Interaction Format
│   └── gw.rs        # LEDA Graph Format
├── layout/          # Layout algorithms
│   ├── traits.rs    # NodeLayout and EdgeLayout traits
│   ├── result.rs    # NetworkLayout output structure
│   ├── default.rs   # Default BFS-based layout
│   └── similarity.rs # Node similarity layout
└── analysis/        # Graph algorithms
    └── graph.rs     # BFS, DFS, connected components
```

## References

- Original Java implementation: https://github.com/wjrl/BioFabric
- BioFabric paper: Longabaugh WJR. Combing the hairball with BioFabric: a new approach for visualization of large networks. BMC Bioinformatics. 2012.
- Desai, R.M., Longabaugh, W.J.R., Hayes, W.B. (2021). BioFabric Visualization of Network Alignments. In: Yoon, BJ., Qian, X. (eds) Recent Advances in Biological Network Analysis. Springer, Cham. https://doi.org/10.1007/978-3-030-57173-3_4

## License

Apache License, Version 2.0.  
