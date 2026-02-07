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

    // Case-insensitive node name normalization (like Java's DataUtil.normKey).
    // Maps uppercase(name) → first-seen original-case name.
    let mut norm_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let normalize = |name: &str, map: &mut std::collections::HashMap<String, String>| -> String {
        let norm = name.to_uppercase().replace(' ', "");
        map.entry(norm).or_insert_with(|| name.to_string()).clone()
    };

    // Phase 1: Parse all lines, collecting raw (normalized) links and lone nodes.
    // Links are stored as (source, relation, target) with normalized node names.
    let mut raw_links: Vec<(String, String, String)> = Vec::new();
    let mut lone_node_names: Vec<String> = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;

        // Skip completely empty lines (after trim)
        if line.trim().is_empty() {
            continue;
        }

        // Split the ORIGINAL line by tab (not trimmed).
        // Java: `line.split("\\t")` operates on untrimmed line.
        // If only 1 token and no tab found, split by space.
        let tokens: Vec<&str> = if line.contains('\t') {
            line.split('\t').collect()
        } else {
            line.split_whitespace().collect()
        };

        match tokens.len() {
            3 => {
                let source = normalize(strip_quotes(tokens[0]), &mut norm_names);
                let relation = strip_quotes(tokens[1]).to_string();
                let target = normalize(strip_quotes(tokens[2]), &mut norm_names);
                raw_links.push((source, relation, target));
            }
            1 => {
                let name = normalize(strip_quotes(tokens[0]), &mut norm_names);
                lone_node_names.push(name);
            }
            _ => {
                stats.bad_lines.push(line.to_string());
            }
        }
    }

    // Phase 2: Preprocess links — deduplicate and remove undirected reverse pairs.
    // This matches Java's BuildExtractorImpl.preprocessLinks():
    // - Exact duplicates (same src, tgt, rel) → culled
    // - For undirected non-feedback links, if both (A→B) and (B→A) exist, keep first-seen
    //
    // We track seen edges using a canonical form (sorted pair) for undirected dedup,
    // but preserve the original link direction from the first occurrence.
    let mut seen_edges: std::collections::HashSet<(String, String, String)> =
        std::collections::HashSet::new();
    let mut links: Vec<Link> = Vec::new();

    for (source, relation, target) in raw_links {
        let is_feedback = source == target;

        // For undirected non-feedback links, use canonical (sorted) key
        // to catch both A→B and B→A as the same edge.
        // Use normKey (uppercase + no spaces) for consistent comparison.
        let norm_src = source.to_uppercase().replace(' ', "");
        let norm_tgt = target.to_uppercase().replace(' ', "");
        let norm_rel = relation.to_uppercase().replace(' ', "");
        let canonical_key = if !is_feedback {
            let (a, b) = if norm_src <= norm_tgt {
                (norm_src, norm_tgt)
            } else {
                (norm_tgt, norm_src)
            };
            (a, b, norm_rel)
        } else {
            (norm_src, norm_tgt, norm_rel)
        };

        if !seen_edges.insert(canonical_key) {
            stats.duplicate_links += 1;
            continue;
        }

        let link = Link::new(source.as_str(), target.as_str(), relation.as_str());
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

    // Phase 3: Build the network
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
