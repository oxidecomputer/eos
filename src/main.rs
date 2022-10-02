#![feature(dir_entry_ext2)]

use clap::Parser;
use colored::*;
use std::io::Result;
use std::path::Path;

mod ninja;
mod spec;
mod util;

const VERSION: &str = "5.11";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

fn main() {
    let _args = Args::parse();

    if let Err(e) = run() {
        eprintln!("{} {}", "error".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let build_files = util::find_build_files(Path::new("usr/src"))?;
    let mut ninja_spec = ninja::Spec::new();
    for path in &build_files {
        ninja_spec
            .statements
            .extend(util::read_spec(path)?.to_ninja(path)?);
    }
    ninja_spec.emit_file()?;

    Ok(())
}
