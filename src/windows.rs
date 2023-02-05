use std::{ffi::OsStr, io};

use crate::{CommandExt, IntoResult};

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    std::process::Command::new("cmd")
        .arg("/c")
        .arg("start")
        .arg(path.as_ref())
        .status_without_output()
        .into_result()
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    std::process::Command::new("cmd")
        .arg("/c")
        .arg(app.into())
        .arg(path.as_ref())
        .status_without_output()
        .into_result()
}
