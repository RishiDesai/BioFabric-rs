/**
 * Shared TypeScript types for BioFabric web frontend.
 *
 * These mirror the Rust types from biofabric-core, deserialized from
 * the JSON returned by the WASM bridge (for metadata) or from raw
 * Float32Array views into WASM memory (for GPU data).
 */

// ---------------------------------------------------------------------------
// GPU instance buffer layout
// ---------------------------------------------------------------------------

/**
 * Number of f32s per line instance.
 * Layout: [x0, y0, x1, y1, r, g, b, a]
 */
export const FLOATS_PER_INSTANCE = 8;

/**
 * Byte stride per instance (8 × 4 = 32 bytes).
 */
export const BYTES_PER_INSTANCE = FLOATS_PER_INSTANCE * 4;

/**
 * Attribute offsets within one instance (in floats, for vertexAttribPointer).
 *
 * Used when configuring the WebGL2 VAO:
 *   - segment: vec4 at offset 0  (x0, y0, x1, y1)
 *   - color:   vec4 at offset 16 (r, g, b, a)
 */
export const ATTR_OFFSET = {
  /** (x0, y0, x1, y1) — line endpoints in grid coordinates. */
  SEGMENT: 0,
  /** (r, g, b, a) — color, normalized [0, 1]. */
  COLOR: 4 * Float32Array.BYTES_PER_ELEMENT,
} as const;

// ---------------------------------------------------------------------------
// Viewport / Camera
// ---------------------------------------------------------------------------

/** A viewport rectangle in BioFabric grid coordinates. */
export interface Viewport {
  x: number;
  y: number;
  width: number;
  height: number;
}

// ---------------------------------------------------------------------------
// Metadata types (returned as JSON from WASM)
// ---------------------------------------------------------------------------

/** Basic network metadata. */
export interface NetworkInfo {
  node_count: number;
  link_count: number;
  relation_types: string[];
}

/** Layout dimensions. */
export interface LayoutDimensions {
  row_count: number;
  column_count: number;
}

/** Layout info for a single node. */
export interface NodeLayoutInfo {
  row: number;
  min_col: number;
  max_col: number;
  name: string;
  color_index: number;
}

/** Layout info for a single link. */
export interface LinkLayoutInfo {
  column: number;
  source_row: number;
  target_row: number;
  source: string;
  target: string;
  relation: string;
  is_shadow: boolean;
  color_index: number;
}

/** Alignment quality scores. */
export interface AlignmentScores {
  ec: number;
  s3: number;
  ics: number;
  nc: number | null;
  ngs: number | null;
  lgs: number | null;
  js: number | null;
}

/** Node info for the info panel. */
export interface NodeInfo {
  id: string;
  degree: number;
  neighbors: string[];
  row: number | null;
  min_col: number | null;
  max_col: number | null;
}
