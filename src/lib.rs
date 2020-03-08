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
//! Alternatively, specify the program to open something with. It should expect to receive the path or URL as first argument.
//! ```test_harness,no_run
//! extern crate open;
//!
//! # #[test]
//! # fn doit() {
//! open::with("http://rust-lang.org", "firefox").unwrap();
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
#[cfg(windows)]
extern crate winapi;

#[cfg(not(windows))]
use std::process::{Command, Stdio};

use std::{ffi::OsStr, io, process::ExitStatus, thread};

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> io::Result<ExitStatus> {
    let path_ref = path.as_ref();
    let mut last_err: io::Error = io::Error::from_raw_os_error(0);
    for program in &["xdg-open", "gnome-open", "kde-open", "wslview"] {
        match Command::new(program)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .arg(path_ref)
            .spawn()
        {
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
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::process::ExitStatusExt;
    use std::ptr;
    use winapi::ctypes::c_int;
    use winapi::um::shellapi::ShellExecuteW;

    const SW_SHOW: c_int = 5;

    let path = windows::convert_path(path.as_ref())?;
    let operation: Vec<u16> = OsStr::new("open\0").encode_wide().collect();
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            path.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOW,
        )
    };
    if result as c_int > 32 {
        Ok(ExitStatus::from_raw(0))
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(target_os = "macos")]
pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> io::Result<ExitStatus> {
    Command::new("open")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .arg(path.as_ref())
        .spawn()?
        .wait()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> io::Result<ExitStatus> {
    Command::new(app.into()).arg(path.as_ref()).spawn()?.wait()
}

#[cfg(target_os = "windows")]
pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> io::Result<ExitStatus> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::process::ExitStatusExt;
    use std::ptr;
    use winapi::ctypes::c_int;
    use winapi::um::shellapi::ShellExecuteW;

    const SW_SHOW: c_int = 5;

    let path = windows::convert_path(path.as_ref())?;
    let operation: Vec<u16> = OsStr::new("open\0").encode_wide().collect();
    let app_name: Vec<u16> = OsStr::new(&format!("{}\0", app.into()))
        .encode_wide()
        .collect();
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            app_name.as_ptr(),
            path.as_ptr(),
            ptr::null(),
            SW_SHOW,
        )
    };
    if result as c_int > 32 {
        Ok(ExitStatus::from_raw(0))
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(target_os = "macos")]
pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> io::Result<ExitStatus> {
    Command::new("open")
        .arg(path.as_ref())
        .arg("-a")
        .arg(app.into())
        .spawn()?
        .wait()
}

/// Convenience function for opening the passed path in a new thread.
/// See documentation of `that(...)` for more details.
pub fn that_in_background<T: AsRef<OsStr> + Sized>(
    path: T,
) -> thread::JoinHandle<io::Result<ExitStatus>> {
    let path = path.as_ref().to_os_string();
    thread::spawn(|| that(path))
}

pub fn with_in_background<T: AsRef<OsStr> + Sized>(
    path: T,
    app: impl Into<String>,
) -> thread::JoinHandle<io::Result<ExitStatus>> {
    let path = path.as_ref().to_os_string();
    let app = app.into();
    thread::spawn(|| with(path, app))
}

#[cfg(windows)]
mod windows {
    use std::ffi::OsStr;
    use std::io;
    use std::os::windows::ffi::OsStrExt;

    pub fn convert_path(path: &OsStr) -> io::Result<Vec<u16>> {
        let mut maybe_result: Vec<_> = path.encode_wide().collect();
        if maybe_result.iter().any(|&u| u == 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path contains NUL byte(s)",
            ));
        }
        maybe_result.push(0);
        Ok(maybe_result)
    }
}
