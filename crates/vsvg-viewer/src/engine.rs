use crate::painters::{
    BasicPainter, LineDisplayOptions, LinePainter, PageSizePainter, PageSizePainterData, Painter,
    PointPainter,
};
use eframe::egui_wgpu::RenderState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::render_data::{LayerRenderData, RenderData};
use vsvg::{Color, Document, DocumentTrait, LayerID, PageSize};
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

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct DisplayOptions {
    /// show points
    pub show_display_vertices: bool,

    /// show pen-up trajectories
    pub show_pen_up: bool,

    /// show control points
    pub show_bezier_handles: bool,

    /// tolerance parameter used for flattening curves by the renderer
    pub tolerance: f64,

    /// anti alias parameter
    pub anti_alias: f32,

    /// display options specific to the line painter
    pub line_display_options: LineDisplayOptions,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            show_display_vertices: false,
            show_pen_up: false,
            show_bezier_handles: false,
            tolerance: crate::DEFAULT_RENDERER_TOLERANCE,
            anti_alias: 0.5,
            line_display_options: LineDisplayOptions::default(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ViewerOptions {
    //TODO: implement that
    /// display mode
    pub display_mode: DisplayMode,

    /// display options
    pub display_options: DisplayOptions,

    /// layer visibility
    #[serde(skip)]
    pub layer_visibility: HashMap<LayerID, bool>,

    /// vertex count (updated by [`Engine::paint`] for display purposes)
    pub vertex_count: u64,
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

/// Painters contains shaders and render pipelines needed for drawing, but not any actually
/// vertex data.
#[allow(clippy::struct_field_names)]
struct LayerPainters {
    /// lines are always displayed
    line_painter: LinePainter,

    /// painter for display vertices
    display_vertices_painter: PointPainter,

    /// painter for pen-up trajectories
    pen_up_painter: BasicPainter,

    /// painter for the bezier handles' points
    bezier_handles_point_painter: PointPainter,

    /// painter for the bezier handles' lines
    bezier_handles_line_painter: BasicPainter,
}

impl LayerPainters {
    fn new(render_objects: &EngineRenderObjects) -> Self {
        Self {
            line_painter: LinePainter::new(render_objects),
            display_vertices_painter: PointPainter::new(render_objects),
            pen_up_painter: BasicPainter::new(render_objects, PrimitiveTopology::LineList),
            bezier_handles_point_painter: PointPainter::new(render_objects),
            bezier_handles_line_painter: BasicPainter::new(
                render_objects,
                PrimitiveTopology::LineList,
            ),
        }
    }

    fn paint(
        &self,
        layer_data: &LayerRenderData,
        display_options: DisplayOptions,
        render_objects: &EngineRenderObjects,
        render_pass: &mut wgpu::RenderPass<'static>,
    ) {
        vsvg::trace_function!();

        if let Some(line_painter_data) = layer_data.line_painter_data() {
            self.line_painter.draw(
                render_pass,
                render_objects,
                //&render_objects.camera_bind_group,
                line_painter_data,
            );
        }

        if display_options.show_bezier_handles {
            if let Some(bezier_handles_painter_data) = layer_data.bezier_handles_painter_data() {
                self.bezier_handles_point_painter.draw(
                    render_pass,
                    render_objects,
                    &bezier_handles_painter_data.point_painter_data,
                );

                self.bezier_handles_line_painter.draw(
                    render_pass,
                    render_objects,
                    &bezier_handles_painter_data.line_painter_data,
                );
            }
        }

        if display_options.show_display_vertices {
            if let Some(display_vertices_painter_data) = layer_data.display_vertices_painter_data()
            {
                self.display_vertices_painter.draw(
                    render_pass,
                    render_objects,
                    display_vertices_painter_data,
                );
            }
        }

        if display_options.show_pen_up {
            if let Some(pen_up_painter_data) = layer_data.pen_up_painter_data() {
                self.pen_up_painter
                    .draw(render_pass, render_objects, pen_up_painter_data);
            }
        }
    }
}

/// wgpu-related objects used by the engine.
///
/// They are grouped in a separate structure, so they can be provided to painters during engine
/// initialization.
pub(crate) struct EngineRenderObjects {
    pub(crate) device: Arc<Device>,
    pub(crate) camera_buffer: Buffer,
    pub(crate) camera_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) camera_bind_group: wgpu::BindGroup,
    pub(crate) target_format: TextureFormat,
}

pub(super) struct Engine {
    pub(super) render_objects: EngineRenderObjects,

    viewer_options: Arc<Mutex<ViewerOptions>>,

    render_data: Option<RenderData>,

    last_page_size: Option<PageSize>,

    // painters
    layer_painters: LayerPainters,

    /// per-layer painter data
    //layer_painter_data: HashMap<LayerID, LayerPainterData>,
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
            render_data: None,
            last_page_size: None,

            layer_painters,
            page_size_painter,
            page_size_painter_data: None,
        }
    }

    pub fn set_document(&mut self, document: Arc<Document>) {
        vsvg::trace_function!();

        // in most cases the page size won't change from a frame to the next, so we only rebuild
        // if needed
        let new_page_size = document.metadata().page_size;
        if self.last_page_size != new_page_size {
            self.page_size_painter_data = new_page_size
                .map(|page_size| PageSizePainterData::new(&self.render_objects, page_size));
            self.last_page_size = new_page_size;
        }

        // rebuild the document data only if the new document is different from the previous one
        if let Some(render_data) = self.render_data.as_ref() {
            if Arc::ptr_eq(&document, render_data.document()) {
                return;
            }
        }
        self.render_data = Some(RenderData::new(document));
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
            self.viewer_options
                .lock()
                .unwrap()
                .display_options
                .anti_alias,
        );

        queue.write_buffer(
            &self.render_objects.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        if let Some(render_data) = &mut self.render_data {
            render_data.prepare(
                &self.render_objects,
                self.viewer_options.lock().unwrap().display_options,
            );
        }
    }

    pub(super) fn paint(&self, render_pass: &mut wgpu::RenderPass<'static>) {
        vsvg::trace_function!();

        if let Some(page_size_painter_data) = &self.page_size_painter_data {
            self.page_size_painter
                .draw(render_pass, &self.render_objects, page_size_painter_data);
        }

        let mut viewer_options = self.viewer_options.lock().unwrap();

        let mut vertex_count = 0;
        if let Some(render_data) = &self.render_data {
            for (lid, layer_data) in render_data.layers() {
                if *viewer_options.layer_visibility.get(lid).unwrap_or(&true) {
                    self.layer_painters.paint(
                        layer_data,
                        viewer_options.display_options,
                        &self.render_objects,
                        render_pass,
                    );

                    vertex_count += layer_data.vertex_count();
                }
            }
        }
        viewer_options.vertex_count = vertex_count;
    }
}
