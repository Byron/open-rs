use std::{ffi::OsStr, io, process::Command};

use crate::{CommandExt, IntoResult};

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    Command::new("/bin/open")
        .arg(path.as_ref())
        .without_io()
        .status()
        .into_result()
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    Command::new(app.into())
        .arg(path.as_ref())
        .without_io()
        .status()
        .into_result()
}

pub fn commands<T: AsRef<OsStr>>(path: T) -> impl Iterator<Item = Command> {
    let mut cmd = Command::new("/bin/open");
    cmd.arg(path.as_ref());
    Some(cmd).into_iter()
}
