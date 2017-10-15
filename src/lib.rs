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
//! if open::that("https://rust-lang.org").is_ok() {
//!     println!("Look at your browser !");
//! }
//! # }
//! ```
//!
//! # Notes
//! As an operating system program is used, chances are that the open operation fails.
//! Therfore, you are advised to at least check the result with `.is_err()` and
//! behave accordingly, e.g. by letting the user know what you tried to open, and failed.
use std::io;
use std::process::{Command, ExitStatus};
use std::ffi::OsStr;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<ExitStatus> {
    let mut last_err: io::Result<ExitStatus> = Err(io::Error::from_raw_os_error(0));
    for program in &["xdg-open", "gnome-open", "kde-open"] {
        match Command::new(program).arg(path.as_ref()).spawn() {
            Ok(mut child) => return child.wait(),
            Err(err) => {
                last_err = Err(err);
                continue;
            },
        }
    }
    last_err
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn with<T:AsRef<OsStr>+Sized>(path: T, app: &str) -> io::Result<ExitStatus> {
    Command::new(app)
        .arg(path.as_ref())
        .spawn()?
        .wait()?
}

#[cfg(target_os = "windows")]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<ExitStatus> {
    try!(Command::new("cmd").arg("/C").arg("start").arg(path.as_ref()).spawn()).wait()
}

#[cfg(target_os = "windows")]
pub fn with<T:AsRef<OsStr>+Sized>(path: T, app: &str) -> io::Result<ExitStatus> {
    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg("/b")
        .arg(app)
        .arg(path.as_ref())
        .spawn()?
        .wait()
}

#[cfg(target_os = "macos")]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<ExitStatus> {
    try!(Command::new("open").arg(path.as_ref()).spawn()).wait()
}

#[cfg(target_os = "macos")]
pub fn with<T:AsRef<OsStr>+Sized>(path: T, app: &str) -> io::Result<ExitStatus> {
    Command::new("open")
        .arg(path.as_ref())
        .arg("-a")
        .arg(app)
        .spawn()?
        .wait()
}
