mod crop;
mod svg_reader;
mod types;
mod viewer;

use crate::svg_reader::*;
use std::error::Error;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    /// path of input SVG file
    path: PathBuf,

    /// return after loading the SVG file without showing the GUI (for benchmarking)
    #[clap(short, long)]
    no_gui: bool,

    /// tolerance when displaying SVG curves
    #[clap(short, long, default_value = "0.01")]
    tolerance: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let doc = parse_svg(cli.path)?;
    if !cli.no_gui {
        doc.show(cli.tolerance)?;
    }

    Ok(())
}
