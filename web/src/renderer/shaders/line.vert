#version 300 es
//
// Instanced line vertex shader for BioFabric.
//
// Draws both horizontal (node) and vertical (link) lines. Each instance
// is a line segment defined by 8 floats: (x0, y0, x1, y1, r, g, b, a).
//
// The shader expands each line into a screen-aligned quad of configurable
// pixel thickness using the unit-quad corner attribute.
//
// Attributes:
//   layout(0) a_corner   — per-vertex, 6 verts forming a unit quad ([-0.5,0.5])
//   layout(1) a_segment  — per-instance (divisor=1), vec4 (x0, y0, x1, y1)
//   layout(2) a_color    — per-instance (divisor=1), vec4 (r, g, b, a)
//
// Uniforms:
//   u_camera     — vec4 (centerX, centerY, zoom, _unused)
//   u_resolution — vec2 (canvas width, canvas height) in physical pixels
//   u_lineWidth  — line thickness in pixels

// Per-vertex: corner of the unit quad
layout(location = 0) in vec2 a_corner;

// Per-instance (divisor = 1)
layout(location = 1) in vec4 a_segment;  // (x0, y0, x1, y1) grid coords
layout(location = 2) in vec4 a_color;    // (r, g, b, a) [0,1]

uniform vec4 u_camera;      // (centerX, centerY, zoom, 0)
uniform vec2 u_resolution;  // (canvasWidth, canvasHeight)
uniform float u_lineWidth;  // px

out vec4 v_color;

void main() {
    // TODO: Implement line quad expansion
    //
    // 1. Compute line direction and perpendicular:
    //      vec2 p0 = a_segment.xy;
    //      vec2 p1 = a_segment.zw;
    //      vec2 dir = normalize(p1 - p0);
    //      vec2 perp = vec2(-dir.y, dir.x);
    //
    // 2. Expand corner vertex along direction and perpendicular:
    //      vec2 pos = mix(p0, p1, a_corner.x + 0.5);   // along line
    //      pos += perp * a_corner.y * halfWidthInGrid;  // perpendicular
    //
    // 3. Apply camera transform (grid → clip space):
    //      pos = (pos - u_camera.xy) * u_camera.z;
    //      gl_Position = vec4(pos / (u_resolution * 0.5), 0.0, 1.0);
    //
    // Note: halfWidthInGrid = u_lineWidth / (2.0 * u_camera.z)
    //       so the line is always `u_lineWidth` pixels thick regardless of zoom.
    //

    v_color = a_color;
    gl_Position = vec4(0.0, 0.0, 0.0, 1.0);  // placeholder
}
