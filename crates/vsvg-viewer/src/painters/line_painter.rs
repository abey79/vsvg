//! Scale-aware line painter with arbitrary color and width.
//!
//! ### Design goal
//!
//! We want to render many (1+ million) poly-lines each with an arbitrary number of segments, for
//! a total of 10+ million vertices.
//!
//! Since the line count may be very large (e.g. if each has a single segment), we must draw all
//! lines in a single draw call. This means that our buffer must somehow encode end-of-line
//! information.
//!
//! Lines ending with a vertex that is equal to the first one must be drawn as a closed shape.
//!
//!
//! ### Shader input
//!
//! The VS is triggered 4 times per instance, to generate a triangle strip for each line segment.
//! Each line segment corresponds to an instance.
//!
//! For each instance, the shader expects the following data:
//! - 4 points (`p0`, `p1`, `p2`, and `p3`)
//! - color and width attributes
//!
//! The point corresponds to:
//! - `p0`: the previous point
//! - `p1`: the starting point of the segment
//! - `p2`: the ending point of the segment
//! - `p3`: the point after the segment
//!
//! If `p0` and `p1` are equal, this means that this segment is the first of a line. Likewise, if
//! `p2` and `p3` are equal, this means that this segment is the last of a line.
//!
//! With this information, the VS can compute the exact portion of the start/end capsule to be drawn
//!  and send that information to the FS.
//!
//!
//! ### Buffer preparation
//!
//! Consider the following example comprising three lines A, B, and C. A has one segment, B has
//! three segments, and C has three segments and is closed.
//!
//! ```text
//!       1 segment                3 segments                   3 segments,
//!                                                             closed line
//!
//!                                                               C0, C3
//! A0 ────────────── A1      B0 ───────┐B1                         ╱╲
//!                                     │                          ╱  ╲
//!                                   B2└────────B3               ╱    ╲
//!                                                             C2──────C1
//!
//! points buffer       │                             │
//! ┌────┬────┬────┬────┼────┬────┬────┬────┬────┬────┼────┬────┬────┬────┬────┬────┐
//! │ A0 │ A0 │ A1 │ A1 │ B0 │ B0 │ B1 │ B2 │ B3 │ B3 │ C2 │ C0 │ C1 │ C2 │ C3 │ C1 │
//! └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘
//!           │                   │                             │                   │
//!           └────────┬──────────┘                             └────────┬──────────┘
//!      │             │     │                                           │
//!      └────────┬────┼─────┘                ....                       │
//! │             │    ││                                                │
//! └────────┬────┼────┼┘                                                │
//!          │    │    │                                                 │
//!          ▼    ▼    ▼                                                 ▼
//!        ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐
//!        │ A0 │    │    │    │ B0 │ B1 │ B2 │    │    │    │ C0 │ C1 │ C2 │
//!        └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘
//!        attributes buffer (one per segment + padding)
//!```
//!
//! We construct the point buffer line-by-line, with each line consisting of:
//! - The sequence of points defining its segments.
//! - Repeating the first and last point.
//! - For closed shape, we pad the point list with the last and 2nd points instead, such that the
//!   shader is always presented with 4 sequential points.
//!
//! Finally, we concatenate all these buffers together in one large point buffer.
//!
//! The key observation is that, for such a buffer topology, a sliding window of 4 points exactly
//! corresponds with the required shader input, _except_ when the window spans two lines.
//!
//! To address that, we create a second buffer where each item is the attributes (color and width)
//! for a single segment. Between each line, we insert three "empty" attributes (actually fully
//! transparent color). The shader uses these empty attributes to detect and ignore point
//! quadruplets which span two lines.
//!
//! The length of the attributes buffer (including the empty attributes) corresponds to the number
//! of instances passed to the draw call.
//!
//!
//! ### Buffer binding
//!
//! Given the above, the "naive" approach would be to bind the point buffer with a stride of one
//! vertex but passing four vertices at a time. This used to work but, it turns out, is not
//! compliant with the [WebGPU spec](https://gpuweb.github.io/gpuweb/#dictdef-gpuvertexbufferlayout).
//! As such, it is rejected by (recent versions of) Chrome and wgpu/metal since (23.0.0).
//!
//! Instead, we bind points as a read-only storage buffer and directly look up vertex data in that
//! buffer.
//!
//! Note: this is not compatible with WebGL.

use wgpu::{
    include_wgsl, util::DeviceExt, vertex_attr_array, Buffer, ColorTargetState, PrimitiveTopology,
    RenderPass, RenderPipeline,
};

use vsvg::{FlattenedPath, PathTrait};

use crate::engine::EngineRenderObjects;
use crate::painters::{Painter, Vertex};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Attribute {
    color: u32,
    width: f32,
}

