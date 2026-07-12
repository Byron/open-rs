use std::{ffi::OsStr, process::Command};

pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg("--").arg(path.as_ref());
    vec![cmd]
}

pub fn with_command<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Command {
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg("-a").arg(app.into()).arg("--").arg(path.as_ref());
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separates_paths_from_open_options() {
        assert_eq!(
            commands("-aCalculator")[0].get_args().collect::<Vec<_>>(),
            [OsStr::new("--"), OsStr::new("-aCalculator")]
        );
        assert_eq!(
            with_command("-aCalculator", "Preview")
                .get_args()
                .collect::<Vec<_>>(),
            [
                OsStr::new("-a"),
                OsStr::new("Preview"),
                OsStr::new("--"),
                OsStr::new("-aCalculator")
            ]
        );
    }
}
