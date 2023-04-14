use crate::engine::Engine;
use std::mem;
use vsvg_core::point::Point;
use vsvg_core::FlattenedPath;
use wgpu::util::DeviceExt;
use wgpu::{
    include_wgsl, vertex_attr_array, Buffer, ColorTargetState, Device, PrimitiveTopology,
    RenderPass, RenderPipeline,
};

pub(crate) trait Painter: Send + Sync {
    fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup);
}

// ======================================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl From<&Point> for Vertex {
    fn from(point: &Point) -> Self {
        Self {
            #[allow(clippy::cast_possible_truncation)]
            position: [point.x() as f32, point.y() as f32],
        }
    }
}

impl From<Point> for Vertex {
    fn from(point: Point) -> Self {
        Self {
            #[allow(clippy::cast_possible_truncation)]
            position: [point.x() as f32, point.y() as f32],
        }
    }
}

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

/// Renders paths as scale-aware lines with variable width and color.
///
/// TODO: explain how this works
pub(super) struct LinePainter {
    render_pipeline: RenderPipeline,
    points_buffer: Buffer,
    attributes_buffer: Buffer,
    instance_count: u32,
}

impl LinePainter {
    pub(crate) fn new<'b, I>(engine: &Engine, device: &Device, paths: I) -> Self
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        let (vertices, attribs) = Self::build_buffers(paths);

        // prepare point buffer
        let points_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point instance buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

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

        // prepare color buffer
        let attributes_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Attributes instance Buffer"),
            contents: bytemuck::cast_slice(attribs.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let attributes_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Attribute>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                4 => Uint32,
                5 => Float32,
            ],
        };

        let shader = device.create_shader_module(include_wgsl!("shaders/line.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&engine.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // enable alpha blending
        let target = ColorTargetState {
            format: engine.target_format,
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

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        Self {
            render_pipeline,
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
            if path.data.len() > 1 {
                if path.data.len() > 2 && path.data.first() == path.data.last() {
                    vertices.push(path.data[path.data.len() - 2].into());
                    vertices.extend(path.data.iter().map(Vertex::from));
                    vertices.push(path.data[1].into());
                } else {
                    vertices.push(path.data.first().expect("length checked").into());
                    vertices.extend(path.data.iter().map(Vertex::from));
                    vertices.push(path.data.last().expect("length checked").into());
                }

                let attr = Attribute {
                    color: path.color.to_rgba(),
                    #[allow(clippy::cast_possible_truncation)]
                    width: path.stroke_width as f32,
                };

                for _ in 0..path.data.len() - 1 {
                    attribs.push(attr);
                }
            }
        }

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

impl Painter for LinePainter {
    fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.points_buffer.slice(..));
        rpass.set_vertex_buffer(1, self.attributes_buffer.slice(..));
        rpass.draw(0..4, 0..self.instance_count);
    }
}

// ======================================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ColorVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: u32,
}

/// Basic painter for drawing filled triangle strips
///
/// TODO: should support multiple distinct strips using indexed draw and primitive restart
pub(super) struct BasicPainter {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    vertex_count: u32,
}

impl BasicPainter {
    pub(crate) fn from_vertices_solid(
        engine: &Engine,
        device: &Device,
        vertices: impl IntoIterator<Item = [f32; 2]>,
        color: u32,
        primitive_type: PrimitiveTopology,
    ) -> Self {
        Self::new(
            engine,
            device,
            vertices
                .into_iter()
                .map(|v| ColorVertex { position: v, color }),
            primitive_type,
        )
    }

    pub(crate) fn new<I>(
        engine: &Engine,
        device: &Device,
        vertices_iterator: I,
        primitive_type: PrimitiveTopology,
    ) -> Self
    where
        I: IntoIterator<Item = ColorVertex>,
    {
        let vertices = vertices_iterator.into_iter().collect::<Vec<_>>();

        // prepare point buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Uint32,
            ],
        };

        let shader = device.create_shader_module(include_wgsl!("shaders/basic.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&engine.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // enable alpha blending
        let target = ColorTargetState {
            format: engine.target_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                    dst_factor: wgpu::BlendFactor::DstAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(target)],
            }),
            primitive: wgpu::PrimitiveState {
                topology: primitive_type,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            render_pipeline,
            vertex_buffer,
            #[allow(clippy::cast_possible_truncation)]
            vertex_count: vertices.len() as u32,
        }
    }
}

impl Painter for BasicPainter {
    fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw(0..self.vertex_count, 0..1);
    }
}

// ======================================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct PointVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: u32,
    pub(crate) size: f32,
}

pub(super) struct PointPainter {
    render_pipeline: RenderPipeline,
    instance_buffer: Buffer,
    instance_count: u32,
}

impl PointPainter {
    pub(crate) fn from_vertices_solid(
        engine: &Engine,
        device: &Device,
        vertices: impl IntoIterator<Item = [f32; 2]>,
        color: u32,
        size: f32,
    ) -> Self {
        Self::new(
            engine,
            device,
            vertices.into_iter().map(|v| PointVertex {
                position: v,
                color,
                size,
            }),
        )
    }

    pub(crate) fn new<I>(engine: &Engine, device: &Device, point_iterator: I) -> Self
    where
        I: IntoIterator<Item = PointVertex>,
    {
        let instances = point_iterator.into_iter().collect::<Vec<_>>();

        // prepare point buffer
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PointPainter instance buffer"),
            contents: bytemuck::cast_slice(instances.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PointVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Uint32,
                2 => Float32,
            ],
        };

        let shader = device.create_shader_module(include_wgsl!("shaders/point.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&engine.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // enable alpha blending
        let target = ColorTargetState {
            format: engine.target_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                    dst_factor: wgpu::BlendFactor::DstAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PointPainter pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
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

        Self {
            render_pipeline,
            instance_buffer,
            #[allow(clippy::cast_possible_truncation)]
            instance_count: instances.len() as u32,
        }
    }
}

impl Painter for PointPainter {
    fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        rpass.draw(0..4, 0..self.instance_count);
    }
}
