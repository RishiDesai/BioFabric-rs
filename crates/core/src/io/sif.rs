//! SIF (Simple Interaction Format) parser.
//!
//! The SIF format is a simple text format for representing networks:
//!
//! ```text
//! # Comments start with #
//! nodeA relation nodeB
//! nodeA relation nodeC
//! nodeD relation nodeE
//! isolatedNode
//! ```
//!
//! Each line contains either:
//! - Three tokens: `source relation target` (defines an edge)
//! - One token: `node` (defines an isolated node with no edges)
//!
//! Tokens can be separated by tabs or spaces.
//!
//! ## References
//!
//! - Java implementation: `org.systemsbiology.biofabric.io.SIFImportLoader`
//! - Cytoscape SIF format: <https://cytoscape.org/manual/Cytoscape3_10_0Manual.pdf>
//!
//! ## Example
//!
//! ```rust,ignore
//! use biofabric::io::sif;
//! use std::path::Path;
//!
//! let network = sif::parse_file(Path::new("network.sif"))?;
//! println!("Loaded {} nodes, {} links", network.node_count(), network.link_count());
//! ```

use super::{ImportStats, ParseError};
use crate::model::{Link, Network};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Parse a SIF file from a path.
///
/// # Arguments
/// * `path` - Path to the SIF file
///
/// # Returns
/// * `Ok(Network)` - The parsed network
/// * `Err(ParseError)` - If the file could not be parsed
///
/// # Example
/// ```rust,ignore
/// let network = sif::parse_file(Path::new("network.sif"))?;
/// ```
pub fn parse_file(path: &Path) -> Result<Network, ParseError> {
    let file = std::fs::File::open(path)?;
    parse_reader(BufReader::new(file))
}

/// Parse a SIF file from any reader.
///
/// # Arguments
/// * `reader` - Any type implementing `Read`
///
/// # Returns
/// * `Ok(Network)` - The parsed network
/// * `Err(ParseError)` - If the content could not be parsed
pub fn parse_reader<R: Read>(reader: BufReader<R>) -> Result<Network, ParseError> {
    let (network, _stats) = parse_reader_with_stats(reader)?;
    Ok(network)
}

/// Parse a SIF file and return import statistics.
///
/// This is useful for debugging or reporting on the import process.
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
    // TODO: Implement SIF parsing
    //
    // See Java implementation: org.systemsbiology.biofabric.io.SIFImportLoader
    //
    // Algorithm:
    // 1. Read file line by line
    // 2. Skip empty lines and comments (lines starting with #)
    // 3. Split each line by tabs, or spaces if no tabs
    // 4. If 3 tokens: create link (source, relation, target)
    //    - Also create shadow link if source != target
    // 5. If 1 token: add as lone node
    // 6. If 2 tokens or >3 tokens: record as bad line
    // 7. Strip quotes from tokens if present
    //
    // Key behaviors to match:
    // - Nodes are deduplicated (same name = same node)
    // - Shadow links are created for non-feedback edges
    // - Relations are case-sensitive
    // - Empty lines are skipped
    //
    todo!("Implement SIF parser - see BioFabric/src/org/systemsbiology/biofabric/io/SIFImportLoader.java")
}

/// Parse a SIF string directly.
///
/// Convenience function for testing or parsing inline data.
pub fn parse_string(content: &str) -> Result<Network, ParseError> {
    parse_reader(BufReader::new(content.as_bytes()))
}

/// Strip surrounding quotes from a string.
///
/// Handles both single and double quotes.
fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

// ============================================================================
// SIF writer
// ============================================================================

/// Write a network to SIF format.
///
/// # Arguments
/// * `network` — The network to write
/// * `path` — Output file path
///
/// # Notes
///
/// - Shadow links are **not** written (they are a display artifact, not data)
/// - Lone nodes are written as single-token lines
/// - Directed links use `->` notation: `source relation -> target`
///   (standard Cytoscape extended SIF); if your downstream tools don't
///   support this, set `directed = None` before writing.
pub fn write_file(network: &Network, path: &Path) -> Result<(), ParseError> {
    let file = std::fs::File::create(path)?;
    write_writer(network, std::io::BufWriter::new(file))
}

/// Write a network in SIF format to any writer.
pub fn write_writer<W: std::io::Write>(
    network: &Network,
    mut writer: W,
) -> Result<(), ParseError> {
    // TODO: Implement SIF writing
    //
    // Algorithm:
    // 1. For each non-shadow link:
    //    - Write: "{source}\t{relation}\t{target}\n"
    // 2. For each lone node:
    //    - Write: "{node_id}\n"
    //
    // Key behaviors:
    // - Skip shadow links (is_shadow == true)
    // - Skip duplicate edges (same source, target, relation but reversed)
    //   for undirected networks
    // - Lone nodes written at the end as single-token lines
    //
    todo!("Implement SIF writer")
}

/// Write a network to SIF format as a string.
///
/// Convenience function for testing.
pub fn write_string(network: &Network) -> Result<String, ParseError> {
    let mut buf = Vec::new();
    write_writer(network, &mut buf)?;
    String::from_utf8(buf).map_err(|e| ParseError::InvalidFormat {
        line: 0,
        message: format!("UTF-8 encoding error: {}", e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("'world'"), "world");
        assert_eq!(strip_quotes("no quotes"), "no quotes");
        assert_eq!(strip_quotes("  \"spaced\"  "), "spaced");
    }

    // TODO: Add more tests once parse_string is implemented
    //
    // #[test]
    // fn test_parse_simple() {
    //     let content = "A activates B\nB inhibits C";
    //     let network = parse_string(content).unwrap();
    //     assert_eq!(network.node_count(), 3);
    //     // Should have 2 real links + 2 shadow links = 4 total
    //     assert_eq!(network.link_count(), 4);
    // }
    //
    // #[test]
    // fn test_parse_lone_node() {
    //     let content = "A activates B\nC";
    //     let network = parse_string(content).unwrap();
    //     assert_eq!(network.node_count(), 3);
    //     assert!(network.lone_nodes().contains(&NodeId::new("C")));
    // }
    //
    // #[test]
    // fn test_parse_feedback() {
    //     let content = "A self A";
    //     let network = parse_string(content).unwrap();
    //     // Feedback links don't get shadows
    //     assert_eq!(network.link_count(), 1);
    // }
}
