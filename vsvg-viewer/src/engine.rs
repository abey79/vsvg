use crate::painters::{BasicPainter, LinePainter, Painter, PointPainter};
use eframe::egui_wgpu::RenderState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use vsvg::{
    Color, Document, DocumentTrait, FlattenedDocument, LayerID, LayerTrait, PathTrait, Point,
};
use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, PrimitiveTopology, TextureFormat};

const PEN_UP_TRAJECTORY_COLOR: u32 = Color::gray(168).to_rgba();
const PAGE_SHADOW_COLOR: u32 = Color::gray(180).to_rgba();
const PAGE_BACKGROUND_COLOR: u32 = Color::WHITE.to_rgba();
const PAGE_BORDER_COLOR: u32 = Color::gray(168).to_rgba();
const PAGE_SHADOW_SIZE: f32 = 7.;
const POINTS_COLOR: u32 = Color::BLACK.to_rgba();
const POINTS_SIZE: f32 = 2.0;
const CONTROL_POINTS_COLOR: u32 = Color::gray(128).to_rgba();
const CONTROL_POINTS_SIZE: f32 = 2.0;

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

#[derive(Debug, Default)]
pub struct DocumentData {
    pub(crate) flattened_document: FlattenedDocument,
    pub(crate) control_points: FlattenedDocument,
    pub(crate) display_vertices: HashMap<LayerID, Vec<Point>>,
}

impl DocumentData {
    pub(crate) fn new(document: &Document) -> Self {
        Self {
            flattened_document: document.flatten(0.01), //TODO: magic number
            control_points: document.control_points(),
            display_vertices: document
                .layers
                .iter()
                .map(|(&lid, layer)| (lid, layer.display_vertices()))
                .collect(),
        }
    }
}

struct LayerData {
    /// lines are always displayed
    line_painter: LinePainter,

    /// painter for points
    point_painter: PointPainter,

    /// painter for pen-up trajectories
    pen_up_painter: BasicPainter,

    /// painter for control points
    control_points_painter: PointPainter,

    /// painters for control lines
    control_lines_painter: BasicPainter,
}

pub(super) struct Engine {
    document_data: Arc<DocumentData>,

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
    layer_data: HashMap<LayerID, LayerData>,
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
        document_data: Arc<DocumentData>,
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
            document_data,
            viewer_options,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            target_format: wgpu_render_state.target_format,
            page_size_painters: vec![],
            layer_data: HashMap::new(),
        };

        // Note: this stuff must be done each time the document change, if we want to support
        // changing the document.
        engine.page_size_painters = engine.build_page_size_painters(device);
        engine.layer_data = engine.build_layer_painters(device);

        engine
    }

    fn build_page_size_painters(&self, device: &Device) -> Vec<BasicPainter> {
        let mut painters = vec![];

        // draw the page
        if let Some(page_size) = self.document_data.flattened_document.metadata().page_size {
            #[allow(clippy::cast_possible_truncation)]
            let (w, h) = (page_size.w as f32, page_size.h as f32);

            // shadow
            let shadow_vertices = [
                [PAGE_SHADOW_SIZE, h + PAGE_SHADOW_SIZE],
                [PAGE_SHADOW_SIZE, h],
                [w + PAGE_SHADOW_SIZE, h + PAGE_SHADOW_SIZE],
                [w, h],
                [w + PAGE_SHADOW_SIZE, PAGE_SHADOW_SIZE],
                [w, PAGE_SHADOW_SIZE],
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

    fn build_layer_painters(&self, device: &Device) -> HashMap<LayerID, LayerData> {
        let mut layers = HashMap::new();

        for (lid, flattened_layer) in &self.document_data.flattened_document.layers {
            let points = self
                .document_data
                .display_vertices
                .get(lid)
                .unwrap()
                .iter()
                .map(Into::into);

            let pen_up_trajectories = flattened_layer
                .pen_up_trajectories()
                .iter()
                .flat_map(|(start, end)| [start.into(), end.into()])
                .collect::<Vec<[f32; 2]>>();

            let control_points: Vec<_> = self.document_data.control_points.layers[lid]
                .paths
                .iter()
                .flat_map(|p| p.data().points())
                .map(Into::into)
                .collect();

            let layer_data = LayerData {
                line_painter: LinePainter::new(self, device, &flattened_layer.paths),
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
                control_points_painter: PointPainter::from_vertices_solid(
                    self,
                    device,
                    control_points.clone(),
                    CONTROL_POINTS_COLOR,
                    CONTROL_POINTS_SIZE,
                ),
                control_lines_painter: BasicPainter::from_vertices_solid(
                    self,
                    device,
                    control_points,
                    CONTROL_POINTS_COLOR,
                    PrimitiveTopology::LineList,
                ),
            };

            layers.insert(*lid, layer_data);
        }

        layers
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
        for (lid, layer_painters) in &self.layer_data {
            if *viewer_options.layer_visibility.get(lid).unwrap_or(&true) {
                layer_painters
                    .line_painter
                    .draw(render_pass, &self.camera_bind_group);

                if viewer_options.show_control_points {
                    layer_painters
                        .control_points_painter
                        .draw(render_pass, &self.camera_bind_group);

                    layer_painters
                        .control_lines_painter
                        .draw(render_pass, &self.camera_bind_group);
                }

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
