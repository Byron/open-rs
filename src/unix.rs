use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

use crate::{ExitStatusPolicy, Launcher};

/// Ordered desktop launchers, keeping argument handling and exit-status policy together.
/// `that()` and `that_detached_with_timeout()` use this policy; `commands()` exposes
/// the same ordering for callers managing execution themselves. Successful exits
/// stop fallback; missing executables allow fallback on every platform. Only
/// `RetryOnFailure` launchers allow fallback after a nonzero exit. Other platforms
/// retain `StopOnFailure`, notably Windows where Explorer's nonzero status is not
/// a dependable failure indication and retrying could open a target twice.
///
/// Exit status describes the launcher's knowledge, not the target application's
/// outcome. In particular, xdg-open can wait for an application that already opened
/// the target and then failed: retries follow its contract but may duplicate opens.
/// Ordinary detached calls confirm spawning, not opening; the timeout variant
/// observes the actual launcher child, not a successful double-fork intermediate.
/// A launcher still running at the deadline is considered handed off, not killed,
/// and later failure cannot trigger fallback.
///
/// Compatibility constraints from the implementation's history:
/// - Real-operation failures were retried before availability probes were added
///   in fdd2ef0 (2023). 55dc10d allowed nonzero *argumentless probe* exits (usage
///   errors), not failed opens. The commands() loop in 245c95e introduced stopping
///   after the first executable ran, even on failure.
/// - Keep convenience API stdio null: 30a144a (#85) fixed waiting before draining
///   stderr; 0bdc6d6 removed capture because applications can inherit pipes and
///   prevent EOF indefinitely. Callers needing diagnostics can configure commands().
/// - Preserve fd29861's (#124) option-injection protection for every candidate,
///   including non-UTF-8 paths: use `--` only when supported, otherwise prefix
///   dash-leading paths with `./`.
struct LauncherSpec {
    program: &'static str,
    args: &'static [&'static str],
    /// Whether `--` can separate launcher options from an unchanged target.
    /// If false, dash-leading paths are prefixed with `./` instead.
    supports_separator: bool,
    exit_status: ExitStatusPolicy,
}

// Keep xdg-open first to respect desktop/portal policy; gio is a general fallback,
// followed by the legacy gnome-open priority and then KDE's configured handlers.
// Versioned KDE names extend the existing fallback position, newest first.
// Do not substitute kioclient{5,6} exec: KDE enables executable/desktop-file
// execution there (allowExec = true), unlike kde-open. Hard-coded file managers
// can bypass default handlers, support fewer target types, and remain running.
// Do not automatically rewrite AppImage PATH/LD_LIBRARY_PATH: callers may select
// launchers/libraries intentionally and can configure commands() for a host-only
// environment themselves.
const DESKTOP_LAUNCHERS: &[LauncherSpec] = &[
    // Documented failure codes 1–4; no safe `--` support. May wait for the app.
    // https://man.archlinux.org/man/xdg-open.1.en
    LauncherSpec {
        program: "xdg-open",
        args: &[],
        supports_separator: false,
        exit_status: ExitStatusPolicy::RetryOnFailure,
    },
    // GLib reports 1 for syntax errors and 2 for launch errors; its handler may
    // differ from KDE's. https://github.com/GNOME/glib/blob/main/gio/gio-tool-open.c
    LauncherSpec {
        program: "gio",
        args: &["open"],
        supports_separator: false,
        exit_status: ExitStatusPolicy::RetryOnFailure,
    },
    // Legacy launcher: `--` is not safe here; preserve option-safe path handling.
    LauncherSpec {
        program: "gnome-open",
        args: &[],
        supports_separator: false,
        exit_status: ExitStatusPolicy::RetryOnFailure,
    },
    // KDE reports 0 for a successful open job and 1 for failure. All versions
    // support `--` and preserve the target unchanged. Implementation and aliases:
    // https://github.com/KDE/kde-cli-tools/blob/master/kioclient/kioclient.cpp
    // https://github.com/KDE/kde-cli-tools/blob/master/kioclient/CMakeLists.txt
    LauncherSpec {
        program: "kde-open6",
        args: &[],
        supports_separator: true,
        exit_status: ExitStatusPolicy::RetryOnFailure,
    },
    LauncherSpec {
        program: "kde-open5",
        args: &[],
        supports_separator: true,
        exit_status: ExitStatusPolicy::RetryOnFailure,
    },
    LauncherSpec {
        program: "kde-open",
        args: &[],
        supports_separator: true,
        exit_status: ExitStatusPolicy::RetryOnFailure,
    },
];

