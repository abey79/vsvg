#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

use std::path::{Path, PathBuf};

use camino::Utf8PathBuf;
use clap::Parser;

use vsvg_viewer::show_with_viewer_app;

mod app;

fn visit_file(file: PathBuf, paths: &mut Vec<Utf8PathBuf>) -> anyhow::Result<()> {
    if file.extension() == Some("svg".as_ref()) {
        paths.push(file.try_into()?);
    }

    Ok(())
}

fn visit_dir(dir: &Path, paths: &mut Vec<Utf8PathBuf>) -> anyhow::Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, paths)?;
        } else {
            visit_file(path, paths)?;
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// SVG files or directories
    #[arg(required = true, num_args = 1..)]
    paths: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut svg_list = Vec::new();

    for path in args.paths {
        if path.is_dir() {
            visit_dir(&path, &mut svg_list)?;
        } else {
            visit_file(path, &mut svg_list)?;
        }
    }

    show_with_viewer_app(app::App::from_paths(svg_list))
}
