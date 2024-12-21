use bpaf::{construct, Bpaf, Parser};

use vsvg::{Angle, Length, Transforms};

use crate::commands::{
    make_command_parser,
    utils::{pivot, Pivot},
    Command, DynCommand, State,
};

/// Parser for this group of commands.
pub(crate) fn parser() -> impl Parser<DynCommand> {
    let translate = make_command_parser(translate());
    let rotate = make_command_parser(rotate());
    let scale = make_command_parser(scale());

    construct!([translate, rotate, scale]).group_help("Transform commands:")
}

/// Translate geometries
///
///
/// This command translates the geometries by TX horizontal (positive right) and TY vertical
/// (positive down). Both TX and TY may have units (default is pixels).
#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Translate {
    #[bpaf(positional("TX"))]
    tx: Length,
    #[bpaf(positional("TY"))]
    ty: Length,
}

impl Command for Translate {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.document.translate(self.tx, self.ty);
        Ok(())
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Rotate {
    #[bpaf(external, optional)]
    pivot: Option<Pivot>,

    //TODO: why is that again??
    #[bpaf(any::<_>("ANGLE", Some))]
    angle: Angle,
}

impl Command for Rotate {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        if let Some(pivot) = &self.pivot {
            state.document.rotate_around(self.angle, pivot.x, pivot.y);
        } else {
            state.document.rotate(self.angle);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Scale {
    #[bpaf(external, optional)]
    pivot: Option<Pivot>,

    #[bpaf(any::<_>("SX", Some))]
    sx: f64,

    #[bpaf(any::<_>("SY", Some), optional, catch)]
    sy: Option<f64>,
}

impl Command for Scale {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        if let Some(pivot) = &self.pivot {
            state
                .document
                .scale_around(self.sx, self.sy.unwrap_or(self.sx), pivot.x, pivot.y);
        } else if let Some(sy) = self.sy {
            state.document.scale_non_uniform(self.sx, sy);
        } else {
            state.document.scale(self.sx);
        }

        Ok(())
    }
}
