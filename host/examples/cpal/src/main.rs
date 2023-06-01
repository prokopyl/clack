extern crate core;

mod buffers;
mod discovery;
mod host;
mod stream;

use clap::Parser;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'f')]
    bundle_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match (&args.bundle_file, ()) {
        (Some(f), _) => run_from_path(&f)?,
        _ => {}
    }

    Ok(())
    // discovery::scan_for_plugin_id("foo");
}

fn run_from_path(path: &Path) -> Result<(), Box<dyn Error>> {
    let plugins = discovery::list_plugins_in_bundle(path)?;

    println!(
        "Found {} plugins in CLAP bundle: {}",
        plugins.len(),
        path.display()
    );

    for x in &plugins {
        println!("\t {x}")
    }

    if plugins.len() == 1 {
        host::run(path, &plugins[0].id)
    } else {
        Ok(())
    }
}
