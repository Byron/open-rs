use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{CommandExt, IntoResult};

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let path = path.as_ref();
    [
        ("xdg-open", &[path] as &[_]),
        ("gio", &[OsStr::new("open"), path]),
        ("gnome-open", &[path]),
        ("kde-open", &[path]),
        ("wslview", &[&wsl_path(path)]),
    ]
    .iter()
    .map(|(command, args)| {
        let mut cmd = Command::new(command);
        cmd.args(*args);
        cmd
    })
    .collect()
}

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    let mut last_err = None;
    for mut command in commands(path) {
        match command.status_without_output() {
            Ok(status) => {
                return Ok(status).into_result();
            }
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.expect("no launcher worked, at least one error"))
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    Command::new(app.into())
        .arg(path.as_ref())
        .status_without_output()
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
