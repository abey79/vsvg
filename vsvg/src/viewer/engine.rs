use crate::viewer::painters::{BasicPainter, LinePainter, Painter, PointPainter};
use eframe::egui_wgpu::RenderState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use vsvg_core::{FlattenedDocument, LayerID};
use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, PrimitiveTopology, TextureFormat};

const PEN_UP_TRAJECTORY_COLOR: u32 = 0xFFA8A8A8;
const PAGE_SHADOW_COLOR: u32 = 0xFFB4B4B4;
const PAGE_BACKGROUND_COLOR: u32 = 0xFFFFFFFF;
const PAGE_BORDER_COLOR: u32 = 0xFFA8A8A8;
const POINTS_COLOR: u32 = 0xFF000000;
const POINTS_SIZE: f32 = 2.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum DisplayMode {
    #[default]
    Preview,
    Outline,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ViewerOptions {
    /// display mode
    pub display_mode: DisplayMode,

    /// show points
    pub show_point: bool,

    /// show pen-up trajectories
    pub show_pen_up: bool,

    /// show control points
    pub show_control_points: bool,

    /// override width
    pub override_width: Option<f32>,

    /// override opacity
    pub override_opacity: Option<f32>,

    /// layer visibility
    #[serde(skip)]
    pub layer_visibility: HashMap<LayerID, bool>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    projection: [[f32; 4]; 4],
    scale: f32,
    screen_size: [f32; 2],
    _padding: [u8; 4], // WGSL uniform buffer requires 16-byte alignment
}

impl CameraUniform {
    fn update(&mut self, m: cgmath::Matrix4<f32>, scale: f32, screen_size: (f32, f32)) {
        self.projection = m.into();
        self.scale = scale;
        self.screen_size = [screen_size.0, screen_size.1];
    }
}

struct LayerPainters {
    /// lines are always displayed
    line_painter: LinePainter,

    /// painter for points
    point_painter: PointPainter,

    /// painter for pen-up trajectories
    pen_up_painter: BasicPainter,
}

pub(super) struct Engine {
    document: Arc<FlattenedDocument>,
    //TODO: implement that
    #[allow(dead_code)]
    control_points: Arc<Option<FlattenedDocument>>,

    viewer_options: Arc<Mutex<ViewerOptions>>,

    // camera stuff
    camera_buffer: Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    camera_bind_group: wgpu::BindGroup,

    pub target_format: TextureFormat,

    // painters
    /// base painters are always active
    page_size_painters: Vec<BasicPainter>,

    /// per-layer painters
    layer_painters: HashMap<LayerID, LayerPainters>,
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
    pub(crate) fn new(
        wgpu_render_state: &RenderState,
        document: Arc<FlattenedDocument>,
        viewer_options: Arc<Mutex<ViewerOptions>>,
    ) -> Self {
        let device = &wgpu_render_state.device;

        // prepare camera uniform buffer and bind group
        let camera_uniform = CameraUniform::default();
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

        let mut engine = Self {
            document,
            control_points: Arc::new(None),
            viewer_options,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            target_format: wgpu_render_state.target_format,
            page_size_painters: vec![],
            layer_painters: HashMap::new(),
        };

        // Note: this stuff must be done each time the document change, if we want to support
        // changing the document.
        engine.page_size_painters = engine.build_page_size_painters(device);
        engine.layer_painters = engine.build_layer_painters(device);

        engine
    }

    fn build_page_size_painters(&self, device: &Device) -> Vec<BasicPainter> {
        let mut painters = vec![];

        // draw the page
        if let Some(page_size) = self.document.page_size {
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

            painters.push(BasicPainter::from_vertices_solid(
                self,
                device,
                shadow_vertices,
                PAGE_SHADOW_COLOR,
                PrimitiveTopology::TriangleStrip,
            ));

            // white background
            let vertices = [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]];
            painters.push(BasicPainter::from_vertices_solid(
                self,
                device,
                vertices,
                PAGE_BACKGROUND_COLOR,
                PrimitiveTopology::TriangleStrip,
            ));

            // page border
            let vertices = [[0., 0.], [w, 0.], [w, h], [0., h], [0., 0.]];
            painters.push(BasicPainter::from_vertices_solid(
                self,
                device,
                vertices,
                PAGE_BORDER_COLOR,
                PrimitiveTopology::LineStrip,
            ));
        }

        painters
    }

    fn build_layer_painters(&self, device: &Device) -> HashMap<LayerID, LayerPainters> {
        self.document
            .layers
            .iter()
            .map(|(&lid, layer)| {
                let points = layer
                    .paths
                    .iter()
                    .flat_map(|p| p.data.iter())
                    .map(|p| p.into());

                let pen_up_trajectories = layer
                    .paths
                    .windows(2)
                    .filter_map(|p| {
                        if let (Some(from), Some(to)) = (p[0].data.last(), p[1].data.first()) {
                            Some([from.into(), to.into()])
                        } else {
                            None
                        }
                    })
                    .flatten();

                (
                    lid,
                    LayerPainters {
                        line_painter: LinePainter::new(self, device, &layer.paths),
                        point_painter: PointPainter::from_vertices_solid(
                            self,
                            device,
                            points,
                            POINTS_COLOR,
                            POINTS_SIZE,
                        ),
                        pen_up_painter: BasicPainter::from_vertices_solid(
                            self,
                            device,
                            pen_up_trajectories,
                            PEN_UP_TRAJECTORY_COLOR,
                            PrimitiveTopology::LineList,
                        ),
                    },
                )
            })
            .collect()
    }

    pub(super) fn prepare(
        &mut self,
        _device: &Device,
        queue: &wgpu::Queue,
        rect: egui::Rect,
        scale: f32,
        origin: cgmath::Point2<f32>,
    ) {
        // Update our uniform buffer with the angle from the UI
        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update(
            projection(origin, scale, rect.width(), rect.height()),
            scale,
            (rect.width(), rect.height()),
        );

        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    pub(super) fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        self.page_size_painters
            .iter()
            .for_each(|painter| painter.draw(render_pass, &self.camera_bind_group));

        let viewer_options = self.viewer_options.lock().unwrap();
        for (lid, layer_painters) in self.layer_painters.iter() {
            if *viewer_options.layer_visibility.get(lid).unwrap_or(&true) {
                layer_painters
                    .line_painter
                    .draw(render_pass, &self.camera_bind_group);

                if viewer_options.show_point {
                    layer_painters
                        .point_painter
                        .draw(render_pass, &self.camera_bind_group);
                }

                if viewer_options.show_pen_up {
                    layer_painters
                        .pen_up_painter
                        .draw(render_pass, &self.camera_bind_group);
                }
            }
        }
    }
}
