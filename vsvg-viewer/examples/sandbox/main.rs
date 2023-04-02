use std::mem;
use wgpu::util::DeviceExt;
use wgpu::{
    include_wgsl, vertex_attr_array, Adapter, Buffer, Device, Instance, Queue, RenderPass,
    RenderPipeline, Surface,
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
}

impl CameraUniform {
    fn update(&mut self, m: cgmath::Matrix4<f32>) {
        self.projection = m.into();
    }
}

struct Engine {
    // wgpu stuff
    pub surface: Surface,
    pub device: Device,
    pub adapter: Adapter,
    pub queue: Queue,
    pub config: wgpu::SurfaceConfiguration,
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
        let width = 1;
        let height = 1;
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
        let swapchain_format = swapchain_capabilities.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
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
        camera_uniform.update(projection(origin, scale, width as f32, height as f32));

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
            adapter,
            queue,
            config,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            origin: cgmath::Point2::new(0.0, 0.0),
            scale: 1.0,
            painters: vec![],
        }
    }

    fn pan(&mut self, delta: cgmath::Vector2<f32>) {
        self.origin -= delta / self.scale;
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Reconfigure the surface with the new size
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn rebuild(&mut self) {
        self.painters.clear();
        self.painters
            .push(Box::new(InstancedTrianglePainter::new(self)));
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
        self.camera_uniform.update(projection(
            self.origin,
            self.scale,
            self.config.width as f32,
            self.config.height as f32,
        ));
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
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
struct Location {
    position: [f32; 3],
}

struct InstancedTrianglePainter {
    render_pipeline: RenderPipeline,
    instance_buffer: Buffer,
}

impl InstancedTrianglePainter {
    const LOCATIONS: [Location; 3] = [
        Location {
            position: [400.0, 300.0, 0.0],
        },
        Location {
            position: [600.0, 500.0, 0.0],
        },
        Location {
            position: [800.0, 200.0, 0.0],
        },
    ];

    fn new(engine: &Engine) -> Self {
        let instance_buffer = engine
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&Self::LOCATIONS),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let buffer_layout = wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Location>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![0 => Float32x3],
        };

        let swapchain_capabilities = engine.surface.get_capabilities(&engine.adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let shader = engine
            .device
            .create_shader_module(include_wgsl!("instanced_triangle.wgsl"));

        let pipeline_layout =
            engine
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&engine.camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            engine
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("instanced triangle"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[buffer_layout],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(swapchain_format.into())],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Self {
            render_pipeline,
            instance_buffer,
        }
    }
}

impl Painter for InstancedTrianglePainter {
    fn draw<'a>(&'a self, rpass: &mut RenderPass<'a>, camera_bind_group: &'a wgpu::BindGroup) {
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, camera_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        rpass.draw(0..3, 0..3);
    }
}

// ======================================================================================

struct PanHelper {
    panning: bool,
    last_cursor_position: Option<cgmath::Vector2<f32>>,
}

impl PanHelper {
    fn new() -> Self {
        Self {
            panning: false,
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

    fn update(&mut self, cursor_position: PhysicalPosition<f64>) -> Option<cgmath::Vector2<f32>> {
        if self.panning {
            let cursor_position = cgmath::vec2(cursor_position.x as f32, cursor_position.y as f32);
            if let Some(last_cursor_position) = self.last_cursor_position {
                let delta = cursor_position - last_cursor_position;
                self.last_cursor_position = Some(cursor_position);
                Some(delta)
            } else {
                self.last_cursor_position = Some(cursor_position);
                None
            }
        } else {
            None
        }
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut engine = Engine::new(&window).await;
    engine.resize(window.inner_size());
    engine.rebuild();

    let mut pan_helper = PanHelper::new();

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

// ======================================================================================

fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    env_logger::init();

    // Temporarily avoid srgb formats for the swapchain on the web
    pollster::block_on(run(event_loop, window));
}
