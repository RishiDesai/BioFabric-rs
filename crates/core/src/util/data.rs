//! Data manipulation utilities.
//!
//! Set operations, normalization helpers, and misc utilities used across
//! the crate.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.util.DataUtil`

use std::collections::HashSet;
use std::hash::Hash;

/// Compute the union of two sets.
pub fn union<'a, T: Eq + Hash>(a: &'a HashSet<T>, b: &'a HashSet<T>) -> HashSet<&'a T> {
    a.union(b).collect()
}

/// Compute the intersection of two sets.
pub fn intersection<'a, T: Eq + Hash>(a: &'a HashSet<T>, b: &'a HashSet<T>) -> HashSet<&'a T> {
    a.intersection(b).collect()
}

/// Compute the set difference (a - b).
pub fn difference<'a, T: Eq + Hash>(a: &'a HashSet<T>, b: &'a HashSet<T>) -> HashSet<&'a T> {
    a.difference(b).collect()
}

/// Normalize a string for comparison (trim, lowercase).
pub fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Normalize a vector of f64 to sum to 1.0.
pub fn normalize_ratios(values: &[f64]) -> Vec<f64> {
    let sum: f64 = values.iter().sum();
    if sum == 0.0 {
        vec![0.0; values.len()]
    } else {
        values.iter().map(|v| v / sum).collect()
    }
}
