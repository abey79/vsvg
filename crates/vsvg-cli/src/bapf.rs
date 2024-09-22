use bpaf::{construct, short, Bpaf, OptionParser, Parser};
use std::fmt::Debug;
use std::path::PathBuf;

use vsvg::{DocumentTrait, Length, Transforms};

use crate::cli_old::State;

// pub(crate) trait Command: Debug {
//     fn execute(&self, state: &mut State);
// }
//
// /// Parser for one or more command.
// pub(crate) type DynCommand = Box<dyn Command>;
//
// pub(crate) fn make_command_parser<T: Command + 'static>(
//     parser: impl Parser<T>,
// ) -> impl Parser<DynCommand> {
//     parser.map(|t| Box::new(t) as DynCommand)
// }
//
// // ==================================================================
//
// #[derive(Clone, Debug, Bpaf)]
// #[bpaf(command, adjacent)]
// pub struct Translate {
//     #[bpaf(positional("TX"))]
//     tx: Length,
//     #[bpaf(positional("TY"))]
//     ty: Length,
// }
//
// impl Command for Translate {
//     fn execute(&self, state: &mut State) {
//         state.document.translate(self.tx, self.ty);
//     }
// }
//
// pub fn transform_parsers() -> impl Parser<DynCommand> {
//     let translate = make_command_parser(translate());
//
//     construct!([translate]).boxed()
// }

// ==================================================================

struct Options {
    verbose: bool,
    commands: Vec<DynCommand>,
}

fn command() -> impl Parser<DynCommand> {
    construct!([transform_parsers()])
}

fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Enable verbose output")
        .switch();
    let commands = command().many();
    construct!(Options { verbose, commands }).to_options()
}

pub fn bpaf_main() {
    let options = options().run();

    let mut state = State::default();

    for command in options.commands {
        if options.verbose {
            println!("Executing stage: {:?}", command);
        }
        command.execute(&mut state);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_dayer() {
        bpaf_main();
    }
}

//
//
// #[derive(Debug, Clone, Bpaf)]
// #[bpaf(options)]
// struct Args {
//     // /// Message to print in a big friendly letters
//     // #[bpaf(positional("MESSAGE"))]
//     // message: String,
//     #[bpaf(external, many)]
//     command: Vec<Command>,
// }
//
// #[derive(Debug, Clone, Bpaf)]
// enum Command {
//     #[bpaf(command, adjacent)]
//     Read {
//         #[bpaf(positional("PATH"))]
//         path: PathBuf,
//
//         single_layer: bool,
//     },
//     #[bpaf(command, adjacent)]
//     Write {
//         #[bpaf(positional("PATH"))]
//         path: PathBuf,
//     },
//     #[bpaf(command, adjacent)]
//     Translate {
//         #[bpaf(positional("TX"))]
//         tx: Length,
//         #[bpaf(positional("TY"))]
//         ty: Length,
//     },
//
//     #[bpaf(command, adjacent)]
//     Scale {
//         #[bpaf(external, optional)]
//         origin: Option<Origin>,
//
//         #[bpaf(positional("SX"))]
//         sx: f64,
//
//         #[bpaf(positional("SY"))]
//         sy: Option<f64>,
//     },
// }
//
// #[derive(Debug, Clone, Bpaf)]
// #[bpaf(adjacent)]
// struct Origin {
//     #[bpaf(short, long)]
//     origin: (),
//     #[bpaf(positional("X"))]
//     x: Length,
//     #[bpaf(positional("Y"))]
//     y: Length,
// }
//
// pub fn bpaf_main() {
//     let args = args().run();
//
//     execute_commands(&args.command);
// }
//
// pub fn execute_commands(cmds: &[Command]) -> anyhow::Result<State> {
//     let mut state = State::default();
//
//     for cmd in cmds {
//         println!("{cmd:?}");
//
//         match cmd {
//             Command::Read { path, single_layer } => {
//                 //TODO: proper error management!!!!
//                 let new_doc = vsvg::Document::from_svg(path, *single_layer).unwrap();
//                 //TODO: merge instead of replace + some kind of metadata merge policy
//                 state.document = new_doc;
//             }
//             Command::Write { path } => {
//                 //TODO: proper error management!!!}
//                 state.document.to_svg_file(path).unwrap();
//             }
//             Command::Translate { tx, ty } => {
//                 state.document.translate(tx, ty);
//             }
//             Command::Scale { sx, sy, origin } => {
//                 if let Some(origin) = origin {
//                     state
//                         .document
//                         .scale_around(*sx, sy.clone().unwrap_or(*sx), origin.x, origin.y);
//                 } else if let Some(sy) = sy {
//                     state.document.scale_non_uniform(*sx, *sy);
//                 } else {
//                     state.document.scale(*sx);
//                 }
//             }
//         }
//     }
//
//     Ok(state)
// }
