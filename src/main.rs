use std::env;
use std::process;

mod tree2d;

use corral::{run, Config};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::parse(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = run(config) {
        println!("Application error {e}");
        process::exit(1);
    }
}
