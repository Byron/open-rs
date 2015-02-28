#![feature(old_io)]
//! Use this library to open a path or URL using the program configured on the system.
//!
//! # Usage
//!
//! ```test_harness,no_run
//! extern crate open;
//! 
//! # #[test]
//! # fn doit() {
//! open::that("/path/to/file/with.fancy-extension");
//! if open::that("https://google.de").is_ok() {
//!     println!("Look at your browser !");
//! }
//! # }
//! ```
//!
//! # Notes
//! As an operating system program is used, chances are that the open operation fails.
//! Therfore, you are advised to at least check the result with `.is_err()` and 
//! behave accordingly, e.g. by letting the user know what you tried to open, and failed.
use std::old_io::IoResult;
use std::old_io::process::{Command, ProcessExit};

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn that(path: &str) -> IoResult<ProcessExit> {
    use std::old_io::IoError;

    let mut res = Err(IoError::from_errno(0, false));
    for program in &["xdg-open", "gnome-open", "kde-open"] {
        res = Command::new(program).arg(path).detached().status();
        match res {
            Ok(_) => return res,
            Err(_) => continue,
        }
    }
    res
}

#[cfg(target_os = "windows")]
pub fn that(path: &str) -> IoResult<ProcessExit> {
    Command::new("start").arg(path).detached().status()
}

#[cfg(target_os = "macos")]
pub fn that(path: &str) -> IoResult<ProcessExit> {
    Command::new("open").arg(path).detached().status()
}