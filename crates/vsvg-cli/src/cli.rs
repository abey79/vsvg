use bpaf::{construct, short, OptionParser, Parser};

use crate::commands::{context, draw, io, ops, transforms, DynCommand, State};

struct Options {
    verbose: bool,
    commands: Vec<DynCommand>,
}

/// Parser for all possible commands.
fn command() -> impl Parser<DynCommand> {
    let context = context::parser();
    let draw = draw::parser();
    let io = io::parser();
    let ops = ops::parser();
    let transform = transforms::parser();
    construct!([context, io, ops, transform, draw])
}

/// Parser for the top-level options.
fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help("Enable verbose output")
        .switch();
    let commands = command().many();
    construct!(Options { verbose, commands }).to_options()
}

/// Run the CLI.
pub fn cli() -> anyhow::Result<()> {
    let options = options().run();

    let mut state = State::default();

    for command in options.commands {
        if options.verbose {
            println!("Executing stage: {:?}", command);
        }
        command.execute(&mut state)?;
    }

    Ok(())
}
