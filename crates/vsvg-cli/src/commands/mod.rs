use crate::draw_state::{DrawState, LayerDrawer};
use bpaf::Parser;
use std::collections::BTreeSet;
use std::fmt::Debug;
use vsvg::{Document, DocumentTrait, Layer, LayerID};

pub(crate) mod context;
pub(crate) mod draw;
pub(crate) mod io;
pub(crate) mod ops;
pub(crate) mod transforms;
pub(crate) mod utils;

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum CommandError {
    #[error("Expected a single layer in context (see `layer` command)")]
    ExpectSingleLayer,
}

pub(crate) struct State {
    pub document: Document,
    pub draw_state: DrawState,
    //pub draw_layer: LayerID,
    /// Current layer context.
    ///
    /// All layers active when empty.
    pub layer_context: BTreeSet<LayerID>,
}

impl State {
    // pub(crate) fn draw(&mut self) -> Result<LayerDrawer, CommandError> {
    //
    //     LayerDrawer {
    //         state: &self.draw_state,
    //         layer: self.document.get_mut(self.draw_layer),
    //     }
    // }

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

    pub(crate) fn single_layer<R>(
        &mut self,
        func: impl FnOnce(&mut Layer, LayerID) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        self.check_single_layer(|state, layer_id| func(state.document.get_mut(layer_id), layer_id))
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

impl Default for State {
    fn default() -> Self {
        Self {
            //draw_layer: 1,
            draw_state: DrawState::default(),
            document: Document::default(),
            layer_context: BTreeSet::new(),
        }
    }
}

pub(crate) trait Command: Debug {
    fn execute(&self, state: &mut State) -> anyhow::Result<()>;
}

/// Parser for one or more command.
pub(crate) type DynCommand = Box<dyn Command>;

/// Transform a concrete [`Command`] parser into a genric [`DynCommand`] parser.
pub(crate) fn make_command_parser<T: Command + 'static>(
    parser: impl Parser<T>,
) -> impl Parser<DynCommand> {
    parser.map(|t| Box::new(t) as DynCommand)
}
