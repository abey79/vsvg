use bpaf::{construct, positional, Bpaf, Parser};

use vsvg::{Angle, Draw, Length, Transforms};

use crate::commands::{
    make_command_parser,
    utils::{pivot, Pivot},
    Command, DynCommand, State,
};

pub(crate) fn parser() -> impl Parser<DynCommand> {
    let d_translate = make_command_parser(d_translate());
    let d_rotate = make_command_parser(d_rotate());
    let d_scale = make_command_parser(d_scale());
    let line = make_command_parser(line());
    let circle = make_command_parser(circle());
    let ellipse = make_command_parser(ellipse());
    construct!([d_translate, d_rotate, d_scale, line, circle, ellipse]).group_help("Draw commands:")
}

// =================================================================================================
// Drawing state

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command("dtranslate"), adjacent)]
struct DTranslate {
    #[bpaf(positional("TX"))]
    tx: Length,
    #[bpaf(positional("TY"))]
    ty: Length,
}

impl Command for DTranslate {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.draw_state.translate(self.tx, self.ty);
        Ok(())
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command("drotate"), adjacent)]
struct DRotate {
    #[bpaf(external, optional)]
    pivot: Option<Pivot>,

    #[bpaf(positional("ANGLE"))]
    angle: Angle,
}

impl Command for DRotate {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        if let Some(pivot) = &self.pivot {
            state.draw_state.rotate_around(self.angle, pivot.x, pivot.y);
        } else {
            state.draw_state.rotate(self.angle);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command("dscale"), adjacent)]
struct DScale {
    #[bpaf(external, optional)]
    pivot: Option<Pivot>,

    #[bpaf(positional("SX"))]
    sx: Length,

    #[bpaf(positional("SY"), optional, catch)]
    sy: Option<Length>,
}

impl Command for DScale {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        if let Some(pivot) = &self.pivot {
            state
                .draw_state
                .scale_around(self.sx, self.sy.unwrap_or(self.sx), pivot.x, pivot.y);
        } else if let Some(sy) = self.sy {
            state.draw_state.scale_non_uniform(self.sx, sy);
        } else {
            state.draw_state.scale(self.sx);
        }

        Ok(())
    }
}

// =================================================================================================
// Drawing primitives

fn extra_coords() -> impl Parser<Vec<(Length, Length)>> {
    let x = positional::<Length>("X3");
    let y = positional::<Length>("Y3");
    construct!(x, y).many().catch()
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Line {
    #[bpaf(short, long)]
    close: bool,

    #[bpaf(positional("X1"))]
    x1: Length,
    #[bpaf(positional("Y1"))]
    y1: Length,
    #[bpaf(positional("X2"))]
    x2: Length,
    #[bpaf(positional("Y2"))]
    y2: Length,
    #[bpaf(external)]
    extra_coords: Vec<(Length, Length)>,
}

impl Command for Line {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.draw(|draw| {
            let first_points = [(self.x1, self.y1), (self.x2, self.y2)];
            let points = first_points
                .iter()
                .chain(&self.extra_coords)
                .map(|(x, y)| vsvg::Point::new(x, y));

            draw.polyline(points, self.close);
            Ok(())
        })
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Circle {
    #[bpaf(positional("CX"))]
    cx: Length,
    #[bpaf(positional("CY"))]
    cy: Length,
    #[bpaf(positional("R"))]
    r: Length,
}

impl Command for Circle {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.draw(|draw| {
            draw.circle(self.cx, self.cy, self.r);

            Ok(())
        })
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Ellipse {
    #[bpaf(short, long)]
    rotation: Option<Angle>,

    #[bpaf(positional("CX"))]
    cx: Length,
    #[bpaf(positional("CY"))]
    cy: Length,
    #[bpaf(positional("RX"))]
    rx: Length,
    #[bpaf(positional("RY"))]
    ry: Length,
}

impl Command for Ellipse {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.draw(|draw| {
            draw.ellipse(
                self.cx,
                self.cy,
                self.rx,
                self.ry,
                self.rotation.unwrap_or_default(),
            );

            Ok(())
        })
    }
}