impl Attribute {
    const fn empty() -> Self {
        Self {
            color: 0,
            width: 1.0,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LineDisplayOptions {
    /// override width
    pub override_width: Option<f32>,

    /// override opacity
    pub override_opacity: Option<f32>,
}

pub(crate) struct LinePainterData {
    points_buffer: Buffer,
    attributes_buffer: Buffer,
    instance_count: u32,
}

impl LinePainterData {
    pub fn new<'b, I>(
        render_objects: &EngineRenderObjects,
        paths: I,
        display_options: &LineDisplayOptions,
    ) -> Self
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        vsvg::trace_function!();

        let (vertices, attribs) = Self::build_buffers(paths, display_options);

        // prepare point buffer – used as a storage binding!
        let points_buffer =
            render_objects
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Point instance buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        // prepare color buffer
        let attributes_buffer =
            render_objects
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Attributes instance Buffer"),
                    contents: bytemuck::cast_slice(attribs.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        #[allow(clippy::cast_possible_truncation)]
        Self {
            points_buffer,
            attributes_buffer,
            instance_count: attribs.len() as u32,
        }
    }

    fn build_buffers<'b, I>(
        paths: I,
        display_options: &LineDisplayOptions,
    ) -> (Vec<Vertex>, Vec<Attribute>)
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        fn add_path(
            path: &FlattenedPath,
            vertices: &mut Vec<Vertex>,
            attribs: &mut Vec<Attribute>,
            display_options: &LineDisplayOptions,
        ) {
            vsvg::trace_function!();

            let points = path.data().points();
            if points.len() > 1 {
                if points.len() > 2 && points.first() == points.last() {
                    vertices.push(points[points.len() - 2].into());
                    vertices.extend(points.iter().map(Vertex::from));
                    vertices.push(points[1].into());
                } else {
                    vertices.push(points.first().expect("length checked").into());
                    vertices.extend(points.iter().map(Vertex::from));
                    vertices.push(points.last().expect("length checked").into());
                }

                let mut color = path.metadata().color.to_rgba();
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                if let Some(opacity) = display_options.override_opacity {
                    color = (color & 0x00_FF_FF_FF)
                        + (((opacity * 255.0).clamp(0., 255.) as u32) << 24);
                }

                #[allow(clippy::cast_possible_truncation)]
                let attr = Attribute {
                    color,
                    width: display_options
                        .override_width
                        .unwrap_or(path.metadata().stroke_width as f32),
                };

                for _ in 0..path.data().points().len() - 1 {
                    attribs.push(attr);
                }
            }
        }

        vsvg::trace_function!();

        let mut iter = paths.into_iter();
        let min_size = 1000.min(iter.size_hint().0 * 4);

        // build the data buffers
        let mut vertices: Vec<Vertex> = Vec::with_capacity(min_size);
        let mut attribs = Vec::with_capacity(min_size);

        if let Some(path) = iter.next() {
            add_path(path, &mut vertices, &mut attribs, display_options);
            for path in iter {
                attribs.push(Attribute::empty());
                attribs.push(Attribute::empty());
                attribs.push(Attribute::empty());
                add_path(path, &mut vertices, &mut attribs, display_options);
            }
        }

        (vertices, attribs)
    }
}

/// Renders paths as scale-aware lines with variable width and color.
///
/// See module documentation for details.
pub(crate) struct LinePainter {
    points_bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: RenderPipeline,
}

impl LinePainter {
    pub(crate) fn new(render_objects: &EngineRenderObjects) -> Self {
        let vertex_attrib_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<Attribute>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Uint32,
                1 => Float32,
            ],
        };

        let points_bind_group_layout =
            render_objects
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("point_bind_group_layout"),
                });

        let shader = render_objects
            .device
            .create_shader_module(include_wgsl!("../shaders/line.wgsl"));

        let pipeline_layout =
            render_objects
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &render_objects.camera_bind_group_layout,
                        &points_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        // enable alpha blending
        let target = ColorTargetState {
            format: render_objects.target_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::DstAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline =
            render_objects
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("line pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[vertex_attrib_buffer_layout],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(target)],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: PrimitiveTopology::TriangleStrip,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

        /////////

        Self {
            points_bind_group_layout,
            render_pipeline,
        }
    }
}

impl Painter for LinePainter {
    type Data = LinePainterData;

    fn draw(
        &self,
        rpass: &mut RenderPass<'static>,
        render_objects: &EngineRenderObjects,
        data: &LinePainterData,
    ) {
        // This also avoids the fact that `Buffer::slice(..)` panics for empty buffers in wgpu 23+
        if data.instance_count == 0 {
            return;
        }

        let points_bind_group =
            render_objects
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.points_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: data.points_buffer.as_entire_binding(),
                    }],
                    label: Some("point_bind_group"),
                });

        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &render_objects.camera_bind_group, &[]);
        rpass.set_bind_group(1, &points_bind_group, &[]);
        rpass.set_vertex_buffer(0, data.attributes_buffer.slice(..));

        rpass.draw(0..4, 0..data.instance_count);
    }
}
