struct CameraUniform {
    view_proj: mat4x4<f32>,
    scale: f32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) point: vec2<f32>,
    @location(1) color: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec4<f32>,
}

fn unpack_color(color: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(color & 255u),
        f32((color >> 8u) & 255u),
        f32((color >> 16u) & 255u),
        f32((color >> 24u) & 255u),
    ) / 255.0;
}

@vertex
fn vs_main(
    in_vertex: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(in_vertex.point, 0.0, 1.0);
    out.color = unpack_color(in_vertex.color);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
