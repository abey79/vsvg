use bpaf::{construct, Bpaf, Parser};

use vsvg::LayerID;

use crate::commands::{make_command_parser, Command, DynCommand, State};

pub(crate) fn parser() -> impl Parser<DynCommand> {
    let layer = make_command_parser(layer());
    construct!([layer]).group_help("Context:")
}

/// Select which layer(s) to operate on
///
///
/// This command sets the target layer(s) on which subsequent command will operate. Without
/// argument, this command resets the context and subsequent commands will operate on all layers.
/// With one or more layer IDs, this command will set the context to the specified layers.
/// Note that not all commands support all contexts. Some require a single layer, while other ignore
/// it altogether.
#[derive(Clone, Debug, Bpaf)]
#[bpaf(command("layer"), adjacent)]
struct Layer {
    #[bpaf(positional("LID"), many, catch)]
    lids: Vec<LayerID>,
}

impl Command for Layer {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.layer_context = self.lids.iter().copied().collect();
        Ok(())
    }
}
