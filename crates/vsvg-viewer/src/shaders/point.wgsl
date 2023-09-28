struct CameraUniform {
    view_proj: mat4x4<f32>,
    scale: f32,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) position: vec2<f32>,
    @location(1) color: u32,
    @location(2) size: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // I used to have @interpolate(linear, sample) here, but its not supported by WebGL
    // and I'm not sure it was ever useful.
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
    @location(2) @interpolate(flat) w2: f32,
}


const AA: f32 = 1.5;

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
    @builtin(vertex_index) in_vertex_index: u32,
    instance: InstanceInput
) -> VertexOutput {
    let w2 = (instance.size / 2.0 + AA) / camera.scale;

    // compute the texture coordinates
    var tex_coords: vec2<f32>;
    if ((in_vertex_index & 1u) != 0u) {
        tex_coords.y = w2;
    } else {
        tex_coords.y = -w2;
    }
    if ((in_vertex_index & 2u) != 0u) {
        tex_coords.x = w2;
    } else {
        tex_coords.x = -w2;
    }

    // generate output structure
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(instance.position + tex_coords, 0.0, 1.0);;
    out.tex_coords = tex_coords;
    out.color = unpack_color(instance.color);
    out.w2 = instance.size / 2.0 / camera.scale;

    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = length(in.tex_coords);
    let aa = AA / camera.scale;

    if (distance < in.w2) {
        return in.color;
    } else if (distance < in.w2 + aa) {
        var alpha = (distance - in.w2) / aa;
        alpha = smoothstep(1.0, 0.0, alpha);
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    } else {
        discard;
    }
}
