use crate::painters::{
    BasicPainter, BasicPainterData, LinePainter, LinePainterData, PageSizePainter,
    PageSizePainterData, Painter, PointPainter, PointPainterData,
};
use eframe::egui_wgpu::RenderState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use vsvg::{
    Color, Document, DocumentTrait, FlattenedDocument, LayerID, LayerTrait, PageSize, PathTrait,
    Point,
};
use wgpu::util::DeviceExt;
use wgpu::{Buffer, Device, PrimitiveTopology, TextureFormat};

pub(crate) const PEN_UP_TRAJECTORY_COLOR: u32 = Color::gray(168).to_rgba();
pub(crate) const PAGE_SHADOW_COLOR: u32 = Color::gray(180).to_rgba();
pub(crate) const PAGE_BACKGROUND_COLOR: u32 = Color::WHITE.to_rgba();
pub(crate) const PAGE_BORDER_COLOR: u32 = Color::gray(168).to_rgba();
pub(crate) const PAGE_SHADOW_SIZE: f32 = 7.;
pub(crate) const POINTS_COLOR: u32 = Color::BLACK.to_rgba();
pub(crate) const POINTS_SIZE: f32 = 2.0;
pub(crate) const CONTROL_POINTS_COLOR: u32 = Color::gray(128).to_rgba();
pub(crate) const CONTROL_POINTS_SIZE: f32 = 2.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum DisplayMode {
    #[default]
    Preview,
    Outline,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

    /// anti alias parameter
    #[serde(skip)]
    pub anti_alias: f32,
}

