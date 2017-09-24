//! Use this library to open a path or URL using the program configured on the system.
//!
//! # Usage
//!
//! The following should open the given URL in a web browser
//!
//! ```test_harness,no_run
//! extern crate open;
//!
//! # #[test]
//! # fn doit() {
//! open::that("http://rust-lang.org").unwrap();
//! # }
//! ```
//!
//! # Notes
//!
//! As an operating system program is used, chances are that the open operation fails.
//! Therefore, you are advised to at least check the result with `.is_err()` and
//! behave accordingly, e.g. by letting the user know what you tried to open, and failed.
//!
//! ```
//! # fn doit() {
//! match open::that("http://rust-lang.org") {
//!     Ok(exit_status) => {
//!         if exit_status.success() {
//!             println!("Look at your browser !");
//!         } else {
//!             if let Some(code) = exit_status.code() {
//!                 println!("Command returned non-zero exit status {}!", code);
//!             } else {
//!                 println!("Command returned with unknown exit status!");
//!             }
//!         }
//!     }
//!     Err(why) => println!("Failure to execute command: {}", why),
//! }
//! # }
//! ```
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

#[cfg(target_os = "windows")]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<ExitStatus> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg("start").arg("");
    if let Some(s) = path.as_ref().to_str() {
        cmd.arg(s.replace("&", "^&"));
    } else {
        cmd.arg(path.as_ref());
    }
    try!(cmd.spawn()).wait()
}

#[cfg(target_os = "macos")]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<ExitStatus> {
    try!(Command::new("open").arg(path.as_ref()).spawn()).wait()
}
