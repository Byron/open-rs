use std::{
    ffi::{OsStr, OsString},
    io,
    os::windows::ffi::OsStrExt,
    ptr,
};

use std::os::raw::c_int;

use crate::IntoResult;

fn convert_path(path: &OsStr) -> io::Result<Vec<u16>> {
    let mut quoted_path = OsString::with_capacity(path.len());

    // Surround path with double quotes "" to handle spaces in path.
    quoted_path.push("\"");
    quoted_path.push(&path);
    quoted_path.push("\"");

    let mut wide_chars: Vec<_> = quoted_path.encode_wide().collect();
    if wide_chars.iter().any(|&u| u == 0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "path contains NUL byte(s)",
        ));
    }
    wide_chars.push(0);
    Ok(wide_chars)
}

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    let path = convert_path(path.as_ref())?;
    Command::new("cmd")
        .arg("/c")
        .arg("start")
        .arg(path.as_ref())
        .status_without_output()
        .into_result()
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    let path = convert_path(path.as_ref())?;
    Command::new("cmd")
        .arg("/c")
        .arg(app.into())
        .arg(path.as_ref())
        .status_without_output()
        .into_result()
}
