mod basic_painter;
mod line_painter;
mod page_size_painter;
mod point_painter;

use vsvg::Point;
use wgpu::RenderPass;

pub(crate) use basic_painter::{BasicPainter, BasicPainterData};
pub(crate) use line_painter::{LineDisplayOptions, LinePainter, LinePainterData};
pub(crate) use page_size_painter::{PageSizePainter, PageSizePainterData};
pub(crate) use point_painter::{PointPainter, PointPainterData};

pub(crate) trait Painter {
    type Data;

    fn draw<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        data: &'a Self::Data,
    );
}

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
