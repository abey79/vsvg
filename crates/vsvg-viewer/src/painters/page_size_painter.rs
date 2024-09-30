use crate::engine::{
    EngineRenderObjects, PAGE_BACKGROUND_COLOR, PAGE_BORDER_COLOR, PAGE_SHADOW_COLOR,
    PAGE_SHADOW_SIZE,
};
use crate::painters::{BasicPainter, BasicPainterData, Painter};
use vsvg::PageSize;
use wgpu::{BindGroup, RenderPass};

pub(crate) struct PageSizePainterData {
    background: BasicPainterData,
    shadow: BasicPainterData,
    border: BasicPainterData,
}

impl PageSizePainterData {
    pub(crate) fn new(render_objects: &EngineRenderObjects, page_size: PageSize) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let (w, h) = (page_size.w() as f32, page_size.h() as f32);

        // shadow
        let shadow_vertices = [
            [PAGE_SHADOW_SIZE, h + PAGE_SHADOW_SIZE],
            [PAGE_SHADOW_SIZE, h],
            [w + PAGE_SHADOW_SIZE, h + PAGE_SHADOW_SIZE],
            [w, h],
            [w + PAGE_SHADOW_SIZE, PAGE_SHADOW_SIZE],
            [w, PAGE_SHADOW_SIZE],
        ];
        let background_vertices = [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]];
        let border_vertices = [[0., 0.], [w, 0.], [w, h], [0., h], [0., 0.]];

        Self {
            shadow: BasicPainterData::new(render_objects, shadow_vertices, PAGE_SHADOW_COLOR),
            background: BasicPainterData::new(
                render_objects,
                background_vertices,
                PAGE_BACKGROUND_COLOR,
            ),
            border: BasicPainterData::new(render_objects, border_vertices, PAGE_BORDER_COLOR),
        }
    }
}

pub(crate) struct PageSizePainter {
    background_and_shadow_painter: BasicPainter, // triangle_strip
    border_painter: BasicPainter,                // line_strip
}

impl PageSizePainter {
    pub(crate) fn new(render_objects: &EngineRenderObjects) -> Self {
        Self {
            background_and_shadow_painter: BasicPainter::new(
                render_objects,
                wgpu::PrimitiveTopology::TriangleStrip,
            ),
            border_painter: BasicPainter::new(render_objects, wgpu::PrimitiveTopology::LineStrip),
        }
    }
}

impl Painter for PageSizePainter {
    type Data = PageSizePainterData;

    fn draw(
        &self,
        rpass: &mut RenderPass<'static>,
        camera_bind_group: &BindGroup,
        data: &Self::Data,
    ) {
        self.background_and_shadow_painter
            .draw(rpass, camera_bind_group, &data.shadow);
        self.background_and_shadow_painter
            .draw(rpass, camera_bind_group, &data.background);
        self.border_painter
            .draw(rpass, camera_bind_group, &data.border);
    }
}
