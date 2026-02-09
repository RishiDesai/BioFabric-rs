//! Quadtree spatial index for efficient hit-testing.
//!
//! Used to quickly determine which node or link is under the mouse cursor
//! in the visualization. Without spatial indexing, checking every element
//! would be O(n) per mouse move — unacceptable for million-element networks.
//!
//! ## References
//!
//! - Java: `org.systemsbiology.biofabric.util.QuadTree`

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Check if this rectangle contains a point.
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x
            && px <= self.x + self.width
            && py >= self.y
            && py <= self.y + self.height
    }

    /// Check if this rectangle intersects another.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// An item stored in the quadtree.
#[derive(Debug, Clone)]
pub struct QuadItem<T> {
    /// Bounding box of this item.
    pub bounds: Rect,
    /// Application-specific data.
    pub data: T,
}

/// A quadtree for spatial indexing.
///
/// Stores axis-aligned bounding boxes and supports efficient range queries.
pub struct QuadTree<T> {
    /// Bounding box of this quadtree node.
    pub bounds: Rect,
    /// Maximum items before splitting.
    pub max_items: usize,
    /// Maximum depth.
    pub max_depth: usize,
    /// Items stored at this node (only in leaf nodes).
    items: Vec<QuadItem<T>>,
    /// Child quadrants (NW, NE, SW, SE). `None` if this is a leaf.
    children: Option<Box<[QuadTree<T>; 4]>>,
    /// Current depth.
    #[allow(dead_code)]
    depth: usize,
}

impl<T> QuadTree<T> {
    /// Create a new quadtree covering the given bounds.
    pub fn new(bounds: Rect, max_items: usize, max_depth: usize) -> Self {
        Self {
            bounds,
            max_items,
            max_depth,
            items: Vec::new(),
            children: None,
            depth: 0,
        }
    }

    /// Insert an item into the quadtree.
    pub fn insert(&mut self, _item: QuadItem<T>) {
        // TODO: Implement quadtree insertion
        //
        // 1. If this is a leaf and under capacity, add to items
        // 2. If this is a leaf and at capacity, split into 4 children
        //    and redistribute items
        // 3. If this is an internal node, insert into the appropriate child
        //    (or into multiple children if the item spans quadrants)
        //
        todo!("Implement quadtree insertion")
    }

    /// Query all items whose bounding boxes intersect the given rectangle.
    pub fn query(&self, _range: &Rect) -> Vec<&QuadItem<T>> {
        // TODO: Implement quadtree range query
        //
        // 1. If range doesn't intersect this node's bounds, return empty
        // 2. If leaf: return items whose bounds intersect range
        // 3. If internal: recursively query children
        //
        todo!("Implement quadtree query")
    }

    /// Total number of items in the quadtree.
    pub fn len(&self) -> usize {
        if let Some(children) = &self.children {
            children.iter().map(|c| c.len()).sum()
        } else {
            self.items.len()
        }
    }

    /// Whether the quadtree is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
