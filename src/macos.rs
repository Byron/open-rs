use std::{ffi::OsStr, io, process::Command};

use crate::{CommandExt, IntoResult};

pub fn command<T: AsRef<OsStr>>(path: T) -> Command {
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg(path.as_ref());
    cmd
}

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    command(path).status_without_output().into_result()
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    Command::new("/usr/bin/open")
        .arg(path.as_ref())
        .arg("-a")
        .arg(app.into())
        .status_without_output()
        .into_result()
}
