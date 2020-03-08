extern crate open;

use std::{
    env,
    io::{stderr, Write},
    process,
};

fn main() {
    let path_or_url = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            writeln!(stderr(), "usage: open <path-or-url>").ok();
            process::exit(1);
        }
    };

    if let Err(err) = open::that(&path_or_url) {
        writeln!(
            stderr(),
            "An error occourred when opening '{}': {}",
            path_or_url,
            err
        )
        .ok();
        process::exit(3);
    }
}
