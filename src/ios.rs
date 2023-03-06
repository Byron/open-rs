use std::{ffi::OsStr, io, process::Command};

use crate::{CommandExt, IntoResult};

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let mut cmd = Command::new("uiopen");
    cmd.arg("--url").arg(path.as_ref());
    vec![cmd]
}

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    commands(path)[0].status_without_output().into_result()
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    Command::new("uiopen")
        .arg("--url")
        .arg(path.as_ref())
        .arg("--bundleid")
        .arg(app.into())
        .status_without_output()
        .into_result()
}
