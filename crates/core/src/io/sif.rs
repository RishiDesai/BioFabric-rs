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
use std::io::{BufRead, BufReader, Read, Write};
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
    let mut stats = ImportStats::new();
    let mut links: Vec<Link> = Vec::new();
    let mut lone_node_names: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Split by tab first; if only 1 token and no tab, split by space
        let tokens: Vec<&str> = if trimmed.contains('\t') {
            trimmed.split('\t').collect()
        } else {
            trimmed.split_whitespace().collect()
        };

        match tokens.len() {
            3 => {
                let source = strip_quotes(tokens[0]);
                let relation = strip_quotes(tokens[1]);
                let target = strip_quotes(tokens[2]);

                let link = Link::new(source, target, relation);
                let is_feedback = link.is_feedback();

                // Add the regular link
                links.push(link.clone());
                stats.link_count += 1;

                // Add inline shadow if not self-loop
                if !is_feedback {
                    if let Some(shadow) = link.to_shadow() {
                        links.push(shadow);
                        stats.shadow_link_count += 1;
                    }
                }
            }
            1 => {
                let node_name = strip_quotes(tokens[0]);
                lone_node_names.push(node_name.to_string());
            }
            _ => {
                stats.bad_lines.push(trimmed.to_string());
            }
        }
    }

    // Build the network: add all links in order, then lone nodes
    let mut network = Network::with_capacity(0, links.len());
    for link in links {
        network.add_link(link);
    }
    for name in &lone_node_names {
        network.add_lone_node(name.as_str());
    }

    stats.node_count = network.node_count();
    stats.lone_node_count = network.lone_nodes().len();

    Ok((network, stats))
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
    // Write non-shadow links
    for link in network.links() {
        if link.is_shadow {
            continue;
        }
        writeln!(writer, "{}\t{}\t{}", link.source, link.relation, link.target)
            .map_err(|e| ParseError::Io(e))?;
    }

    // Write lone nodes
    for id in network.lone_nodes() {
        writeln!(writer, "{}", id).map_err(|e| ParseError::Io(e))?;
    }

    Ok(())
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
