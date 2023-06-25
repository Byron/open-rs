use std::{env, process};

fn main() {
    let path_or_url = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            eprintln!("usage: open <path-or-url>");
            process::exit(1);
        }
    };

    #[cfg(not(windows))]
    let result = match std::env::var("OPEN_WITH").ok() {
        Some(program) => open::with(&path_or_url, program),
        None => open::that(&path_or_url),
    };

    #[cfg(windows)]
    let result = match env::args().nth(2) {
        Some(arg) if arg == "--with" || arg == "-w" => match env::args().nth(3) {
            Some(program) => open::with(&path_or_url, program),
            None => open::that(&path_or_url),
        },
        _ => open::that(&path_or_url),
    };

    match result {
        Ok(()) => println!("Opened '{}' successfully.", path_or_url),
        Err(err) => {
            eprintln!("An error occurred when opening '{}': {}", path_or_url, err);
            process::exit(3);
        }
    }
}
