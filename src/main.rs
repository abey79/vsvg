mod cli;
mod commands;
mod svg_reader;
mod types;
mod viewer;

#[macro_use]
mod test_utils;

use crate::commands::command_list;
use crate::svg_reader::*;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let commands = command_list();
    let mut matches = cli::cli(&commands).get_matches();

    // remove global args
    let path = matches
        .remove_one::<PathBuf>("PATH")
        .expect("PATH is a required arg");
    let no_show = matches.remove_one::<bool>("no-show").unwrap();
    let verbose = matches.remove_one::<bool>("verbose").unwrap();

    if verbose {
        tracing_subscriber::fmt::init();
    }

    // create and process document
    let mut doc = parse_svg(path)?;
    let values = cli::CommandValue::from_matches(&matches, &commands);
    for (id, value) in values.iter() {
        let command_desc = commands.get(id).expect("id came from matches");
        doc = (command_desc.action)(value, doc)?;
    }

    // display gui
    if !no_show {
        doc.show(0.1)?;
    }

    Ok(())
}
