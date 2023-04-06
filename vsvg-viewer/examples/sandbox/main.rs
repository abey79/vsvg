use std::{env, mem, process};
use vsvg_core::point::Point;
use vsvg_core::{Document, FlattenedPath};
use wgpu::util::DeviceExt;
use wgpu::{
    include_wgsl, vertex_attr_array, Buffer, ColorTargetState, Device, Instance, PrimitiveTopology,
    Queue, RenderPass, RenderPipeline, Surface,
};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    projection: [[f32; 4]; 4],
    scale: f32,
    screen_size: [f32; 2],
    _padding: [u8; 4], // WGSL uniform buffer requires 16-byte alignment
}

impl CameraUniform {
    fn update(&mut self, m: cgmath::Matrix4<f32>, scale: f32, screen_size: (u32, u32)) {
        self.projection = m.into();
        self.scale = scale;
        self.screen_size = [screen_size.0 as f32, screen_size.1 as f32];
    }
}

struct Engine {
    // wgpu stuff
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub color_format: wgpu::TextureFormat,

    // camera stuff
    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,

    // viewport stuff
    pub origin: cgmath::Point2<f32>,
    pub scale: f32,

    // painters
    pub painters: Vec<Box<dyn Painter>>,
}

fn projection(
    origin: cgmath::Point2<f32>,
    scale: f32,
    width: f32,
    height: f32,
) -> cgmath::Matrix4<f32> {
    cgmath::ortho(
        origin.x,
        origin.x + width / scale,
        origin.y + height / scale,
        origin.y,
        -1.0,
        1.0,
    )
}

impl Engine {
    async fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
    ) -> Self {
        // TODO: cleaner way to deal with that?
        let width = 4;
        let height = 4;
        // Handle to some wgpu API, can specify which backend(s) to make available
        // metal only on Mac. vulkan or dx12 on windows. etc.
        let instance = Instance::default();

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // Both are owned by this function, so it's ok.
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        // Handle to some actual GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let color_format = Self::preferred_framebuffer_format(&swapchain_capabilities.formats);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: color_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        // handle camera uniform buffer
        let origin = cgmath::Point2::new(0.0, 0.0);
        let scale = 1.0;
        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update(
            projection(origin, scale, width as f32, height as f32),
            1.0,
            (width, height),
        );

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            surface,
            device,
            queue,
            config,
            color_format,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            origin: cgmath::Point2::new(0.0, 0.0),
            scale: 1.0,
            painters: vec![],
        }
    }

    /// Find a preferred framebuffer format from those available.
    ///
    /// Copied from egui_wgpu backend.
    fn preferred_framebuffer_format(formats: &[wgpu::TextureFormat]) -> wgpu::TextureFormat {
        for &format in formats {
            if matches!(
                format,
                wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
            ) {
                return format;
            }
        }
        formats[0]
    }

    fn pan(&mut self, delta: cgmath::Vector2<f32>) {
        self.origin -= delta / self.scale;
    }

    fn zoom(&mut self, zoom: f32, mouse: cgmath::Vector2<f32>) {
        let new_scale = self.scale * (1.0 + zoom);

        let dz = 1. / self.scale - 1. / new_scale;
        self.origin = (self.origin + mouse * dz).into();
        self.scale = new_scale;
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Reconfigure the surface with the new size
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn clear_painters(&mut self) {
        self.painters.clear();
    }

    fn add_painter(&mut self, painter: Box<dyn Painter>) {
        self.painters.push(painter);
    }

    fn render(&mut self) {
        // frame to render to
        let output = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        // "control how the render code interacts with the texture"
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // update camera uniform buffer
        self.camera_uniform.update(
            projection(
                self.origin,
                self.scale,
                self.config.width as f32,
                self.config.height as f32,
            ),
            self.scale,
            (self.config.width, self.config.height),
        );
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // "Most modern graphics frameworks expect commands to be stored in a command buffer
        // before being sent to the gpu. The encoder builds a command buffer that we can
        // then send to the gpu."
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // this mutably borrows encoder
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 242.0 / 255.0,
                            g: 242.0 / 255.0,
                            b: 242.0 / 255.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // paint stuff!
            self.painters
                .iter()
                .for_each(|painter| painter.draw(&mut rpass, &self.camera_bind_group));
        }
        // borrow ends here, allowing to `finish()`

        // execute drawing
        self.queue.submit(Some(encoder.finish()));
        output.present();
    }
}

// ======================================================================================

trait Painter {
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
            position: [point.x() as f32, point.y() as f32],
        }
    }
}

