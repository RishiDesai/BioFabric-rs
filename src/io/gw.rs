//! GW (LEDA Graph Format) parser.
//!
//! The GW format is a text format used by the LEDA library:
//!
//! ```text
//! LEDA.GRAPH
//! string
//! short
//! -2
//! 5
//! |{node1}|
//! |{node2}|
//! |{node3}|
//! |{node4}|
//! |{node5}|
//! 6
//! 1 2 0 |{edge_label}|
//! 2 3 0 |{edge_label}|
//! ...
//! ```
//!
//! ## Format Structure
//!
//! 1. Header: `LEDA.GRAPH`
//! 2. Node type (usually `string`)
//! 3. Edge type (usually `short`)
//! 4. Direction indicator (-1 = directed, -2 = undirected)
//! 5. Number of nodes
//! 6. Node labels, one per line: `|{label}|`
//! 7. Number of edges
//! 8. Edge definitions: `source target reversal |{label}|`
//!
//! ## References
//!
//! - LEDA GW format: <http://www.algorithmic-solutions.info/leda_manual/GW.html>
//! - Java implementation: `org.systemsbiology.biofabric.io.GWImportLoader`
//!
//! ## Example
//!
//! ```rust,ignore
//! use biofabric::io::gw;
//! use std::path::Path;
//!
//! let network = gw::parse_file(Path::new("network.gw"))?;
//! ```

use super::{ImportStats, ParseError};
use crate::model::Network;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Expected header for GW files.
const GW_HEADER: &str = "LEDA.GRAPH";

/// Parse a GW file from a path.
///
/// # Arguments
/// * `path` - Path to the GW file
///
/// # Returns
/// * `Ok(Network)` - The parsed network
/// * `Err(ParseError)` - If the file could not be parsed
pub fn parse_file(path: &Path) -> Result<Network, ParseError> {
    let file = std::fs::File::open(path)?;
    parse_reader(BufReader::new(file))
}

/// Parse a GW file from any reader.
///
/// # Arguments
/// * `reader` - Buffered reader for the input
///
/// # Returns
/// * `Ok(Network)` - The parsed network
/// * `Err(ParseError)` - If the content could not be parsed
pub fn parse_reader<R: Read>(reader: BufReader<R>) -> Result<Network, ParseError> {
    let (network, _stats) = parse_reader_with_stats(reader)?;
    Ok(network)
}

/// Parse a GW file and return import statistics.
///
/// # Arguments
/// * `reader` - Buffered reader for the input
///
/// # Returns
/// * `Ok((Network, ImportStats))` - The parsed network and statistics
/// * `Err(ParseError)` - If the file could not be parsed
pub fn parse_reader_with_stats<R: Read>(
    reader: BufReader<R>,
) -> Result<(Network, ImportStats), ParseError> {
    // TODO: Implement GW parsing
    //
    // See Java implementation: org.systemsbiology.biofabric.io.GWImportLoader
    //
    // Algorithm:
    // 1. Read header line, verify it's "LEDA.GRAPH"
    // 2. Read node type line (e.g., "string")
    // 3. Read edge type line (e.g., "short")
    // 4. Read direction indicator (-1 = directed, -2 = undirected)
    // 5. Read node count
    // 6. Read node labels (format: |{label}|)
    // 7. Read edge count
    // 8. Read edges (format: source_idx target_idx reversal_idx |{label}|)
    //    - Indices are 1-based
    //    - reversal_idx is 0 for no reversal
    //
    // Key behaviors to match:
    // - Node indices in edges are 1-based
    // - Edge labels become the relation
    // - Empty labels get a default relation
    // - Handle both directed and undirected graphs
    //
    todo!("Implement GW parser - see BioFabric/src/org/systemsbiology/biofabric/io/GWImportLoader.java")
}

/// Parse a GW string directly.
///
/// Convenience function for testing or parsing inline data.
pub fn parse_string(content: &str) -> Result<Network, ParseError> {
    parse_reader(BufReader::new(content.as_bytes()))
}

/// Extract a label from GW format: |{label}|
fn extract_label(s: &str) -> Option<&str> {
    let s = s.trim();
    if s.starts_with("|{") && s.ends_with("}|") {
        Some(&s[2..s.len() - 2])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_label() {
        assert_eq!(extract_label("|{hello}|"), Some("hello"));
        assert_eq!(extract_label("|{}|"), Some(""));
        assert_eq!(extract_label("no braces"), None);
        assert_eq!(extract_label("  |{spaced}|  "), Some("spaced"));
    }

    // TODO: Add more tests once parse_string is implemented
    //
    // #[test]
    // fn test_parse_simple_gw() {
    //     let content = r#"LEDA.GRAPH
    // string
    // short
    // -2
    // 3
    // |{A}|
    // |{B}|
    // |{C}|
    // 2
    // 1 2 0 |{rel1}|
    // 2 3 0 |{rel2}|
    // "#;
    //     let network = parse_string(content).unwrap();
    //     assert_eq!(network.node_count(), 3);
    //     assert_eq!(network.link_count(), 4); // 2 links + 2 shadows
    // }
}
