use std::collections::BTreeSet;
use std::fmt::Debug;

use bpaf::Parser;

use vsvg::{Document, DocumentTrait, Layer, LayerID};

use crate::draw_state::{DrawState, LayerDrawer};

pub(crate) mod context;
pub(crate) mod draw;
pub(crate) mod io;
pub(crate) mod layers;
pub(crate) mod ops;
pub(crate) mod transforms;
pub(crate) mod utils;

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum CommandError {
    #[error("Expected a single layer in context (see `layer` command)")]
    ExpectSingleLayer,
}

#[derive(Default, Debug)]
pub(crate) struct State {
    pub document: Document,
    pub draw_state: DrawState,

    /// Current layer context.
    ///
    /// When empty, all layers are active.
    pub layer_context: BTreeSet<LayerID>,
}

impl State {
    fn check_single_layer<R>(
        &mut self,
        func: impl FnOnce(&mut Self, LayerID) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let layer_id = self
            .layer_context
            .first()
            .copied()
            .ok_or(CommandError::ExpectSingleLayer)?;

        if self.layer_context.len() > 1 {
            return Err(CommandError::ExpectSingleLayer.into());
        }

        func(self, layer_id)
    }

    //TODO: revive if needed
    // pub(crate) fn single_layer<R>(
    //     &mut self,
    //     func: impl FnOnce(&mut Layer, LayerID) -> anyhow::Result<R>,
    // ) -> anyhow::Result<R> {
    //     self.check_single_layer(|state, layer_id| func(state.document.get_mut(layer_id), layer_id))
    // }

    /// Get the selected layers.
    pub(crate) fn layers(&self) -> Vec<LayerID> {
        if self.layer_context.is_empty() {
            self.document.layers().keys().copied().collect()
        } else {
            self.layer_context.iter().copied().collect()
        }
    }

    //TODO: this creates selected layer than don't exists, not always desirable!
    pub(crate) fn iter_layers(&mut self) -> impl Iterator<Item = (&mut Layer, LayerID)> + '_ {
        self.document
            .layers_mut()
            .iter_mut()
            .filter_map(|(id, layer)| {
                if self.layer_context.is_empty() || self.layer_context.contains(id) {
                    Some((layer, *id))
                } else {
                    None
                }
            })
    }

    pub(crate) fn for_layer(
        &mut self,
        mut func: impl FnMut(&mut Layer, LayerID) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for (layer, layer_id) in self.iter_layers() {
            func(layer, layer_id)?;
        }

        Ok(())
    }

    pub(crate) fn draw<R>(
        &mut self,
        func: impl FnOnce(&mut LayerDrawer) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        self.check_single_layer(|state, layer_id| {
            func(&mut LayerDrawer {
                state: &state.draw_state,
                layer: state.document.get_mut(layer_id),
            })
        })
    }
}

pub(crate) trait Command: Debug {
    fn execute(&self, state: &mut State) -> anyhow::Result<()>;
}

/// Parser for one or more command.
pub(crate) type DynCommand = Box<dyn Command>;

/// Transform a concrete [`Command`] parser into a generic [`DynCommand`] parser.
pub(crate) fn make_command_parser<T: Command + 'static>(
    parser: impl Parser<T>,
) -> impl Parser<DynCommand> {
    parser.map(|t| Box::new(t) as DynCommand)
}
