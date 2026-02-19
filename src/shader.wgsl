struct PdfUniforms {
    transform: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> uniforms: PdfUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // CCW Fullscreen Triangle
    // 0: (-1, -1), UV (0, 1)
    // 1: ( 3, -1), UV (2, 1)
    // 2: (-1,  3), UV (0, -1)
    
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    
    // Apply zoom/pan transform to the base full-screen triangle
    let raw_pos = vec4<f32>(x, y, 0.0, 1.0);
    out.clip_position = uniforms.transform * raw_pos;
    out.tex_coords = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(t_diffuse, s_diffuse, in.tex_coords).rgb, 1.0);
}
