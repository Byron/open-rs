use std::{
    ffi::{OsStr, OsString},
    process::Command,
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
        .raw_arg(app.into())
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
