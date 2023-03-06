use std::{ffi::OsStr, io, process::Command};

use crate::{CommandExt, IntoResult};

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg(path.as_ref());
    vec![cmd]
}

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    let cmd = &mut commands(path)[0];
    cmd.status_without_output().into_result(cmd)
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg(path.as_ref())
        .arg("-a")
        .arg(app.into())
        .status_without_output()
        .into_result(&cmd)
}
