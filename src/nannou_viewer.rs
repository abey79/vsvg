use crate::types::{Color, Document, PageSize, Polylines};

use nannou::color::IntoLinSrgba;
use nannou::draw::properties::ColorScalar;
use nannou::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::ops::{Add, AddAssign};
use std::sync::Mutex;

struct Model {
    _window: window::Id,
    data: Data,
    offset: Vec2,
    drag_offset: Vec2,
}

impl IntoLinSrgba<ColorScalar> for Color {
    fn into_lin_srgba(self) -> LinSrgba<ColorScalar> {
        LinSrgba::new(
            self.r as f32 / 255.,
            self.g as f32 / 255.,
            self.b as f32 / 255.,
            self.a as f32 / 255.,
        )
    }
}

#[derive(Default)]
struct Data {
    polylines: Polylines,
    page_size: Option<PageSize>,
}

// this really silly way of passing state is due to the fact that `nannou::app()` doesn't accept
// a closure: https://github.com/nannou-org/nannou/issues/793
lazy_static! {
    static ref DATA: Mutex<RefCell<Data>> = Mutex::new(RefCell::new(Data::default()));
}

fn update(app: &App, model: &mut Model, _update: Update) {}

fn event(app: &App, model: &mut Model, event: Event) {
    for (btn, pt) in app.mouse.buttons.pressed() {
        if btn == MouseButton::Left {
            model.drag_offset = Point2::new(app.mouse.x, app.mouse.y) - pt;
        }
    }

    if app.mouse.buttons.left().is_up() {
        model.offset += model.drag_offset;
        model.drag_offset = vec2(0., 0.);
    }

    match event {
        Event::WindowEvent {
            id: _id,
            simple: Some(win_evt),
        } => println!("Window event: {:?}", win_evt),
        Event::DeviceEvent(id, evt) => println!("Device event: {:?} {:?}", id, evt),
        _ => (),
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let win = app.window_rect().pad(25.);
    let mut draw = app.draw();
    draw.background().color(LIGHTGRAY);

    draw = draw
        .x_y(-win.w() / 2., -win.h() / 2.)
        .translate(model.offset.extend(0.))
        .translate(model.drag_offset.extend(0.));

    if let Some(page_size) = model.data.page_size {
        draw.rect()
            .x_y(page_size.w as f32 / 2. + 10., page_size.h as f32 / 2. - 10.)
            .w_h(page_size.w as f32, page_size.h as f32)
            .color(GREY);
        draw.rect()
            .x_y(page_size.w as f32 / 2., page_size.h as f32 / 2.)
            .w_h(page_size.w as f32, page_size.h as f32)
            .color(WHITE)
            .stroke_weight(1.)
            .stroke(DARKGREY);
    }

    let mut counts: HashMap<_, usize> = HashMap::new();
    for line in model.data.polylines.iter() {
        (*counts.entry(line.points.len()).or_default()).add_assign(1);
        draw.polyline()
            .weight(line.stroke_width as f32)
            .color(line.color)
            .points(
                line.points
                    .iter()
                    .map(|p| Point2::new(p[0] as f32, p[1] as f32)),
            );
    }

    println!(
        "drawing {} polylines ({:?})",
        model.data.polylines.lines.len(),
        counts
    );

    draw.to_frame(app, &frame).unwrap();
}

impl Document {
    pub fn show(&self, tolerance: f64) -> Result<(), Box<dyn Error>> {
        DATA.lock().unwrap().replace(Data {
            polylines: self.flatten(tolerance),
            page_size: self.page_size,
        });

        nannou::app(|app| {
            let _window = app.new_window().view(view).build().unwrap();
            let data = (*DATA.lock().unwrap()).replace(Data::default());

            app.set_loop_mode(LoopMode::wait());

            Model {
                _window,
                data,
                offset: vec2(0., 0.),
                drag_offset: vec2(0., 0.),
            }
        })
        .update(update)
        .event(event)
        .run();
        Ok(())
    }
}
