use crate::engine::EngineRenderObjects;
use crate::painters::{Painter, Vertex};
use std::mem;
use vsvg::{FlattenedPath, PathTrait};
use wgpu::util::DeviceExt;
use wgpu::{
    include_wgsl, vertex_attr_array, Buffer, ColorTargetState, PrimitiveTopology, RenderPass,
    RenderPipeline,
};

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

pub(crate) struct LinePainterData {
    points_buffer: Buffer,
    attributes_buffer: Buffer,
    instance_count: u32,
}

impl LinePainterData {
    pub fn new<'b, I>(render_objects: &EngineRenderObjects, paths: I) -> Self
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        vsvg::trace_function!();

        let (vertices, attribs) = Self::build_buffers(paths);

        // prepare point buffer
        let points_buffer =
            render_objects
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Point instance buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
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

        Self {
            points_buffer,
            attributes_buffer,
            #[allow(clippy::cast_possible_truncation)]
            instance_count: attribs.len() as u32,
        }
    }

    fn build_buffers<'b, I>(paths: I) -> (Vec<Vertex>, Vec<Attribute>)
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        fn add_path(
            path: &FlattenedPath,
            vertices: &mut Vec<Vertex>,
            attribs: &mut Vec<Attribute>,
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

                let attr = Attribute {
                    color: path.metadata().color.to_rgba(),
                    #[allow(clippy::cast_possible_truncation)]
                    width: path.metadata().stroke_width as f32,
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
            add_path(path, &mut vertices, &mut attribs);
            for path in iter {
                attribs.push(Attribute::empty());
                attribs.push(Attribute::empty());
                attribs.push(Attribute::empty());
                add_path(path, &mut vertices, &mut attribs);
            }
        }

        (vertices, attribs)
    }
}

/// Renders paths as scale-aware lines with variable width and color.
///
/// TODO: explain how this works
pub(crate) struct LinePainter {
    render_pipeline: RenderPipeline,
}

impl LinePainter {
    pub(crate) fn new(render_objects: &EngineRenderObjects) -> Self {
        // key insight: the stride is one point, but we expose 4 points at once!
        let points_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Float32x2,
                2 => Float32x2,
                3 => Float32x2,
            ],
        };

        let attributes_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Attribute>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                4 => Uint32,
                5 => Float32,
            ],
        };

        let shader = render_objects
            .device
            .create_shader_module(include_wgsl!("../shaders/line.wgsl"));

        let pipeline_layout =
            render_objects
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&render_objects.camera_bind_group_layout],
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
                        entry_point: "vs_main",
                        buffers: &[points_buffer_layout, attributes_buffer_layout],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(target)],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: PrimitiveTopology::TriangleStrip,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Self { render_pipeline }
    }
}

impl Painter for LinePainter {
    type Data = LinePainterData;

    fn draw<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        data: &'a LinePainterData,
    ) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, data.points_buffer.slice(..));
        rpass.set_vertex_buffer(1, data.attributes_buffer.slice(..));
        rpass.draw(0..4, 0..data.instance_count);
    }
}
