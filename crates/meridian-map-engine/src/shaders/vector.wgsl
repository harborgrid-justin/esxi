// Vector rendering shaders for points, lines, and polygons

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    zoom: f32,
    _padding: vec3<f32>,
}

struct StyleUniform {
    fill_color: vec4<f32>,
    stroke_color: vec4<f32>,
    stroke_width: f32,
    opacity: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> style: StyleUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct InstanceInput {
    @location(3) transform_0: vec4<f32>,
    @location(4) transform_1: vec4<f32>,
    @location(5) transform_2: vec4<f32>,
    @location(6) transform_3: vec4<f32>,
    @location(7) instance_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_pos: vec2<f32>,
}

// Vertex shader with instancing support
@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;

    // Build instance transform matrix
    let instance_transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );

    // Apply instance transform
    let world_pos = instance_transform * vec4<f32>(vertex.position, 0.0, 1.0);
    output.world_pos = world_pos.xy;

    // Apply camera transform
    output.clip_position = camera.view_proj * world_pos;

    output.tex_coords = vertex.tex_coords;
    output.color = vertex.color * instance.instance_color;

    return output;
}

// Fragment shader for filled polygons
@fragment
fn fs_fill(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = input.color * style.fill_color;
    color.a *= style.opacity;
    return color;
}

// Fragment shader for stroked lines
@fragment
fn fs_stroke(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = input.color * style.stroke_color;
    color.a *= style.opacity;
    return color;
}

// Fragment shader for anti-aliased circles (points)
@fragment
fn fs_circle(input: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate distance from center (tex_coords go from 0 to 1)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(input.tex_coords, center);

    // Anti-aliased circle
    let radius = 0.5;
    let edge_width = 0.05;
    let alpha = smoothstep(radius, radius - edge_width, dist);

    var color = input.color * style.fill_color;
    color.a *= alpha * style.opacity;

    return color;
}

// Fragment shader for anti-aliased lines with rounded caps
@fragment
fn fs_line_rounded(input: VertexOutput) -> @location(0) vec4<f32> {
    // Distance from line center (0.5 in v coordinate)
    let dist = abs(input.tex_coords.y - 0.5) * 2.0;

    // Anti-aliased edge
    let edge_width = 0.05;
    let alpha = smoothstep(1.0, 1.0 - edge_width, dist);

    var color = input.color * style.stroke_color;
    color.a *= alpha * style.opacity;

    return color;
}

// Fragment shader for dashed lines
@fragment
fn fs_line_dashed(input: VertexOutput) -> @location(0) vec4<f32> {
    // Create dash pattern
    let dash_length = 10.0;
    let gap_length = 5.0;
    let pattern_length = dash_length + gap_length;

    let pattern_pos = fract(input.tex_coords.x * pattern_length / dash_length);

    if (pattern_pos > (dash_length / pattern_length)) {
        discard;
    }

    // Distance from line center
    let dist = abs(input.tex_coords.y - 0.5) * 2.0;

    // Anti-aliased edge
    let edge_width = 0.05;
    let alpha = smoothstep(1.0, 1.0 - edge_width, dist);

    var color = input.color * style.stroke_color;
    color.a *= alpha * style.opacity;

    return color;
}

// Fragment shader for polygon outlines
@fragment
fn fs_outline(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = input.color * style.stroke_color;
    color.a *= style.opacity;
    return color;
}
