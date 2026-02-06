//! Alignment loader stubs.
//!
//! Parity with Java `AlignmentLoader`.

use crate::io::ParseError;
use crate::io::align::AlignmentMap;

/// Load alignment mappings from string data (stub).
pub struct AlignmentLoader;

impl AlignmentLoader {
    pub fn parse_string(_data: &str) -> Result<AlignmentMap, ParseError> {
        // TODO: Implement alignment file parsing (delegates to io::align).
        todo!("Implement alignment loader")
    }
}
