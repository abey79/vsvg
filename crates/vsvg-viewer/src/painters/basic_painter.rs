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
pub(crate) struct ColorVertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: u32,
}

pub(crate) struct BasicPainterData {
    vertex_buffer: Buffer,
    vertex_count: u32,
}

impl BasicPainterData {
    pub fn new(
        render_objects: &EngineRenderObjects,
        vertices: impl IntoIterator<Item = [f32; 2]>,
        color: u32,
    ) -> Self {
        vsvg::trace_function!();

        let vertices = vertices
            .into_iter()
            .map(|v| ColorVertex { position: v, color })
            .collect::<Vec<_>>();

        let vertex_buffer =
            render_objects
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex buffer"),
                    contents: bytemuck::cast_slice(vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Self {
            vertex_buffer,
            #[allow(clippy::cast_possible_truncation)]
            vertex_count: vertices.len() as u32,
        }
    }
}

/// Basic painter for drawing filled triangle strips
///
/// TODO: should support multiple distinct strips using indexed draw and primitive restart
pub(crate) struct BasicPainter {
    render_pipeline: RenderPipeline,
}

impl BasicPainter {
    pub(crate) fn new(
        render_objects: &EngineRenderObjects,
        primitive_type: PrimitiveTopology,
    ) -> Self {
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ColorVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2,
                1 => Uint32,
            ],
        };

        let shader = render_objects
            .device
            .create_shader_module(include_wgsl!("../shaders/basic.wgsl"));

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

        Self { render_pipeline }
    }
}

impl Painter for BasicPainter {
    type Data = BasicPainterData;
    fn draw<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        data: &'a Self::Data,
    ) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, data.vertex_buffer.slice(..));
        rpass.draw(0..data.vertex_count, 0..1);
    }
}
