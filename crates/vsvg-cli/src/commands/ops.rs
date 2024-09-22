use bpaf::{construct, Bpaf, Parser};

use vsvg::Length;

use crate::commands::{make_command_parser, Command, DynCommand, State};

pub(crate) fn parser() -> impl Parser<DynCommand> {
    let crop = make_command_parser(crop());
    construct!([crop]).group_help("Crop commands:")
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Crop {
    /// DX and DY are absolute coordinates instead of width/height
    #[bpaf(short, long)]
    absolute: bool,

    /// X coordinate of the top-left corner
    #[bpaf(positional("X"))]
    x: Length,

    /// Y coordinate of the top-left corner
    #[bpaf(positional("Y"))]
    y: Length,

    /// Width of the crop rectangle (or X coordinate of the bottom-right corner with `-a`)
    #[bpaf(positional("DX"))]
    dx: Length,

    /// Height of the crop rectangle (or Y coordinate of the bottom-right corner with `-a`)
    #[bpaf(positional("DY"))]
    dy: Length,
}

impl Command for Crop {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        let (x_max, y_max) = if self.absolute {
            (self.dx, self.dy)
        } else {
            (self.x + self.dx, self.y + self.dy)
        };

        state.document.crop(self.x, self.y, x_max, y_max);
        Ok(())
    }
}
