use std::{ffi::OsStr, io, process::Command};

use crate::{CommandExt, IntoResult};

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg("start").arg(path.as_ref());
    vec![cmd]
}

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    let cmd = &mut commands(path)[0];
    cmd.status_without_output().into_result(cmd)
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c")
        .arg(app.into())
        .arg(path.as_ref())
        .status_without_output()
        .into_result(&cmd)
}
