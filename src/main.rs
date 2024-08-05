use std::process;

use config::*;

pub mod config;

fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        eprintln!("Problem building config: {err}");
        process::exit(1);
    });

    println!("Config parsed: {:?}", config); // will be deleted soon
}
