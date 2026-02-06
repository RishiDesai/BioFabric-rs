/**
 * WebGL2 BioFabric Renderer — GPU-instanced line drawing.
 *
 * ## Approach
 *
 * Each line (node or link) is one GPU **instance** described by 8 floats:
 * `[x0, y0, x1, y1, r, g, b, a]`.
 *
 * A single shader program handles both horizontal (node) and vertical (link)
 * lines. The vertex shader expands each instance into a screen-aligned quad
 * of configurable thickness.
 *
 * Per frame:
 *   1. WASM extracts visible instances → flat Float32Array in WASM memory
 *   2. We upload the Float32Array to a WebGL2 instance VBO (one for nodes,
 *      one for links)
 *   3. Two instanced draw calls: links first (behind), then nodes (on top)
 *
 * ## Why not separate shaders for nodes vs links?
 *
 * The shader logic is identical — both are just lines. The only difference
 * is the line width uniform, which we set between the two draw calls.
 *
 * ## Performance
 *
 * - 2M instances × 32 bytes = 64 MB of GPU memory.
 *   `bufferData` with `DYNAMIC_DRAW` handles this fine.
 * - Instanced rendering: 1 draw call = 1M+ lines at 60 fps on any
 *   GPU made after ~2015.
 * - Viewport culling in WASM means we typically upload far fewer instances
 *   than the total network size.
 */

import {
  FLOATS_PER_INSTANCE,
  BYTES_PER_INSTANCE,
  ATTR_OFFSET,
  type Viewport,
} from "../types";
import type { BioFabricWasm, Handle } from "../wasm/bridge";

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/** Camera state for pan/zoom. */
export interface Camera {
  /** Center X in grid coordinates. */
  centerX: number;
  /** Center Y in grid coordinates. */
  centerY: number;
  /** Zoom level (screen pixels per grid unit). */
  zoom: number;
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

export class FabricRenderer {
  private _canvas: HTMLCanvasElement;
  private _gl: WebGL2RenderingContext | null = null;
  private _camera: Camera = { centerX: 0, centerY: 0, zoom: 4 };

  // Shader program (shared by nodes and links)
  private _program: WebGLProgram | null = null;

  // VAO + VBO for link instances
  private _linkVao: WebGLVertexArrayObject | null = null;
  private _linkVbo: WebGLBuffer | null = null;
  private _linkCount = 0;

  // VAO + VBO for node instances
  private _nodeVao: WebGLVertexArrayObject | null = null;
  private _nodeVbo: WebGLBuffer | null = null;
  private _nodeCount = 0;

  // Unit quad VBO (shared — 6 vertices for a [-0.5, 0.5] quad)
  private _quadVbo: WebGLBuffer | null = null;

  // Uniform locations
  private _uCamera: WebGLUniformLocation | null = null;
  private _uLineWidth: WebGLUniformLocation | null = null;
  private _uResolution: WebGLUniformLocation | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this._canvas = canvas;

    // TODO: Initialize WebGL2
    //
    // 1. canvas.getContext("webgl2", { antialias: true, alpha: false })
    //
    // 2. Compile the line shader (see shaders/line.vert, line.frag)
    //    - Vertex shader: takes per-instance (segment, color) + per-vertex
    //      (corner) data; outputs screen-space quad
    //    - Fragment shader: outputs v_color
    //
    // 3. Create the unit quad VBO (6 vertices: two triangles forming a quad)
    //    [-0.5, -0.5], [0.5, -0.5], [-0.5, 0.5],
    //    [-0.5, 0.5],  [0.5, -0.5], [0.5, 0.5]
    //
    // 4. Create node VAO + VBO, link VAO + VBO
    //    For each:
    //    a. Bind VAO
    //    b. Bind quad VBO → attribute 0 (vec2 corner), divisor = 0
    //    c. Bind instance VBO → attribute 1 (vec4 segment, divisor = 1)
    //                         → attribute 2 (vec4 color, divisor = 1)
    //    d. The instance VBO starts empty; we'll upload data per frame
    //
    // 5. Enable blending: gl.blendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA)
    //
    // 6. Observe canvas resize via ResizeObserver → call resize()
    //
  }

  /** Current camera state. */
  get camera(): Camera {
    return this._camera;
  }

