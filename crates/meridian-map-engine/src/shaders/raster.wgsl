// Raster tile rendering shaders

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    zoom: f32,
    _padding: vec3<f32>,
}

struct RasterUniform {
    opacity: f32,
    brightness: f32,
    contrast: f32,
    saturation: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_tile: texture_2d<f32>;
@group(1) @binding(1)
var s_tile: sampler;

@group(2) @binding(0)
var<uniform> raster: RasterUniform;

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

// Basic tile rendering
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_tile, s_tile, input.tex_coords);
    color.a *= raster.opacity;
    return color;
}

// Tile rendering with color adjustments
@fragment
fn fs_adjusted(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_tile, s_tile, input.tex_coords);

    // Apply brightness
    color.r += raster.brightness;
    color.g += raster.brightness;
    color.b += raster.brightness;

    // Apply contrast
    color.r = (color.r - 0.5) * raster.contrast + 0.5;
    color.g = (color.g - 0.5) * raster.contrast + 0.5;
    color.b = (color.b - 0.5) * raster.contrast + 0.5;

    // Apply saturation
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color.r = mix(gray, color.r, raster.saturation);
    color.g = mix(gray, color.g, raster.saturation);
    color.b = mix(gray, color.b, raster.saturation);

    // Apply opacity
    color.a *= raster.opacity;

    return color;
}

// Hillshade rendering for elevation tiles
@fragment
fn fs_hillshade(input: VertexOutput) -> @location(0) vec4<f32> {
    let elevation = textureSample(t_tile, s_tile, input.tex_coords).r;

    // Calculate hillshade based on elevation
    // This is a simplified version - production would use proper normal calculation
    let light_direction = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = vec3<f32>(0.0, 0.0, 1.0); // Simplified - would calculate from elevation

    let diffuse = max(dot(normal, light_direction), 0.0);
    let ambient = 0.3;
    let shade = min(diffuse + ambient, 1.0);

    var color = vec4<f32>(shade, shade, shade, 1.0);
    color.a *= raster.opacity;

    return color;
}

// Heat map rendering
@fragment
fn fs_heatmap(input: VertexOutput) -> @location(0) vec4<f32> {
    let value = textureSample(t_tile, s_tile, input.tex_coords).r;

    // Heat map gradient (blue -> cyan -> green -> yellow -> red)
    var color: vec4<f32>;

    if (value < 0.25) {
        // Blue to cyan
        let t = value / 0.25;
        color = mix(vec4<f32>(0.0, 0.0, 1.0, 1.0), vec4<f32>(0.0, 1.0, 1.0, 1.0), t);
    } else if (value < 0.5) {
        // Cyan to green
        let t = (value - 0.25) / 0.25;
        color = mix(vec4<f32>(0.0, 1.0, 1.0, 1.0), vec4<f32>(0.0, 1.0, 0.0, 1.0), t);
    } else if (value < 0.75) {
        // Green to yellow
        let t = (value - 0.5) / 0.25;
        color = mix(vec4<f32>(0.0, 1.0, 0.0, 1.0), vec4<f32>(1.0, 1.0, 0.0, 1.0), t);
    } else {
        // Yellow to red
        let t = (value - 0.75) / 0.25;
        color = mix(vec4<f32>(1.0, 1.0, 0.0, 1.0), vec4<f32>(1.0, 0.0, 0.0, 1.0), t);
    }

    color.a *= raster.opacity;
    return color;
}

// Tile blending for smooth transitions between zoom levels
@fragment
fn fs_blend(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_tile, s_tile, input.tex_coords);

    // Fade at tile edges for seamless blending
    let edge_width = 0.02;
    let edge_x = min(input.tex_coords.x, 1.0 - input.tex_coords.x);
    let edge_y = min(input.tex_coords.y, 1.0 - input.tex_coords.y);
    let edge_alpha = smoothstep(0.0, edge_width, min(edge_x, edge_y));

    color.a *= edge_alpha * raster.opacity;
    return color;
}

// Sepia tone filter
@fragment
fn fs_sepia(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_tile, s_tile, input.tex_coords);

    let sepia_r = dot(color.rgb, vec3<f32>(0.393, 0.769, 0.189));
    let sepia_g = dot(color.rgb, vec3<f32>(0.349, 0.686, 0.168));
    let sepia_b = dot(color.rgb, vec3<f32>(0.272, 0.534, 0.131));

    color = vec4<f32>(sepia_r, sepia_g, sepia_b, color.a);
    color.a *= raster.opacity;

    return color;
}

// Grayscale filter
@fragment
fn fs_grayscale(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_tile, s_tile, input.tex_coords);

    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color = vec4<f32>(gray, gray, gray, color.a);
    color.a *= raster.opacity;

    return color;
}
