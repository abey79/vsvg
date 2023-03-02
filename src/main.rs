#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

mod cli;
mod commands;
pub mod types;
mod viewer;

#[macro_use]
mod test_utils;

use crate::commands::command_list;
use crate::types::Document;
use std::error::Error;
use std::io::Read;
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
    let mut doc = if path == PathBuf::from("-") {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Document::from_string(s.as_str())?
    } else {
        Document::from_svg(path)?
    };
    let values = cli::CommandValue::from_matches(&matches, &commands);
    for (id, value) in &values {
        let command_desc = commands.get(id).expect("id came from matches");
        doc = (command_desc.action)(value, doc)?;
    }

    // display gui
    if !no_show {
        doc.show(0.1)?;
    }

    Ok(())
}