  /** Compute the current viewport in grid coordinates. */
  get viewport(): Viewport {
    const w = this._canvas.width / (this._camera.zoom * devicePixelRatio);
    const h = this._canvas.height / (this._camera.zoom * devicePixelRatio);
    return {
      x: this._camera.centerX - w / 2,
      y: this._camera.centerY - h / 2,
      width: w,
      height: h,
    };
  }

  /** Screen pixels per grid unit (the value WASM needs). */
  get pixelsPerUnit(): number {
    return this._camera.zoom * devicePixelRatio;
  }

  // -----------------------------------------------------------------------
  // Data upload
  // -----------------------------------------------------------------------

  /**
   * Upload node instance data from a Float32Array (typically a zero-copy
   * view into WASM memory).
   */
  uploadNodeInstances(data: Float32Array): void {
    // TODO: Implement GPU upload
    //
    // 1. Bind _nodeVbo
    // 2. gl.bufferData(ARRAY_BUFFER, data, DYNAMIC_DRAW)
    // 3. _nodeCount = data.length / FLOATS_PER_INSTANCE
    //
  }

  /**
   * Upload link instance data.
   */
  uploadLinkInstances(data: Float32Array): void {
    // Same as uploadNodeInstances but for _linkVbo
  }

  // -----------------------------------------------------------------------
  // Render loop
  // -----------------------------------------------------------------------

  /**
   * Draw one frame.
   */
  draw(): void {
    // TODO: Implement draw
    //
    // 1. Clear (dark background)
    // 2. Set u_camera and u_resolution uniforms
    // 3. Draw links: bind _linkVao, set u_lineWidth to ~1.0px,
    //    gl.drawArraysInstanced(TRIANGLES, 0, 6, _linkCount)
    // 4. Draw nodes: bind _nodeVao, set u_lineWidth to ~2.0px,
    //    gl.drawArraysInstanced(TRIANGLES, 0, 6, _nodeCount)
    //
  }

  /**
   * Full render cycle: extract from WASM, upload, draw.
   */
  renderFrame(wasm: BioFabricWasm, layoutHandle: Handle): void {
    // TODO: Implement full render cycle
    //
    // 1. const vp = this.viewport;
    // 2. wasm.extractRenderData(layoutHandle,
    //        vp.x, vp.y, vp.width, vp.height,
    //        this.pixelsPerUnit,
    //        this._canvas.width, this._canvas.height);
    // 3. this.uploadLinkInstances(wasm.getLinkInstances());
    // 4. this.uploadNodeInstances(wasm.getNodeInstances());
    // 5. this.draw();
    //
  }

  // -----------------------------------------------------------------------
  // Camera controls
  // -----------------------------------------------------------------------

  /** Pan the camera by a screen-space delta (pixels). */
  pan(dx: number, dy: number): void {
    this._camera.centerX -= dx / this.pixelsPerUnit;
    this._camera.centerY -= dy / this.pixelsPerUnit;
  }

  /** Zoom the camera around a screen point. */
  zoomAt(factor: number, screenX: number, screenY: number): void {
    // Zoom toward the point under the cursor
    const before = this.screenToGrid(screenX, screenY);
    this._camera.zoom = Math.max(0.001, Math.min(10000, this._camera.zoom * factor));
    const after = this.screenToGrid(screenX, screenY);
    this._camera.centerX += before.x - after.x;
    this._camera.centerY += before.y - after.y;
  }

  /** Convert screen coordinates (CSS pixels) to grid coordinates. */
  screenToGrid(screenX: number, screenY: number): { x: number; y: number } {
    const rect = this._canvas.getBoundingClientRect();
    const canvasX = (screenX - rect.left) * devicePixelRatio;
    const canvasY = (screenY - rect.top) * devicePixelRatio;
    return {
      x: this._camera.centerX + (canvasX - this._canvas.width / 2) / this.pixelsPerUnit,
      y: this._camera.centerY + (canvasY - this._canvas.height / 2) / this.pixelsPerUnit,
    };
  }

  // -----------------------------------------------------------------------
  // Lifecycle
  // -----------------------------------------------------------------------

  /** Resize the canvas to fill its container. */
  resize(): void {
    const rect = this._canvas.parentElement!.getBoundingClientRect();
    this._canvas.width = rect.width * devicePixelRatio;
    this._canvas.height = rect.height * devicePixelRatio;
    this._gl?.viewport(0, 0, this._canvas.width, this._canvas.height);
  }

  /** Clean up all WebGL resources. */
  destroy(): void {
    // TODO: Delete program, VAOs, VBOs
  }
}
