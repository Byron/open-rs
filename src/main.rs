use std::{env, process};

fn main() {
    let path_or_url = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            eprintln!("usage: open <path-or-url>");
            process::exit(1);
        }
    };

    match open::that(&path_or_url) {
        Ok(status) if status.success() => (),
        Ok(status) => match status.code() {
            Some(code) => print_error_and_exit(code, &path_or_url, &format!("error code: {}", code)),
            None => print_error_and_exit(3, &path_or_url, "error unknown"),
        },
        Err(err) => print_error_and_exit(3, &path_or_url, &err.to_string()),
    }
}

fn print_error_and_exit(code: i32, path: &str, error_message: &str) -> ! {
    eprintln!(
        "An error occurred when opening '{}': {}",
        path, error_message
    );
    process::exit(code);
}
