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
//!             println!("Look at your browser!");
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
use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> io::Result<ExitStatus> {
    let path_ref = path.as_ref();
    let mut last_err: io::Error = io::Error::from_raw_os_error(0);
    for program in &["xdg-open", "gnome-open", "kde-open"] {
        match Command::new(program).arg(path_ref).spawn() {
            Ok(mut child) => return child.wait(),
            Err(err) => {
                last_err = err;
                continue;
            }
        }
    }
    Err(last_err)
}

#[cfg(target_os = "windows")]
pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> io::Result<ExitStatus> {
    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("")
        .arg({
            let path_ref = path.as_ref();
            match path_ref.to_str() {
                Some(s) => s.replace("&", "^&"),
                None => path_ref,
            }
        })
        .spawn()?
        .wait()
}

#[cfg(target_os = "macos")]
pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> io::Result<ExitStatus> {
    Command::new("open").arg(path.as_ref()).spawn()?.wait()
}
