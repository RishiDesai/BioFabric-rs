//! Hit-testing for BioFabric layouts.
//!
//! Given a point in grid coordinates (e.g., from a mouse click), determines
//! which node or link is at that position. Uses the [`QuadTree`] spatial
//! index for efficient O(log N) lookup in large networks.
//!
//! ## Architecture
//!
//! 1. After a layout is computed, call [`HitIndex::build`] to create a
//!    spatial index over all nodes and links.
//! 2. On user interaction (click, hover), call [`HitIndex::hit_test`] with
//!    the grid coordinates and a tolerance (in grid units, accounting for
//!    zoom level).
//! 3. The result tells you which node(s) and/or link(s) are under the cursor.
//!
//! ## References
//!
//! - Java: `BioFabricPanel.selectAtPoint()`, `BioFabricPanel.selectWithRect()`

use super::quadtree::{QuadItem, QuadTree, Rect};
use crate::layout::result::{LinkLayout, NetworkLayout, NodeLayout};
use crate::model::NodeId;

/// What kind of element was hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitElement {
    /// A node (horizontal line) was hit.
    Node {
        /// The node ID.
        id: NodeId,
        /// The row this node occupies.
        row: usize,
    },
    /// A link (vertical line segment) was hit.
    Link {
        /// Column this link occupies.
        column: usize,
        /// Index into the layout's link list.
        link_index: usize,
        /// Source node ID.
        source: NodeId,
        /// Target node ID.
        target: NodeId,
    },
}

/// Result of a hit-test query.
#[derive(Debug, Clone, Default)]
pub struct HitResult {
    /// All elements found at or near the query point.
    pub hits: Vec<HitElement>,
}

impl HitResult {
    /// Whether any element was hit.
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }

    /// The first (closest) node hit, if any.
    pub fn first_node(&self) -> Option<&NodeId> {
        self.hits.iter().find_map(|h| match h {
            HitElement::Node { id, .. } => Some(id),
            _ => None,
        })
    }

    /// The first (closest) link hit, if any.
    pub fn first_link(&self) -> Option<usize> {
        self.hits.iter().find_map(|h| match h {
            HitElement::Link { link_index, .. } => Some(*link_index),
            _ => None,
        })
    }
}

/// Which elements in a hit-test index entry represent.
#[derive(Debug, Clone)]
enum HitData {
    Node { id: NodeId, row: usize },
    Link { index: usize, source: NodeId, target: NodeId, column: usize },
}

/// Spatial index for hit-testing a computed layout.
///
/// Build once after layout computation, then query repeatedly on
/// user interactions (mouse move, click, drag-select).
pub struct HitIndex {
    tree: QuadTree<HitData>,
}

impl HitIndex {
    /// Build a hit-test index from a computed layout.
    ///
    /// # Arguments
    ///
    /// * `layout` — A completed `NetworkLayout`
    /// * `show_shadows` — Whether to include shadow links in the index.
    ///   Should match the current display mode.
    pub fn build(layout: &NetworkLayout, show_shadows: bool) -> Self {
        let bounds = Rect::new(
            0.0,
            0.0,
            (if show_shadows {
                layout.column_count
            } else {
                layout.column_count_no_shadows
            }) as f64
                + 1.0,
            layout.row_count as f64 + 1.0,
        );

        let mut tree = QuadTree::new(bounds, 16, 12);

        // Insert nodes
        for (id, node) in layout.iter_nodes() {
            let (min_col, max_col) = if show_shadows {
                (node.min_col, node.max_col)
            } else {
                (node.min_col_no_shadows, node.max_col_no_shadows)
            };

            // Skip nodes with no edges in this display mode
            if min_col > max_col {
                continue;
            }

            let item = QuadItem {
                bounds: Rect::new(
                    min_col as f64,
                    node.row as f64,
                    (max_col - min_col) as f64,
                    0.0, // nodes are horizontal lines (zero height)
                ),
                data: HitData::Node {
                    id: id.clone(),
                    row: node.row,
                },
            };
            tree.insert(item);
        }

        // Insert links
        for (i, link) in layout.iter_links().enumerate() {
            if !show_shadows && link.is_shadow {
                continue;
            }

            let column = if show_shadows {
                link.column as f64
            } else {
                match link.column_no_shadows {
                    Some(c) => c as f64,
                    None => continue, // shadow link in no-shadow mode
                }
            };

            let top = link.top_row() as f64;
            let bottom = link.bottom_row() as f64;

            let item = QuadItem {
                bounds: Rect::new(
                    column,
                    top,
                    0.0, // links are vertical lines (zero width)
                    bottom - top,
                ),
                data: HitData::Link {
                    index: i,
                    source: link.source.clone(),
                    target: link.target.clone(),
                    column: link.column,
                },
            };
            tree.insert(item);
        }

        Self { tree }
    }

    /// Test what elements are at the given grid coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` — X coordinate (column) in grid space
    /// * `y` — Y coordinate (row) in grid space
    /// * `tolerance` — Half-width of the query rectangle in grid units.
    ///   Increase at low zoom to make clicking easier.
    ///
    /// # Returns
    ///
    /// A `HitResult` containing all elements within the tolerance area.
    pub fn hit_test(&self, x: f64, y: f64, tolerance: f64) -> HitResult {
        let query_rect = Rect::new(
            x - tolerance,
            y - tolerance,
            tolerance * 2.0,
            tolerance * 2.0,
        );

        let candidates = self.tree.query(&query_rect);
        let hits = candidates
            .into_iter()
            .map(|item| match &item.data {
                HitData::Node { id, row } => HitElement::Node {
                    id: id.clone(),
                    row: *row,
                },
                HitData::Link {
                    index,
                    source,
                    target,
                    column,
                } => HitElement::Link {
                    column: *column,
                    link_index: *index,
                    source: source.clone(),
                    target: target.clone(),
                },
            })
            .collect();

        HitResult { hits }
    }

    /// Select all elements within a rectangle (for drag-select).
    ///
    /// # Arguments
    ///
    /// * `rect` — Selection rectangle in grid coordinates
    pub fn select_rect(&self, rect: &Rect) -> HitResult {
        let candidates = self.tree.query(rect);
        let hits = candidates
            .into_iter()
            .map(|item| match &item.data {
                HitData::Node { id, row } => HitElement::Node {
                    id: id.clone(),
                    row: *row,
                },
                HitData::Link {
                    index,
                    source,
                    target,
                    column,
                } => HitElement::Link {
                    column: *column,
                    link_index: *index,
                    source: source.clone(),
                    target: target.clone(),
                },
            })
            .collect();

        HitResult { hits }
    }
}
