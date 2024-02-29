use std::{
    ffi::{OsStr, OsString},
    io, iter,
    os::windows::ffi::OsStrExt,
    process::Command,
    ptr,
};

use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .raw_arg(wrap_in_quotes(path))
        .creation_flags(CREATE_NO_WINDOW);
    vec![cmd]
}

pub fn with_command<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .raw_arg(wrap_in_quotes(app.into()))
        .raw_arg(wrap_in_quotes(path))
        .creation_flags(CREATE_NO_WINDOW);
    cmd
}

fn wrap_in_quotes<T: AsRef<OsStr>>(path: T) -> OsString {
    let mut result = OsString::from("\"");
    result.push(path);
    result.push("\"");

    result
}

pub fn that_detached<T: AsRef<OsStr>>(path: T) -> std::io::Result<()> {
    detached(path, None::<&str>)
}

pub fn with_detached<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> std::io::Result<()> {
    detached(path, Some(app))
}

#[inline]
fn detached<T: AsRef<OsStr>>(path: T, app: Option<impl Into<String>>) -> std::io::Result<()> {
    let path = wide(path);
    let app = app.map(|a| wide(a.into()));

    let (app, args) = match app {
        Some(app) => (app.as_ptr(), path.as_ptr()),
        None => (path.as_ptr(), ptr::null()),
    };

    unsafe { ShellExecuteW(0, ffi::OPEN, app, args, ptr::null(), ffi::SW_SHOW) }
}

/// Encodes as wide and adds a null character.
fn wide<T: AsRef<OsStr>>(input: T) -> Vec<u16> {
    input.as_ref().encode_wide().chain(iter::once(0)).collect()
}

/// Performs an operation on a specified file.
///
/// <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew>
#[allow(non_snake_case)]
pub unsafe fn ShellExecuteW(
    hwnd: isize,
    lpoperation: *const u16,
    lpfile: *const u16,
    lpparameters: *const u16,
    lpdirectory: *const u16,
    nshowcmd: i32,
) -> std::io::Result<()> {
    let hr = ffi::ShellExecuteW(
        hwnd,
        lpoperation,
        lpfile,
        lpparameters,
        lpdirectory,
        nshowcmd,
    );

    // ShellExecuteW returns > 32 on success
    // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew#return-value
    if hr > 32 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

mod ffi {
    /// Activates the window and displays it in its current size and position.
    ///
    /// <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow>
    pub const SW_SHOW: i32 = 5;

    /// Null-terminated UTF-16 encoding of `open`.  
    pub const OPEN: *const u16 = [111, 112, 101, 110, 0].as_ptr();

    #[link(name = "Shell32")]
    extern "C" {
        pub fn ShellExecuteW(
            hwnd: isize,
            lpoperation: *const u16,
            lpfile: *const u16,
            lpparameters: *const u16,
            lpdirectory: *const u16,
            nshowcmd: i32,
        ) -> i32;
    }
}