/// Prevent launchers from interpreting a path that starts with `-` as an option.
///
/// Prefixing such paths with `./` preserves their meaning as paths relative to the
/// current directory. This is used for launchers that do not support `--` as an
/// end-of-options marker.
fn option_safe_path(path: &OsStr) -> OsString {
    use std::os::unix::ffi::OsStrExt;

    if path.as_bytes().first() == Some(&b'-') {
        let mut safe = OsString::from("./");
        safe.push(path);
        safe
    } else {
        path.to_os_string()
    }
}

pub(super) fn launchers<T: AsRef<OsStr>>(path: T) -> Vec<Launcher> {
    let path = path.as_ref();
    let is_wsl = is_wsl::is_wsl();
    let mut commands = vec![];

    if is_wsl {
        // Prefer Windows integration on WSL. PowerShell stops on script errors;
        // wslview was replaced because it could report success for nonexistent
        // paths (668734e / v2.0.0), not because nonzero exits should be ignored.
        commands.push(Launcher {
            command: crate::wsl::command(path),
            exit_status: ExitStatusPolicy::RetryOnFailure,
        });
    }

    let safe_path = option_safe_path(path);

    for spec in DESKTOP_LAUNCHERS {
        let mut command = Command::new(spec.program);
        command.args(spec.args);
        if spec.supports_separator {
            command.arg("--").arg(path);
        } else {
            command.arg(&safe_path);
        }
        commands.push(Launcher {
            command,
            exit_status: spec.exit_status,
        });
    }

    commands
}

pub fn with_command<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> Command {
    let mut cmd = Command::new(app.into());
    cmd.arg(path.as_ref());
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;

    fn args(command: &Command) -> Vec<&OsStr> {
        command.get_args().collect()
    }

    #[test]
    fn protects_dash_leading_paths_from_launcher_option_parsing() {
        let commands = commands("--load-modules=/tmp/evil.so");

        let expected_len = DESKTOP_LAUNCHERS.len();
        let commands = &commands[commands.len() - expected_len..];
        assert_eq!(commands.len(), expected_len);
        assert_eq!(commands[0].get_program(), "xdg-open");
        assert_eq!(
            args(&commands[0]),
            [OsStr::new("./--load-modules=/tmp/evil.so")],
            "xdg-open has no end-of-options marker, so the path must not start with a dash"
        );
        assert_eq!(commands[1].get_program(), "gio");
        assert_eq!(
            args(&commands[1]),
            [
                OsStr::new("open"),
                OsStr::new("./--load-modules=/tmp/evil.so")
            ],
            "gio has no end-of-options marker, so the path must not start with a dash"
        );
        assert_eq!(commands[2].get_program(), "gnome-open");
        assert_eq!(
            args(&commands[2]),
            [OsStr::new("./--load-modules=/tmp/evil.so")],
            "gnome-open would otherwise interpret the path as a module-loading option"
        );
        for (kde, program) in commands[3..]
            .iter()
            .zip(["kde-open6", "kde-open5", "kde-open"])
        {
            assert_eq!(kde.get_program(), program);
            assert_eq!(
                args(kde),
                [OsStr::new("--"), OsStr::new("--load-modules=/tmp/evil.so")],
                "KDE launchers support separating their options from an unchanged path"
            );
        }
    }

    #[test]
    fn leaves_urls_and_regular_paths_unchanged() {
        for path in ["https://example.com", "document.pdf", "/tmp/-document.pdf"] {
            let commands = commands(path);
            let expected_len = DESKTOP_LAUNCHERS.len();
            let commands = &commands[commands.len() - expected_len..];
            assert_eq!(args(&commands[0]), [OsStr::new(path)]);
            assert_eq!(args(&commands[1]), [OsStr::new("open"), OsStr::new(path)]);
            assert_eq!(args(&commands[2]), [OsStr::new(path)]);
            for command in &commands[3..] {
                assert_eq!(args(command), [OsStr::new("--"), OsStr::new(path)]);
            }
        }
    }
}
