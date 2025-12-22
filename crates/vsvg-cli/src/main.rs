mod cli;
mod commands;
mod draw_state;

use crate::commands::command_list;

use std::error::Error;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use crate::cli::State;
use vsvg::Document;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let commands = command_list();
    let mut matches = cli::cli(&commands).get_matches();

    // remove global args
    let path = matches
        .remove_one::<PathBuf>("PATH")
        .expect("PATH is a required arg");
    let no_show = matches.remove_one::<bool>("no-show").unwrap();
    let verbose = matches.remove_one::<bool>("verbose").unwrap();
    let single_layer = matches.remove_one::<bool>("single-layer").unwrap();

    if verbose {
        tracing_subscriber::fmt::init();
    }

    // create and process document
    let mut state = State {
        document: if path.as_os_str() == "-" {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            Document::from_string(s.as_str(), single_layer)?
        } else {
            Document::from_svg(path, single_layer)?
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
        vsvg_viewer::show(Arc::new(state.document))?;
    }

    Ok(())
}
