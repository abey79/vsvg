use crate::engine::EngineRenderObjects;
use crate::painters::Painter;
use std::mem;
use wgpu::util::DeviceExt;
use wgpu::{
    include_wgsl, vertex_attr_array, Buffer, ColorTargetState, PrimitiveTopology, RenderPass,
    RenderPipeline,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct PointVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: u32,
    pub(crate) size: f32,
}

pub(crate) struct PointPainterData {
    instance_buffer: Buffer,
    instance_count: u32,
}

impl PointPainterData {
    pub fn new(
        render_objects: &EngineRenderObjects,
        vertices: impl IntoIterator<Item = [f32; 2]>,
        color: u32,
        size: f32,
    ) -> Self {
        let instances = vertices
            .into_iter()
            .map(|v| PointVertex {
                position: v,
                color,
                size,
            })
            .collect::<Vec<_>>();

        let instance_buffer =
            render_objects
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("PointPainter instance buffer"),
                    contents: bytemuck::cast_slice(instances.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Self {
            instance_buffer,
            #[allow(clippy::cast_possible_truncation)]
            instance_count: instances.len() as u32,
        }
    }
}

pub(crate) struct PointPainter {
    render_pipeline: RenderPipeline,
}

impl PointPainter {
    pub(crate) fn new(render_objects: &EngineRenderObjects) -> Self {
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<PointVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Uint32,
                2 => Float32,
            ],
        };

        let shader = render_objects
            .device
            .create_shader_module(include_wgsl!("../shaders/point.wgsl"));

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
                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
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

        Self { render_pipeline }
    }
}

impl Painter for PointPainter {
    type Data = PointPainterData;

    fn draw<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        data: &'a Self::Data,
    ) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, data.instance_buffer.slice(..));
        rpass.draw(0..4, 0..data.instance_count);
    }
}