impl Default for ViewerOptions {
    fn default() -> Self {
        Self {
            display_mode: DisplayMode::default(),
            show_point: false,
            show_pen_up: false,
            show_control_points: false,
            override_width: None,
            override_opacity: None,
            layer_visibility: HashMap::default(),
            anti_alias: 0.5,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    projection: [[f32; 4]; 4],

    // these two must be grouped to avoid padding
    scale: f32,
    anti_alias: f32,

    screen_size: [f32; 2],
    // no padding: this is already 16-byte aligned!
}

impl CameraUniform {
    fn update(
        &mut self,
        m: cgmath::Matrix4<f32>,
        scale: f32,
        screen_size: (f32, f32),
        anti_alias: f32,
    ) {
        self.projection = m.into();
        self.scale = scale;
        self.screen_size = [screen_size.0, screen_size.1];
        self.anti_alias = anti_alias;
    }
}

#[derive(Debug, Default)]
pub struct DocumentData {
    pub(crate) flattened_document: FlattenedDocument,
    pub(crate) control_points: FlattenedDocument,
    pub(crate) display_vertices: HashMap<LayerID, Vec<Point>>,
}

impl DocumentData {
    #[must_use]
    pub fn new(document: &Document) -> Self {
        vsvg::trace_function!();

        Self {
            flattened_document: document.flatten(0.1), //TODO: magic number
            control_points: document.control_points(),
            display_vertices: document
                .layers
                .iter()
                .map(|(&lid, layer)| (lid, layer.display_vertices()))
                .collect(),
        }
    }
}

/// Painters contains shaders and render pipelines needed for drawing, but not any actually
/// vertex data.
struct LayerPainters {
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

impl LayerPainters {
    fn new(render_objects: &EngineRenderObjects) -> Self {
        Self {
            line_painter: LinePainter::new(render_objects),
            point_painter: PointPainter::new(render_objects),
            pen_up_painter: BasicPainter::new(render_objects, PrimitiveTopology::LineList),
            control_points_painter: PointPainter::new(render_objects),
            control_lines_painter: BasicPainter::new(render_objects, PrimitiveTopology::LineList),
        }
    }
}

/// Data needed for drawing a single layer, to be provided to the layer painters.
struct LayerPainterData {
    line_painter_data: LinePainterData,
    point_painter_data: PointPainterData,
    pen_up_painter_data: BasicPainterData,
    control_points_painter_data: PointPainterData,
    control_lines_painter_data: BasicPainterData,
}

impl LayerPainterData {
    fn build(
        render_objects: &EngineRenderObjects,
        document_data: &DocumentData,
    ) -> HashMap<LayerID, LayerPainterData> {
        let mut layers = HashMap::new();

        for (lid, flattened_layer) in &document_data.flattened_document.layers {
            let points = document_data
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

            let control_points: Vec<_> = document_data.control_points.layers[lid]
                .paths
                .iter()
                .flat_map(|p| p.data().points())
                .map(Into::into)
                .collect();

            let layer_data = LayerPainterData {
                line_painter_data: LinePainterData::new(render_objects, &flattened_layer.paths),
                point_painter_data: PointPainterData::new(
                    render_objects,
                    points,
                    POINTS_COLOR,
                    POINTS_SIZE,
                ),
                pen_up_painter_data: BasicPainterData::new(
                    render_objects,
                    pen_up_trajectories,
                    PEN_UP_TRAJECTORY_COLOR,
                ),
                control_points_painter_data: PointPainterData::new(
                    render_objects,
                    control_points.clone(),
                    CONTROL_POINTS_COLOR,
                    CONTROL_POINTS_SIZE,
                ),
                control_lines_painter_data: BasicPainterData::new(
                    render_objects,
                    control_points,
                    CONTROL_POINTS_COLOR,
                ),
            };

            layers.insert(*lid, layer_data);
        }

        layers
    }
}

/// wgpu-related objects used by the engine.
///
/// They are grouped in a separate structure, so they can be provided to painters during engine
/// initialisation.
pub(crate) struct EngineRenderObjects {
    pub(crate) device: Arc<Device>,
    pub(crate) camera_buffer: Buffer,
    pub(crate) camera_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) target_format: TextureFormat,
}

pub(super) struct Engine {
    render_objects: EngineRenderObjects,

    viewer_options: Arc<Mutex<ViewerOptions>>,

    last_page_size: Option<PageSize>,

    // painters
    layer_painters: LayerPainters,

    /// per-layer painter data
    layer_painter_data: HashMap<LayerID, LayerPainterData>,

    page_size_painter: PageSizePainter,
    page_size_painter_data: Option<PageSizePainterData>,
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
        viewer_options: Arc<Mutex<ViewerOptions>>,
    ) -> Self {
        let device = wgpu_render_state.device.clone();

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

        let render_objects = EngineRenderObjects {
            device,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
            target_format: wgpu_render_state.target_format,
        };

        let layer_painters = LayerPainters::new(&render_objects);
        let page_size_painter = PageSizePainter::new(&render_objects);

        Self {
            render_objects,

            viewer_options,
            last_page_size: None,

            layer_painters,
            layer_painter_data: HashMap::default(),
            page_size_painter,
            page_size_painter_data: None,
        }
    }

    pub fn set_document_data(&mut self, document_data: &DocumentData) {
        // in most cases the page size won't change from a frame to the next, so we only rebuild
        // if needed
        let new_page_size = document_data.flattened_document.metadata().page_size;
        if self.last_page_size != new_page_size {
            self.page_size_painter_data = new_page_size
                .map(|page_size| PageSizePainterData::new(&self.render_objects, page_size));
            self.last_page_size = new_page_size;
        }

        self.layer_painter_data = LayerPainterData::build(&self.render_objects, document_data);
    }

    pub(super) fn prepare(
        &mut self,
        _device: &Device,
        queue: &wgpu::Queue,
        rect: egui::Rect,
        scale: f32,
        origin: cgmath::Point2<f32>,
    ) {
        vsvg::trace_function!();

        // Update our uniform buffer with the angle from the UI
        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update(
            projection(origin, scale, rect.width(), rect.height()),
            scale,
            (rect.width(), rect.height()),
            self.viewer_options.lock().unwrap().anti_alias,
        );

        queue.write_buffer(
            &self.render_objects.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    pub(super) fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        vsvg::trace_function!();

        if let Some(page_size_painter_data) = &self.page_size_painter_data {
            self.page_size_painter.draw(
                render_pass,
                &self.render_objects.camera_bind_group,
                page_size_painter_data,
            );
        }

        let viewer_options = self.viewer_options.lock().unwrap();
        for (lid, layer_painter_data) in &self.layer_painter_data {
            if *viewer_options.layer_visibility.get(lid).unwrap_or(&true) {
                //TODO: move that to LayerPainters
                self.layer_painters.line_painter.draw(
                    render_pass,
                    &self.render_objects.camera_bind_group,
                    &layer_painter_data.line_painter_data,
                );

                if viewer_options.show_control_points {
                    self.layer_painters.control_points_painter.draw(
                        render_pass,
                        &self.render_objects.camera_bind_group,
                        &layer_painter_data.control_points_painter_data,
                    );

                    self.layer_painters.control_lines_painter.draw(
                        render_pass,
                        &self.render_objects.camera_bind_group,
                        &layer_painter_data.control_lines_painter_data,
                    );
                }

                if viewer_options.show_point {
                    self.layer_painters.point_painter.draw(
                        render_pass,
                        &self.render_objects.camera_bind_group,
                        &layer_painter_data.point_painter_data,
                    );
                }

                if viewer_options.show_pen_up {
                    self.layer_painters.pen_up_painter.draw(
                        render_pass,
                        &self.render_objects.camera_bind_group,
                        &layer_painter_data.pen_up_painter_data,
                    );
                }
            }
        }
    }
}
