use std::{
    ffi::{OsStr, OsString},
    fmt::Debug,
    io,
    process::{Command, Stdio},
};

use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub enum WindowsRunCommand {
    Start(Command),
    Explorer(Command),
}

impl WindowsRunCommand {
    pub fn status_without_output(&mut self) -> io::Result<std::process::ExitStatus> {
        let cmd = match self {
            WindowsRunCommand::Start(cmd) => cmd,
            WindowsRunCommand::Explorer(cmd) => cmd,
        };
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }

    pub fn extract(self) -> Command {
        match self {
            WindowsRunCommand::Start(cmd) => cmd,
            WindowsRunCommand::Explorer(cmd) => cmd,
        }
    }
}

impl Debug for WindowsRunCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start(_) => write!(f, "start"),
            Self::Explorer(_) => write!(f, "explorer"),
        }
    }
}

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<WindowsRunCommand> {
    let mut start_cmd = Command::new("cmd");
    let path = path.as_ref();
    start_cmd
        .arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .raw_arg(wrap_in_quotes(path))
        .creation_flags(CREATE_NO_WINDOW);
    // in certain situations the above doesn't work, so we try to use explorer as mediator.
    // See https://github.com/Byron/open-rs/issues/73 for details.
    let mut explorer_cmd = Command::new("cmd");
    explorer_cmd
        .arg("/c")
        .arg("explorer")
        .raw_arg(wrap_in_quotes(path))
        .creation_flags(CREATE_NO_WINDOW);
    vec![
        WindowsRunCommand::Start(start_cmd),
        WindowsRunCommand::Explorer(explorer_cmd),
    ]
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
