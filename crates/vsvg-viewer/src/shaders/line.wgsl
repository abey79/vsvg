struct CameraUniform {
    view_proj: mat4x4<f32>,
    scale: f32,
    anti_alias: f32,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;

// Vertex data.
//
// This buffer contains all the line vertices with the starting and ending vertices duplicated (or wrapped, for closed
// lines). The lenght of this array is `instance_count` + 4.
@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;

struct InstanceInput {
    @location(0) color: u32,
    @location(1) width: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // I used to have @interpolate(linear, sample) here, but its not supported by WebGL
    // and I'm not sure it was ever useful.
    @location(0) tex_coords: vec2<f32>,

    @location(1) distance: f32,
    @location(2) @interpolate(flat) color: vec4<f32>,
    @location(3) @interpolate(flat) m0: vec2<f32>,
    @location(4) @interpolate(flat) m2: vec2<f32>,
    @location(5) @interpolate(flat) width: f32,
}


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

    let w2 = instance.width/2. + (camera.anti_alias / camera.scale) / 2.;

    // sliding window over the points buffer
    let p0 = points[in_instance_index + 0];
    let p1 = points[in_instance_index + 1];
    let p2 = points[in_instance_index + 2];
    let p3 = points[in_instance_index + 3];


    let d = distance(p1, p2);
    let v = normalize(p2 - p1);
    let n = vec2<f32>(-v.y, v.x);

    var vertex: vec2<f32>;
    var tex_coords: vec2<f32>;
    switch (in_vertex_index) {
        case 0u: {
            vertex = p1 + w2 * (-v - n);
            tex_coords = vec2<f32>(-w2, -w2);
        }
        case 1u: {
            vertex = p1 + w2 * (-v + n);
            tex_coords = vec2<f32>(-w2, w2);
        }
        case 2u: {
            vertex = p2 + w2 * (v - n);
            tex_coords = vec2<f32>(d + w2, -w2);
        }
        default: { // case 3u
            vertex = p2 + w2 * (v + n);
            tex_coords = vec2<f32>(d + w2, w2);
        }
    }

    // generate output structure
    var out: VertexOutput;
    out.position = camera.view_proj * vec4<f32>(vertex, 0.0, 1.0);
    out.tex_coords = tex_coords;


    // all of this is needed *only* for vertices 0 and 1, thanks to flat shading
    // unfortunately, it breaks on WebGL
    /* if (in_vertex_index < 2u) */
    {
        out.distance = d;

        // compute miter points
        let critical_length_mid = length(p2 - p1 + w2 * n);
        let m0 = compute_miter(p0, p1, n, critical_length_mid, w2);
        let m2 = compute_miter(p2, p3, n, critical_length_mid, w2);
        out.color = color;
        out.width = instance.width;

        // to be useful, the miter points must be rotated in the texture coordinate system
        let rot_mat = transpose(mat2x2<f32>(v, n));
        out.m0 = rot_mat * m0;
        out.m2 = rot_mat * m2;
    }

    return out;
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


    if (any(in.m0 != vec2<f32>(0., 0.))) {
        let side = dot(in.tex_coords, vec2<f32>(-in.m0.y, in.m0.x));

        if (side > 0.) {
            discard;
        }
    }

    if (any(in.m2 !=  vec2<f32>(0., 0.))) {
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
        // this used to be in a separate function, but it was breaking on WebGL, likely because
        // naga somehow doesn't support `discard` in functions
        let w2 = in.width/2.;
        let antialias = (camera.anti_alias/camera.scale) / 2.;
        let color = in.color;

        if (distance < w2 - antialias) {
            return color;
        } else if (distance < w2 + antialias) {
            //var alpha = (distance - w2 - antialias) / antialias;
            let alpha = smoothstep(w2 + antialias, w2 - antialias, distance);
            return vec4<f32>(color.rgb, color.a * alpha);
        } else {
             discard;
        }
    }

    // should never happen, appease Chrome and/or the spec
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
