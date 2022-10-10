#![feature(path_file_prefix)]

use std::{env, error::Error, process};

mod bounding_box;
mod config;
mod packer;
mod tree2d;

use config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let config = Config::parse(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    packer::run(config)
}
