//! Shared utility types and functions.
//!
//! - [`quadtree`] — Spatial indexing for efficient range queries
//! - [`hit_test`] — Hit-testing infrastructure for user interaction (click, hover, select)
//! - [`data`] — Set operations, normalization, and data manipulation helpers

pub mod data;
pub mod hit_test;
pub mod quadtree;
