use bpaf::{construct, Bpaf, Parser};

use vsvg::{DocumentTrait as _, LayerID, LayerTrait as _};

use crate::commands::{make_command_parser, Command, DynCommand, State};

pub(crate) fn parser() -> impl Parser<DynCommand> {
    let layer_delete = make_command_parser(layer_delete());
    let layer_copy = make_command_parser(layer_copy());
    construct!([layer_delete, layer_copy]).group_help("Layers:")
}

// TODO: LIDs passed as arguement for convenience—it's weird to use `layer 1 2 3 ldelete`. Maybe
// there should be a "generic" delete command (which uses the context) along these layer-spacific
// ones.

/// Delete the specified layers
///
/// This command deletes everything if no layers are specified at all.
#[derive(Clone, Debug, Bpaf)]
#[bpaf(command("ldelete"), adjacent)]
struct LayerDelete {
    /// Delete the layers selected by the `layer` command
    #[bpaf(short, long)]
    selected: bool,

    /// Layer(s) to delete (ignored if `--selected` is used)
    #[bpaf(positional("LID"), many, catch)]
    lids: Vec<LayerID>,
}

impl Command for LayerDelete {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        if self.selected {
            //TODO: warn if `lids` is not empty

            for layer_id in state.layers() {
                state.document.remove(layer_id);
            }
        } else if self.lids.is_empty() {
            state.document.clear();
        } else {
            for layer_id in &self.lids {
                state.document.remove(*layer_id);
            }
        }

        Ok(())
    }
}

/// Copy the specified layer(s) to the target layer
#[derive(Clone, Debug, Bpaf)]
#[bpaf(command("lcopy"), adjacent)]
struct LayerCopy {
    #[bpaf(positional("TARGET"))]
    target: LayerID,
}

impl Command for LayerCopy {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        let mut destination_layer = state.document.get_mut(self.target).clone();

        state.for_layer(|layer, _| {
            destination_layer.merge(layer);
            Ok(())
        })?;

        *state.document.get_mut(self.target) = destination_layer;

        Ok(())
    }
}

//TODO: lmerge
//TODO: lmove
