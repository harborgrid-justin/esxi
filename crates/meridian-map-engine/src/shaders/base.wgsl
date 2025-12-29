// Base vertex and fragment shaders for 2D rendering

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    zoom: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Transform position to clip space
    let world_pos = vec4<f32>(input.position, 0.0, 1.0);
    output.clip_position = camera.view_proj * world_pos;

    output.tex_coords = input.tex_coords;
    output.color = input.color;

    return output;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture and multiply by vertex color
    let tex_color = textureSample(t_diffuse, s_diffuse, input.tex_coords);
    return tex_color * input.color;
}

// Shader variant without texture
@fragment
fn fs_main_no_texture(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
