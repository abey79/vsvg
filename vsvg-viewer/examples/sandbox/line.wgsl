struct CameraUniform {
    view_proj: mat4x4<f32>,
    scale: f32,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) p0: vec2<f32>,
    @location(1) p1: vec2<f32>,
    @location(2) p2: vec2<f32>,
    @location(3) p3: vec2<f32>,
    @location(4) color: u32,
    @location(5) width: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(linear, sample) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) distance: f32,
    @location(2) @interpolate(flat) color: vec4<f32>,
    @location(3) @interpolate(flat) m0: vec2<f32>,
    @location(4) @interpolate(flat) m2: vec2<f32>,
    @location(5) @interpolate(flat) width: f32,
}


const AA: f32 = 1.5;


fn compute_miter(p0: vec2<f32>, p1: vec2<f32>, n: vec2<f32>, crit_length: f32, w2: f32) -> vec2<f32> {
    if (all(p0 == p1)) {
        return vec2<f32>(0.0, 0.0);
    }

    let dp = p1 - p0;
    let v0 = normalize(dp);
    let n0 = vec2<f32>(-v0.y, v0.x);
    let m0 = normalize(n0 + n);
    let len_m0 = w2 / dot(m0, n);

    let critical_length_0 = length(dp + w2 * n0);
    if (len_m0 < min(crit_length, critical_length_0))
    {
        return m0;
    }
    else
    {
        return vec2<f32>(0.0, 0.0);
    }
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
    @builtin(vertex_index) in_vertex_index: u32,
    @builtin(instance_index) in_instance_index: u32,
    instance: InstanceInput
) -> VertexOutput {
    let color = unpack_color(instance.color);

    // discard anything that has transpartent color
    if (color.a == 0.) {
        var out = VertexOutput();
        // such value of z will be clipped
        out.position = vec4<f32>(0.0, 0.0, 10.0, 1.0);
        return out;
    }

    let w2 = instance.width/2. + AA / camera.scale;

    let d = distance(instance.p1, instance.p2);
    let v = normalize(instance.p2 - instance.p1);
    let n = vec2<f32>(-v.y, v.x);

    var vertex: vec2<f32>;
    var tex_coords: vec2<f32>;
    switch (in_vertex_index) {
        case 0u: {
            vertex = instance.p1 + w2 * (-v - n);
            tex_coords = vec2<f32>(-w2, -w2);
        }
        case 1u: {
            vertex = instance.p1 + w2 * (-v + n);
            tex_coords = vec2<f32>(-w2, w2);
        }
        case 2u: {
            vertex = instance.p2 + w2 * (v - n);
            tex_coords = vec2<f32>(d + w2, -w2);
        }
        default: { // case 3u
            vertex = instance.p2 + w2 * (v + n);
            tex_coords = vec2<f32>(d + w2, w2);
        }
    }

    // generate output structure
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(vertex, 0.0, 1.0);
    out.tex_coords = tex_coords;


    // all of this is needed *only* for vertices 0 and 1, thanks to flat shading
    if (in_vertex_index < 2u) {
        out.distance = d;

        // compute miter points
        let critical_length_mid = length(instance.p2 - instance.p1 + w2 * n);
        let m0 = compute_miter(instance.p0, instance.p1, n, critical_length_mid, w2);
        let m2 = compute_miter(instance.p2, instance.p3, n, critical_length_mid, w2);
        out.color = color;
        out.width = instance.width;

        // to be useful, the miter points must be rotated in the texture coordinate system
        let rot_mat = transpose(mat2x2<f32>(v, n));
        out.m0 = rot_mat * m0;
        out.m2 = rot_mat * m2;
    }
    return out;
}


fn stroke(distance: f32, w2: f32, antialias: f32, color: vec4<f32>) -> vec4<f32> {
    if (distance < w2) {
        return color;
    } else if (distance < w2 + antialias) {
        var alpha = (distance - w2) / antialias;
        alpha = exp(-alpha*alpha);
        return vec4<f32>(color.rgb, color.a * alpha);
    } else {
        discard;
    }
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var distance = abs(in.tex_coords.y);

    // draw miter points
    if (false)
    {
        let dm0 = length(in.tex_coords - in.width/2.*in.m0);
        if (dm0 < 2.) {
            return vec4<f32>(0.2, 0.1, 0.1, 1.0);
        }

        let dm2 = length(in.tex_coords - vec2<f32>(in.distance, 0.) - in.width/2.*in.m2);
        if (2. < dm2 && dm2 < 4.) {
            return vec4<f32>(0.1, 0.1, 0.3, 1.0);
        }
    }


    if (any(in.m0 != 0.)) {
        let side = dot(in.tex_coords, vec2<f32>(-in.m0.y, in.m0.x));

        if (side > 0.) {
            discard;
        }
    }

    if (any(in.m2 != 0.)) {
        let side = dot(in.tex_coords - vec2<f32>(in.distance, 0.), vec2<f32>(-in.m2.y, in.m2.x));

        if (side <= 0. ) {
            discard;
        }
    }


    if (in.tex_coords.x < 0.0) {
        distance = length(in.tex_coords);
    } else if (in.tex_coords.x > in.distance) {
        distance = length(in.tex_coords - vec2<f32>(in.distance, 0.0));
    }

    if (false) {
        return vec4<f32>(1.0 - distance / in.width * 2., 0.0, 0.0, 1.0);
    } else {
        return stroke(distance, in.width/2., AA/camera.scale, in.color);
    }
}
