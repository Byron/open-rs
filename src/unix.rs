use std::{ffi::OsStr, process::Command};

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let path = path.as_ref();
    let is_wsl = is_wsl::is_wsl();
    let mut commands = vec![];

    if is_wsl {
        commands.push(crate::wsl::command(path));
    }

    let fallback_commands = [
        ("xdg-open", vec![path]),
        ("gio", vec![OsStr::new("open"), path]),
        ("gnome-open", vec![path]),
        ("kde-open", vec![path]),
    ];

    commands.extend(fallback_commands.iter().map(|(command, args)| {
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd
    }));

    commands
}

pub fn with_command<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Command {
    let mut cmd = Command::new(app.into());
    cmd.arg(path.as_ref());
    cmd
}
