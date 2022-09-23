use std::env;
use std::process;

mod bounding_box;
mod config;
mod packer;
mod tree2d;

use config::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::parse(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = packer::run(config) {
        println!("Application error {e}");
        process::exit(1);
    }
}
