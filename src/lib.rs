//! Use this library to open a path or URL using the program configured on the system.
//!
//! # Usage
//!
//! Open the given URL in the default web browser.
//!
//! ```no_run
//! open::that("http://rust-lang.org").unwrap();
//! ```
//!
//! Alternatively, specify the program to be used to open the path or URL.
//!
//! ```no_run
//! open::with("http://rust-lang.org", "firefox").unwrap();
//! ```
//!
//! # Notes
//!
//! As an operating system program is used, the open operation can fail.
//! Therefore, you are advised to at least check the result and behave
//! accordingly, e.g. by letting the user know that the open operation failed.
//!
//! ```no_run
//! let path = "http://rust-lang.org";
//!
//! match open::that(path) {
//!     Ok(()) => println!("Opened '{}' successfully.", path),
//!     Err(err) => eprintln!("An error occurred when opening '{}': {}", path, err),
//! }
//! ```

#[cfg(target_os = "windows")]
use windows as os;

#[cfg(target_os = "macos")]
use macos as os;

#[cfg(target_os = "ios")]
use ios as os;

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris"
))]
use unix as os;

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
)))]
compile_error!("open is not supported on this platform");

use std::{
    ffi::OsStr,
    io,
    process::{Command, Output, Stdio},
    thread,
};

type Result = io::Result<()>;

/// Open path with the default application.
///
/// # Examples
///
/// ```no_run
/// let path = "http://rust-lang.org";
///
/// match open::that(path) {
///     Ok(()) => println!("Opened '{}' successfully.", path),
///     Err(err) => panic!("An error occurred when opening '{}': {}", path, err),
/// }
/// ```
///
/// # Errors
///
/// A [`std::io::Error`] is returned on failure. Because different operating systems
/// handle errors differently it is recommend to not match on a certain error.
pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> Result {
    os::that(path)
}

/// Open path with the given application.
///
/// # Examples
///
/// ```no_run
/// let path = "http://rust-lang.org";
/// let app = "firefox";
///
/// match open::with(path, app) {
///     Ok(()) => println!("Opened '{}' successfully.", path),
///     Err(err) => panic!("An error occurred when opening '{}': {}", path, err),
/// }
/// ```
///
/// # Errors
///
/// A [`std::io::Error`] is returned on failure. Because different operating systems
/// handle errors differently it is recommend to not match on a certain error.
pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> Result {
    os::with(path, app)
}

/// Open path with the default application in a new thread.
///
/// See documentation of [`that`] for more details.
pub fn that_in_background<T: AsRef<OsStr> + Sized>(path: T) -> thread::JoinHandle<Result> {
    let path = path.as_ref().to_os_string();
    thread::spawn(|| that(path))
}

/// Open path with the given application in a new thread.
///
/// See documentation of [`with`] for more details.
pub fn with_in_background<T: AsRef<OsStr> + Sized>(
    path: T,
    app: impl Into<String>,
) -> thread::JoinHandle<Result> {
    let path = path.as_ref().to_os_string();
    let app = app.into();
    thread::spawn(|| with(path, app))
}

trait IntoResult<T> {
    fn into_result(self) -> T;
}

impl IntoResult<Result> for io::Result<Output> {
    fn into_result(self) -> Result {
        match self {
            Ok(o) if o.status.success() => Ok(()),
            Ok(o) => Err(from_output(o)),
            Err(err) => Err(err),
        }
    }
}

