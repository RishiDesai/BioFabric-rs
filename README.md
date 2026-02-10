# BioFabric-rs

A Rust rewrite of [BioFabric](https://github.com/wjrl/BioFabric) and the [VISNAB](https://github.com/wjrl/AlignmentPlugin) network alignment plugin. This project aims to provide a high-performance, memory-efficient implementation of BioFabric's visualization techniques.

## Overview

BioFabric uses a unique visualization approach:

- **Nodes** are horizontal lines
- **Edges** are vertical line segments connecting two node lines

This eliminates the "hairball" problem of traditional node-link diagrams and scales to very large networks.

## Project Structure

This is a Cargo workspace:

```
biofabric-rs/
├── crates/
│   ├── core/    # biofabric-core — pure computation library
│   └── cli/     # biofabric      — command-line tool
```

## Building

```bash
cargo build
```

## Testing

See `tests/parity/README.md` for detailed test documentation.

```bash
# All tests (requires goldens)
cargo test

# Parity tests only
cargo test --test parity_tests

# Analysis tests only
cargo test --test analysis_tests

# CLI integration tests
cargo test --test cli_tests
```

## References

- **BioFabric**: Longabaugh, W. J. R. (2012). [Combing the hairball with BioFabric: a new approach for visualization of large networks](https://doi.org/10.1186/1471-2105-13-275). *BMC Bioinformatics*, 13(1), 275.
- **VISNAB**: Desai, R. M., Longabaugh, W. J. R., & Hayes, W. B. (2021). [BioFabric Visualization of Network Alignments](https://doi.org/10.1007/978-3-030-57173-3_4). In *Biological Networks and Pathway Analysis* (pp. 53-73). Springer, Cham.

## License

LGPL-2.1
