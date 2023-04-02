struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) position: vec3<f32>,
};


@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @builtin(instance_index) in_instance_index: u32,
    instance: InstanceInput
) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1) * 100.;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 100.;

    return camera.view_proj * vec4<f32>(x + instance.position.x, y + instance.position.y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.5, 0.2, 0.3, 1.0);
}
