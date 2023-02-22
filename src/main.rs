#[macro_use]
extern crate lazy_static;

mod crop;
#[cfg(feature = "egui-viewer")]
mod egui_plot_viewer;
#[cfg(feature = "nannou-viewer")]
mod nannou_viewer;
mod svg_reader;
mod types;

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
    //#[cfg(debug_assertions)]
    //tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let doc = parse_svg(cli.path)?;
    if !cli.no_gui {
        doc.show(cli.tolerance)?;
    }

    Ok(())
}
