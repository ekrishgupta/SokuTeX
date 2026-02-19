struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(in_vertex_index) & 1);
    let y = f32(i32(in_vertex_index) >> 1);
    let uv = vec2<f32>(x * 2.0, y * 2.0); // 0,0  2,0  0,2  (covering -1,-1 to 3,3 in screen space clips to full screen?)
    // Actually standard big triangle:
    // 0: (-1, -1), uv (0, 1)
    // 1: (3, -1), uv (2, 1)
    // 2: (-1, 3), uv (0, -1)
    // This covers screen.
    // Wait, let's just use explicit vertices or simpler math.
    
    // Simpler: Just rely on hardcoded vec2 array if needed, but let's try standard approach.
    // Full screen quad using 3 vertices:
    // ( -1, 3) (2)
    // |      \
    // ( -1, -1) (0) -- ( 3, -1) (1)
    // UVs follow.
    
    let u = f32((in_vertex_index << 1u) & 2u);
    let v = f32(in_vertex_index & 2u);
    out.tex_coords = vec2<f32>(u, v);
    out.clip_position = vec4<f32>(u * 2.0 - 1.0, 1.0 - v * 2.0, 0.0, 1.0); // Y is flipped in WGPU clip space? WGPU Y is up? No, WGPU Y is up -1 to 1. UV 0,0 is top-left usually for textures.
    // Let's assume standard: UV (0,0) is top-left.
    // WGPU NDCs: (-1, -1) is bottom-left. (1, 1) is top-right.
    
    out.clip_position = vec4<f32>(u * 2.0 - 1.0, 1.0 - v * 2.0, 0.0, 1.0);
    // Vertex 0: u=0, v=0 -> (-1, 1, ..) Top-Left
    // Vertex 1: u=2, v=0 -> (3, 1, ..) Top-Right-Far
    // Vertex 2: u=0, v=2 -> (-1, -3, ..) Bottom-Left-Far
    
    // Seems correct for covering screen.
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
