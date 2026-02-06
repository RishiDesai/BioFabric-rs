//! Alignment file parser (.align format).
//!
//! The `.align` format is a simple two-column text file that maps nodes from
//! a smaller network (G1) to a larger network (G2):
//!
//! ```text
//! nodeG1_1  nodeG2_1
//! nodeG1_2  nodeG2_2
//! nodeG1_3  nodeG2_3
//! ```
//!
//! Columns are separated by tabs or spaces. Each line maps exactly one G1
//! node to one G2 node. Not every G2 node needs to appear in the mapping
//! (G2 is typically larger than G1).
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.plugin.core.align.NetworkAlignmentPlugIn` (alignment file loading)

use super::ParseError;
use crate::model::NodeId;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// A parsed alignment mapping from G1 node names to G2 node names.
pub type AlignmentMap = HashMap<NodeId, NodeId>;

/// Parse an alignment file from a path.
///
/// Returns a mapping from G1 node IDs to G2 node IDs.
pub fn parse_file(path: &Path) -> Result<AlignmentMap, ParseError> {
    let file = std::fs::File::open(path)?;
    parse_reader(BufReader::new(file))
}

/// Parse an alignment file from any reader.
pub fn parse_reader<R: Read>(_reader: BufReader<R>) -> Result<AlignmentMap, ParseError> {
    // TODO: Implement alignment file parsing
    //
    // Algorithm:
    // 1. Read line by line
    // 2. Skip empty lines and comment lines (starting with #)
    // 3. Split each line by whitespace (tab or space)
    // 4. Expect exactly 2 tokens per line: g1_node g2_node
    // 5. Build HashMap<NodeId, NodeId> mapping g1 -> g2
    // 6. Check for duplicate g1 entries (each g1 node maps to exactly one g2 node)
    //
    todo!("Implement .align parser")
}

/// Parse an alignment string directly.
pub fn parse_string(content: &str) -> Result<AlignmentMap, ParseError> {
    parse_reader(BufReader::new(content.as_bytes()))
}