#[cfg(windows)]
impl IntoResult<Result> for winapi::ctypes::c_int {
    fn into_result(self) -> Result {
        match self {
            i if i > 32 => Ok(()),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

fn from_output(output: Output) -> io::Error {
    let error_msg = match output.stderr.is_empty() {
        true => output.status.to_string(),
        false => format!(
            "{} ({})",
            String::from_utf8_lossy(&output.stderr).trim(),
            output.status
        ),
    };

    io::Error::new(io::ErrorKind::Other, error_msg)
}

trait CommandExt {
    fn output_stderr(&mut self) -> io::Result<Output>;
}

impl CommandExt for Command {
    fn output_stderr(&mut self) -> io::Result<Output> {
        let mut process = self
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        // Consume all stderr - it's open just for a few programs which can't handle it being closed.
        use std::io::Read;
        let mut stderr = vec![0; 256];
        let mut stderr_src = process.stderr.take().expect("piped stderr");

        let len = stderr_src.read(&mut stderr).unwrap_or(0);
        stderr.truncate(len);

        // consume the rest to avoid blocking
        std::io::copy(&mut stderr_src, &mut std::io::sink()).ok();

        let status = process.wait()?;
        Ok(Output {
            status,
            stderr,
            stdout: vec![],
        })
    }
}

#[cfg(windows)]
mod windows {
    use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, ptr};

    use winapi::ctypes::c_int;
    use winapi::um::shellapi::ShellExecuteW;

    use crate::{IntoResult, Result};

    fn convert_path(path: &OsStr) -> io::Result<Vec<u16>> {
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

    pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> Result {
        const SW_SHOW: c_int = 5;

        let path = convert_path(path.as_ref())?;
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
        (result as c_int).into_result()
    }

    pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> Result {
        const SW_SHOW: c_int = 5;

        let path = convert_path(path.as_ref())?;
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
        (result as c_int).into_result()
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::{ffi::OsStr, process::Command};

    use crate::{CommandExt, IntoResult, Result};

    pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> Result {
        Command::new("/usr/bin/open")
            .arg(path.as_ref())
            .output_stderr()
            .into_result()
    }

    pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> Result {
        Command::new("/usr/bin/open")
            .arg(path.as_ref())
            .arg("-a")
            .arg(app.into())
            .output_stderr()
            .into_result()
    }
}

#[cfg(target_os = "ios")]
mod ios {
    use std::{ffi::OsStr, process::Command};

    use crate::{CommandExt, IntoResult, Result};

    pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> Result {
        Command::new("uiopen")
            .arg("--url")
            .arg(path.as_ref())
            .output_stderr()
            .into_result()
    }

    pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> Result {
        Command::new("uiopen")
            .arg("--url")
            .arg(path.as_ref())
            .arg("--bundleid")
            .arg(app.into())
            .output_stderr()
            .into_result()
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris"
))]
mod unix {
    use std::{
        env,
        ffi::{OsStr, OsString},
        path::{Path, PathBuf},
        process::Command,
    };

    use crate::{CommandExt, IntoResult, Result};

    pub fn that<T: AsRef<OsStr> + Sized>(path: T) -> Result {
        let path = path.as_ref();
        let open_handlers = [
            ("xdg-open", &[path] as &[_]),
            ("gio", &[OsStr::new("open"), path]),
            ("gnome-open", &[path]),
            ("kde-open", &[path]),
            ("wslview", &[&wsl_path(path)]),
        ];

        let mut unsuccessful = None;
        let mut io_error = None;

        for (command, args) in &open_handlers {
            let result = Command::new(command).args(*args).output_stderr();

            match result {
                Ok(o) if o.status.success() => return Ok(()),
                Ok(o) => unsuccessful = unsuccessful.or_else(|| Some(crate::from_output(o))),
                Err(err) => io_error = io_error.or(Some(err)),
            }
        }

        Err(unsuccessful
            .or(io_error)
            .expect("successful cases don't get here"))
    }

    pub fn with<T: AsRef<OsStr> + Sized>(path: T, app: impl Into<String>) -> Result {
        Command::new(app.into())
            .arg(path.as_ref())
            .output_stderr()
            .into_result()
    }

    // Polyfill to workaround absolute path bug in wslu(wslview). In versions before
    // v3.1.1, wslview is unable to find absolute paths. `wsl_path` converts an
    // absolute path into a relative path starting from the current directory. If
    // the path is already a relative path or the conversion fails the original path
    // is returned.
    fn wsl_path<T: AsRef<OsStr>>(path: T) -> OsString {
        fn path_relative_to_current_dir<T: AsRef<OsStr>>(path: T) -> Option<PathBuf> {
            let path = Path::new(&path);

            if path.is_relative() {
                return None;
            }

            let base = env::current_dir().ok()?;
            pathdiff::diff_paths(path, base)
        }

        match path_relative_to_current_dir(&path) {
            None => OsString::from(&path),
            Some(relative) => OsString::from(relative),
        }
    }
}