impl From<Point> for Vertex {
    fn from(point: Point) -> Self {
        Self {
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
struct LinePainter {
    render_pipeline: RenderPipeline,
    points_buffer: Buffer,
    attributes_buffer: Buffer,
    instance_count: u32,
}

impl LinePainter {
    fn new<'b, I>(engine: &Engine, paths: I) -> Self
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        let (vertices, attribs) = Self::build_buffers(paths);

        // prepare point buffer
        let points_buffer = engine
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
        let attributes_buffer =
            engine
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

        let shader = engine
            .device
            .create_shader_module(include_wgsl!("line.wgsl"));

        let pipeline_layout =
            engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&engine.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        // enable alpha blending
        let target = ColorTargetState {
            format: engine.color_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline =
            engine
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

        Self {
            render_pipeline,
            points_buffer,
            attributes_buffer,
            instance_count: attribs.len() as u32,
        }
    }

    fn build_buffers<'b, I>(paths: I) -> (Vec<Vertex>, Vec<Attribute>)
    where
        I: IntoIterator<Item = &'b FlattenedPath>,
    {
        let mut iter = paths.into_iter();
        let min_size = 1000.min(iter.size_hint().0 * 4);

        // build the data buffers
        let mut vertices: Vec<Vertex> = Vec::with_capacity(min_size);
        let mut attribs = Vec::with_capacity(min_size);

        fn add_path(
            path: &FlattenedPath,
            vertices: &mut Vec<Vertex>,
            attribs: &mut Vec<Attribute>,
        ) {
            if path.data.len() > 1 {
                if path.data.len() > 2 && path.data.first() == path.data.last() {
                    vertices.push(path.data[path.data.len() - 2].into());
                    vertices.extend(path.data.iter().map(|p| Vertex::from(p)));
                    vertices.push(path.data[1].into());
                } else {
                    vertices.push(path.data.first().expect("length checked").into());
                    vertices.extend(path.data.iter().map(|p| Vertex::from(p)));
                    vertices.push(path.data.last().expect("length checked").into());
                }

                let attr = Attribute {
                    color: path.color.to_rgba(),
                    width: path.stroke_width as f32,
                };

                for _ in 0..path.data.len() - 1 {
                    attribs.push(attr);
                }
            }
        }

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
struct ColorVertex {
    position: [f32; 2],
    color: u32,
}

/// Basic painter for drawing filled triangle strips
///
/// TODO: should support multiple distinct strips using indexed draw and primitive restart
struct BasicPainter {
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    vertex_count: u32,
}

impl BasicPainter {
    fn new<I>(engine: &Engine, vertices_iterator: I, primitive_type: PrimitiveTopology) -> Self
    where
        I: IntoIterator<Item = ColorVertex>,
    {
        let vertices = vertices_iterator.into_iter().collect::<Vec<_>>();

        // prepare point buffer
        let vertex_buffer = engine
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

        let shader = engine
            .device
            .create_shader_module(include_wgsl!("basic.wgsl"));

        let pipeline_layout =
            engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&engine.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        // enable alpha blending
        let target = ColorTargetState {
            format: engine.color_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline =
            engine
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

        Self {
            render_pipeline,
            vertex_buffer,
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
struct PointVertex {
    position: [f32; 2],
    color: u32,
    size: f32,
}

struct PointPainter {
    render_pipeline: RenderPipeline,
    instance_buffer: Buffer,
    instance_count: u32,
}

impl PointPainter {
    fn new<I>(engine: &Engine, point_iterator: I) -> Self
    where
        I: IntoIterator<Item = PointVertex>,
    {
        let instances = point_iterator.into_iter().collect::<Vec<_>>();

        // prepare point buffer
        let instance_buffer = engine
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

        let shader = engine
            .device
            .create_shader_module(include_wgsl!("point.wgsl"));

        let pipeline_layout =
            engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&engine.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        // enable alpha blending
        let target = ColorTargetState {
            format: engine.color_format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline =
            engine
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

        Self {
            render_pipeline,
            instance_buffer,
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

// ======================================================================================

struct ZoomPanHelper {
    panning: bool,
    cursor_position: cgmath::Vector2<f32>,
    last_cursor_position: Option<cgmath::Vector2<f32>>,
}

impl ZoomPanHelper {
    fn new() -> Self {
        Self {
            panning: false,
            cursor_position: cgmath::vec2(0.0, 0.0),
            last_cursor_position: None,
        }
    }

    fn start(&mut self) {
        self.panning = true;
        self.last_cursor_position = None;
    }

    fn stop(&mut self) {
        self.panning = false;
    }

    fn cursor(&self) -> cgmath::Vector2<f32> {
        self.cursor_position
    }

    fn update(&mut self, cursor_position: PhysicalPosition<f64>) -> Option<cgmath::Vector2<f32>> {
        self.cursor_position = cgmath::vec2(cursor_position.x as f32, cursor_position.y as f32);
        if self.panning {
            if let Some(last_cursor_position) = self.last_cursor_position {
                let delta = self.cursor_position - last_cursor_position;
                self.last_cursor_position = Some(self.cursor_position);
                Some(delta)
            } else {
                self.last_cursor_position = Some(self.cursor_position);
                None
            }
        } else {
            None
        }
    }
}

pub struct Viewer<'a> {
    doc: &'a Document,
}

impl<'a> Viewer<'a> {
    pub fn new(doc: &'a Document) -> Self {
        Self { doc }
    }
}

impl Viewer<'_> {
    async fn run_loop(&self, event_loop: EventLoop<()>, window: Window) {
        let mut engine = Engine::new(&window).await;
        engine.resize(window.inner_size());
        self.rebuild(&mut engine);

        let mut pan_helper = ZoomPanHelper::new();

        event_loop.run(move |event, _, control_flow| {
            // Have the closure take ownership of the resources.
            // `event_loop.run` never returns, therefore we must do this to ensure
            // the resources are properly cleaned up.

            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                }
                | Event::WindowEvent {
                    event:
                        WindowEvent::ScaleFactorChanged {
                            new_inner_size: &mut size,
                            ..
                        },
                    ..
                } => {
                    engine.resize(size);

                    // On macOS the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    if button == MouseButton::Left {
                        if state == ElementState::Pressed {
                            pan_helper.start();
                        } else {
                            pan_helper.stop();
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    if let Some(delta) = pan_helper.update(position) {
                        engine.pan(delta);
                        engine.render();
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::TouchpadMagnify { delta, .. },
                    ..
                } => {
                    //println!("{event:?}");
                    engine.zoom(delta as f32, pan_helper.cursor());
                    engine.render();
                }
                Event::RedrawRequested(_) => {
                    engine.render();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });
    }

    fn rebuild(&self, engine: &mut Engine) {
        engine.clear_painters();

        // draw the page
        if let Some(page_size) = self.doc.page_size {
            let (w, h) = (page_size.w as f32, page_size.h as f32);

            // shadow
            const OFFSET: f32 = 10.;
            let shadow_vertices = [
                [OFFSET, h + OFFSET],
                [OFFSET, h],
                [w + OFFSET, h + OFFSET],
                [w, h],
                [w + OFFSET, OFFSET],
                [w, OFFSET],
            ];

            engine.add_painter(Box::new(BasicPainter::new(
                engine,
                shadow_vertices.iter().map(|v| ColorVertex {
                    position: *v,
                    color: 0xFFB4B4B4,
                }),
                PrimitiveTopology::TriangleStrip,
            )));

            // white background
            let vertices = [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]];
            engine.add_painter(Box::new(BasicPainter::new(
                engine,
                vertices.iter().map(|v| ColorVertex {
                    position: *v,
                    color: 0xFFFFFFFF,
                }),
                PrimitiveTopology::TriangleStrip,
            )));

            // page border
            let vertices = [[0., 0.], [w, 0.], [w, h], [0., h], [0., 0.]];
            engine.add_painter(Box::new(BasicPainter::new(
                engine,
                vertices.iter().map(|v| ColorVertex {
                    position: *v,
                    color: 0xFFA8A8A8,
                }),
                PrimitiveTopology::LineStrip,
            )));
        }

        const TOLERANCE: f64 = 0.1;
        for (_, layer) in self.doc.layers.iter() {
            let flat_layer = layer.flatten(TOLERANCE);

            // draw the layer paths
            engine.add_painter(Box::new(LinePainter::new(engine, &flat_layer.paths)));

            // draw the pen-up trajectories
            let pen_up_trajectories = flat_layer
                .paths
                .windows(2)
                .filter_map(|p| {
                    if let (Some(from), Some(to)) = (p[0].data.last(), p[1].data.first()) {
                        Some([
                            ColorVertex {
                                position: from.into(),
                                color: 0xFFA8A8A8,
                            },
                            ColorVertex {
                                position: to.into(),
                                color: 0xFFA8A8A8,
                            },
                        ])
                    } else {
                        None
                    }
                })
                .flatten();

            engine.add_painter(Box::new(BasicPainter::new(
                engine,
                pen_up_trajectories,
                PrimitiveTopology::LineList,
            )));

            // draw the points
            let points = flat_layer
                .paths
                .iter()
                .flat_map(|p| p.data.iter())
                .map(|p| PointVertex {
                    position: p.into(),
                    color: 0xFF000000,
                    size: 10.0,
                });
            engine.add_painter(Box::new(PointPainter::new(engine, points)));
        }
    }

    pub fn show(&self) {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).unwrap();
        env_logger::init();

        pollster::block_on(self.run_loop(event_loop, window));
    }
}

// ======================================================================================

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: cargo run --release --example -- <svg-file>");
        process::exit(1);
    });

    let doc = Document::from_svg(path, false).unwrap();

    let viewer = Viewer::new(&doc);
    viewer.show();
}
