use std::path::PathBuf;
use std::sync::Arc;

use bpaf::{construct, Bpaf, Parser};

use vsvg::DocumentTrait;

use crate::commands::{make_command_parser, Command, DynCommand, State};

pub(crate) fn parser() -> impl Parser<DynCommand> {
    let read = make_command_parser(read());
    let write = make_command_parser(write());
    let show = make_command_parser(show());

    construct!([read, write, show]).group_help("I/O commands:")
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Read {
    #[bpaf(short, long)]
    single_layer: bool,

    #[bpaf(positional("PATH"))]
    path: PathBuf,
}

impl Command for Read {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        let new_doc = vsvg::Document::from_svg(&self.path, self.single_layer).unwrap();
        state.document = new_doc;
        Ok(())
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Write {
    #[bpaf(positional("PATH"))]
    path: PathBuf,
}

impl Command for Write {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        state.document.to_svg_file(&self.path)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Bpaf)]
#[bpaf(command, adjacent)]
struct Show {}

impl Command for Show {
    fn execute(&self, state: &mut State) -> anyhow::Result<()> {
        vsvg_viewer::show(Arc::new(state.document.clone()))
    }
}
