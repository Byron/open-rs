use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, ptr};

use std::os::raw::c_int;
use windows_sys::Win32::UI::Shell::ShellExecuteW;

use crate::IntoResult;

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

pub fn that<T: AsRef<OsStr>>(path: T) -> io::Result<()> {
    const SW_SHOW: c_int = 5;

    let path = convert_path(path.as_ref())?;
    let operation: Vec<u16> = OsStr::new("open\0").encode_wide().collect();
    let result = unsafe {
        ShellExecuteW(
            0,
            operation.as_ptr(),
            path.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOW,
        )
    };
    (result as c_int).into_result()
}

pub fn with<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    const SW_SHOW: c_int = 5;

    let path = convert_path(path.as_ref())?;
    let operation: Vec<u16> = OsStr::new("open\0").encode_wide().collect();
    let app_name: Vec<u16> = OsStr::new(&format!("{}\0", app.into()))
        .encode_wide()
        .collect();
    let result = unsafe {
        ShellExecuteW(
            0,
            operation.as_ptr(),
            app_name.as_ptr(),
            path.as_ptr(),
            ptr::null(),
            SW_SHOW,
        )
    };
    (result as c_int).into_result()
}
