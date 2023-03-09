mod cli;
mod commands;
mod frame_history;
mod viewer;

use crate::commands::command_list;
use crate::viewer::Show;

use std::error::Error;
use std::io::Read;
use std::path::PathBuf;

use crate::cli::State;
use vsvg_core::Document;

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
    let mut state = State {
        document: if path == PathBuf::from("-") {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            Document::from_string(s.as_str())?
        } else {
            Document::from_svg(path)?
        },
        ..Default::default()
    };

    let values = cli::CommandValue::from_matches(&matches, &commands);
    for (id, value) in &values {
        let command_desc = commands.get(id).expect("id came from matches");
        (command_desc.action)(value, &mut state)?;
    }

    // display gui
    if !no_show {
        state.document.show(0.1)?;
    }

    Ok(())
}
