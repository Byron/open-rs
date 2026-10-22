//! Use this library to open a given path or URL in some application, obeying the current user's desktop configuration.
//!
//! It is expected that `http:` and `https:` URLs will open in a web browser, although the desktop configuration may override this.
//! The choice of application for opening other path types is highly system-dependent.
//!
//! To always open a web browser, see the [webbrowser](https://docs.rs/webbrowser) crate, which offers functionality for that specific case.
//!
//! # Usage
//!
//! Open the given URL in the default web browser, without blocking.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! open::that("http://rust-lang.org")?;
//! # Ok(())
//! # }
//! ```
//!
//! Alternatively, specify the program to be used to open the path or URL.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! open::with("http://rust-lang.org", "firefox")?;
//! # Ok(())
//! # }
//! ```
//!
//! Or obtain the commands to launch a file or path without running them.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cmds = open::commands("http://rust-lang.org")[0].status()?;
//! # Ok(())
//! # }
//! ```
//!
//! It's also possible to obtain a command that can be used to open a path in an application.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let status = open::with_command("http://rust-lang.org", "firefox").status()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Notes
//!
//! ## Nonblocking operation
//!
//! The functions provided are nonblocking as it will return even though the
//! launched child process is still running. Note that depending on the operating
//! system, spawning launch helpers, which this library does under the hood,
//! might still take 100's of milliseconds.
//!
//! **Beware that on some platforms and circumstances, the launcher may block**.
//! In this case, please use the [`commands()`] or [`with_command()`] accordingly
//! to `spawn()` it without blocking.
//!
//! ## Error handling
//!
//! As an operating system program is used, the open operation can fail.
//! Therefore, you are advised to check the result and behave
//! accordingly, e.g. by letting the user know that the open operation failed.
//!
//! ```no_run
//! let path = "http://rust-lang.org";
//!
//! match open::that(path) {
//!     Ok(()) => println!("Opened '{}' successfully.", path),
//!     Err(err) => eprintln!("An error occurred when opening '{}': {}", path, err),
//! }
//! ```
//!
//! ## UNIX Desktop
//!
//! The choice of application for opening a given URL or path on a UNIX desktop is highly dependent on the user's GUI framework (if any) and its configuration.
//! `open::that()` tries a sequence of opener commands to open the specified path.
//! The configuration of these openers is dependent on the window system.
//!
// On a console, without a window manager, results will likely be poor. The openers expect to be able to open in a new or existing window, something that consoles lack.
//!
//! On Windows WSL, PowerShell is tried first to delegate to the Windows default handler, then `xdg-open`. In other UNIX environments, `xdg-open` is tried first. If this fails, a sequence of other openers is tried.
//!
//! Currently the `BROWSER` environment variable is ignored even for `http:` and `https:` URLs unless the opener being used happens to respect it.
//!
//! It cannot be overemphasized how fragile this all is in UNIX environments. It is common for the various MIME tables to incorrectly specify the application "owning" a given filetype.
//! It is common for openers to behave strangely. Use with caution, as this crate merely inherits a particular platforms shortcomings.
//!
//! ## Security boundaries
//!
//! Paths passed to system launchers are separated from launcher options where the
//! launcher supports it. Applications selected through [`with()`] have arbitrary
//! command-line grammars, however, so the crate cannot guarantee that every custom
//! application treats a dash-leading path as data. Launching a document or custom
//! URL scheme also trusts the configured local handler and the launched content.
//! The `insecure` Cargo feature restores the legacy `cmd /c start` launcher on
//! Windows for compatibility, but must not be enabled when paths or URLs may be
//! attacker-controlled.

#[cfg(target_os = "windows")]
use windows as os;

#[cfg(target_os = "macos")]
use macos as os;

#[cfg(target_os = "ios")]
use ios as os;

#[cfg(target_os = "visionos")]
use ios as os;

#[cfg(target_os = "haiku")]
use haiku as os;

#[cfg(target_os = "redox")]
use redox as os;

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "aix",
    target_os = "hurd"
))]
use unix as os;

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "ios",
    target_os = "visionos",
    target_os = "macos",
    target_os = "windows",
    target_os = "haiku",
    target_os = "redox",
    target_os = "aix",
    target_os = "hurd"
)))]
compile_error!("open is not supported on this platform");

use std::{
    ffi::OsStr,
    io,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

/// How long [`that_detached()`] observes a launcher for an immediate failure
/// before treating it as handed off. Misconfigured launchers fail within
/// milliseconds; callers with a different budget can use
/// [`that_detached_with_timeout()`].
#[cfg(all(
    not(target_os = "macos"),
    any(not(feature = "shellexecute-on-windows"), not(windows))
))]
const DETACHED_FAILURE_CHECK_TIMEOUT: Duration = Duration::from_millis(250);

/// Open path with the default application without blocking.
///
/// # Examples
///
/// ```no_run
/// let path = "http://rust-lang.org";
///
/// match open::that(path) {
///     Ok(()) => println!("Opened '{}' successfully.", path),
///     Err(err) => panic!("An error occurred when opening '{}': {}", path, err),
/// }
/// ```
///
/// # Errors
///
/// A [`std::io::Error`] is returned on failure. Because different operating systems
/// handle errors differently it is recommend to not match on a certain error.
/// On UNIX desktops, unsuccessful launcher exits cause the next candidate to be
/// tried. If none succeeds, the first unsuccessful exit is reported in preference
/// to errors from missing fallback executables.
///
/// # Beware
///
/// Sometimes, depending on the platform and system configuration, launchers *can* block.
/// If you want to be sure they don't, use [`that_in_background()`] or [`that_detached`] instead.
pub fn that(path: impl AsRef<OsStr>) -> io::Result<()> {
    try_launchers(launchers(path), |cmd| {
        cmd.status_without_output().map(LauncherStatus::Exited)
    })
}

/// Open path with the given application.
///
/// This function may block if the application or launcher doesn't detach itself.
/// In that case, consider using [`with_in_background()`] or [`with_command()].
///
/// # Examples
///
/// ```no_run
/// let path = "http://rust-lang.org";
/// let app = "firefox";
///
/// match open::with(path, app) {
///     Ok(()) => println!("Opened '{}' successfully.", path),
///     Err(err) => panic!("An error occurred when opening '{}': {}", path, err),
/// }
/// ```
///
/// # Errors
///
/// A [`std::io::Error`] is returned on failure. Because different operating systems
/// handle errors differently it is recommend to not match on a certain error.
pub fn with(path: impl AsRef<OsStr>, app: impl Into<String>) -> io::Result<()> {
    let mut cmd = with_command(path, app);
    cmd.status_without_output().into_result(&cmd)
}

/// Get multiple commands that open `path` with the default application.
///
/// Each command represents a launcher to try.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "http://rust-lang.org";
/// assert!(open::commands(path)[0].status()?.success());
/// # Ok(())
/// # }
/// ```
pub fn commands(path: impl AsRef<OsStr>) -> Vec<Command> {
    launchers(path)
        .into_iter()
        .map(|launcher| launcher.command)
        .collect()
}

/// Whether a launcher's failure is sufficiently reliable to try another opener.
#[derive(Clone, Copy)]
#[allow(dead_code)] // Each platform constructs only the policies it needs.
enum ExitStatusPolicy {
    RetryOnFailure,
    StopOnFailure,
}

struct Launcher {
    command: Command,
    exit_status: ExitStatusPolicy,
}

enum LauncherStatus {
    Exited(std::process::ExitStatus),
    Running,
    WaitError(io::Error),
}

fn launchers(path: impl AsRef<OsStr>) -> Vec<Launcher> {
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "illumos",
        target_os = "solaris",
        target_os = "aix",
        target_os = "hurd"
    ))]
    return unix::launchers(path);

    #[cfg(any(
        windows,
        target_os = "macos",
        target_os = "ios",
        target_os = "visionos",
        target_os = "haiku",
        target_os = "redox"
    ))]
    os::commands(path)
        .into_iter()
        .map(|command| Launcher {
            command,
            // Preserve existing behavior on other platforms. In particular,
            // Explorer's nonzero exit status is not a reliable launch failure.
            exit_status: ExitStatusPolicy::StopOnFailure,
        })
        .collect()
}

fn try_launchers(
    launchers: Vec<Launcher>,
    mut launch: impl FnMut(&mut Command) -> io::Result<LauncherStatus>,
) -> io::Result<()> {
    let mut first_exit_err = None;
    let mut last_spawn_err = None;
    for mut launcher in launchers {
        let cmd = &mut launcher.command;
        match launch(cmd) {
            Ok(LauncherStatus::Running) => return Ok(()),
            // The launcher already started. An inability to observe its exit
            // is not evidence that opening failed, so do not launch it twice.
            Ok(LauncherStatus::WaitError(err)) => return Err(err),
            Ok(LauncherStatus::Exited(status)) => match Ok(status).into_result(cmd) {
                Ok(()) => return Ok(()),
                Err(err) => match launcher.exit_status {
                    ExitStatusPolicy::StopOnFailure => return Err(err),
                    ExitStatusPolicy::RetryOnFailure => {
                        // Missing later launchers should not mask a launcher's
                        // actual failure with an unrelated ENOENT error.
                        first_exit_err = first_exit_err.or(Some(err));
                    }
                },
            },
            Err(err) => last_spawn_err = Some(err),
        }
    }
    Err(first_exit_err
        .or(last_spawn_err)
        .expect("no launcher worked, at least one error"))
}

/// Get a command that uses `app` to open `path`.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path = "http://rust-lang.org";
/// assert!(open::with_command(path, "app").status()?.success());
/// # Ok(())
/// # }
/// ```
pub fn with_command(path: impl AsRef<OsStr>, app: impl Into<String>) -> Command {
    os::with_command(path, app)
}

/// Open path with the default application in a new thread to assure it's non-blocking.
///
/// See documentation of [`that()`] for more details.
pub fn that_in_background(path: impl AsRef<OsStr>) -> thread::JoinHandle<io::Result<()>> {
    let path = path.as_ref().to_os_string();
    thread::spawn(|| that(path))
}

/// Open path with the given application in a new thread, which is useful if
/// the program ends up to be blocking. Otherwise, prefer [`with()`] for
/// straightforward error handling.
///
/// See documentation of [`with()`] for more details.
pub fn with_in_background<T: AsRef<OsStr>>(
    path: T,
    app: impl Into<String>,
) -> thread::JoinHandle<io::Result<()>> {
    let path = path.as_ref().to_os_string();
    let app = app.into();
    thread::spawn(|| with(path, app))
}

/// Open path with the default application using a detached process. which is useful if
/// the program ends up to be blocking or want to out-live your app
///
/// See documentation of [`that()`] for more details.
///
/// Except on macOS, each launcher is observed for an additional 250 ms. A
/// launcher that fails within that window falls through to the next candidate;
/// one that is still running is treated as handed off. Failures after the
/// window cannot be reported or retried. Use
/// [`that_detached_with_timeout()`] for a different window.
/// With `shellexecute-on-windows`, success instead confirms that the Windows
/// Shell API accepted the request.
///
/// ## Platform-specific
///
/// - **macOS**: `/usr/bin/open` already hands the target off to LaunchServices and
///   returns. Extra double-fork detachment (`spawn_detached`) is unnecessary and
///   leaves unreaped zombie children. This function therefore waits for `open`
///   itself, matching [`with_detached()`].
pub fn that_detached(path: impl AsRef<OsStr>) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        that(path)
    }

    #[cfg(all(
        not(target_os = "macos"),
        any(not(feature = "shellexecute-on-windows"), not(windows))
    ))]
    {
        that_detached_with_timeout(path, DETACHED_FAILURE_CHECK_TIMEOUT)
    }

    #[cfg(all(windows, feature = "shellexecute-on-windows"))]
    {
        windows::that_detached(path)
    }
}

/// Open a path with the default application in a detached process, checking for
/// launcher failures for up to `timeout` **per launcher**.
///
/// On UNIX desktops, a launcher that exits unsuccessfully within this interval
/// causes the next launcher to be tried. Success means that a launcher exited
/// successfully or was still running when its timeout elapsed. It does not
/// confirm that an application opened the target: failures after the timeout
/// cannot be reported, and some launchers report success before the hand-off
/// completes. A zero timeout performs a single immediate status check.
///
/// A launcher still running at the timeout is reaped by a background thread when
/// it exits. It can outlive the calling application. Spawning a process and
/// scheduling the waiter can take additional time beyond the timeout.
///
/// This uses command-based launchers on all platforms, including Windows when
/// the `shellexecute-on-windows` feature is enabled. Windows retains [`that()`]'s
/// policy of stopping on nonzero exit status because launcher exit codes can be
/// unreliable there.
///
/// ```no_run
/// # fn main() -> std::io::Result<()> {
/// open::that_detached_with_timeout("document.pdf", std::time::Duration::from_secs(1))?;
/// # Ok(())
/// # }
/// ```
pub fn that_detached_with_timeout(path: impl AsRef<OsStr>, timeout: Duration) -> io::Result<()> {
    try_launchers(launchers(path), |cmd| {
        cmd.status_detached_with_timeout(timeout)
    })
}

/// Open path with the given application using a detached process, which is useful if
/// the program ends up to be blocking or want to out-live your app. Otherwise, prefer [`with()`] for
/// straightforward error handling.
///
/// See documentation of [`with()`] for more details.
pub fn with_detached<T: AsRef<OsStr>>(path: T, app: impl Into<String>) -> io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        let mut cmd = with_command(path, app);
        cmd.status_without_output().into_result(&cmd)
    }

    #[cfg(all(
        not(target_os = "macos"),
        any(not(feature = "shellexecute-on-windows"), not(windows))
    ))]
    {
        let mut cmd = with_command(path, app);
        cmd.spawn_detached()
    }

    #[cfg(all(windows, feature = "shellexecute-on-windows"))]
    {
        windows::with_detached(path, app)
    }
}

trait IntoResult<T> {
    fn into_result(self, cmd: &Command) -> T;
}

impl IntoResult<io::Result<()>> for io::Result<std::process::ExitStatus> {
    fn into_result(self, cmd: &Command) -> io::Result<()> {
        match self {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Launcher {cmd:?} failed with {:?}", status),
            )),
            Err(err) => Err(err),
        }
    }
}

trait CommandExt {
    fn status_without_output(&mut self) -> io::Result<std::process::ExitStatus>;
    fn status_detached_with_timeout(&mut self, timeout: Duration) -> io::Result<LauncherStatus>;
    #[cfg_attr(
        any(target_os = "macos", feature = "shellexecute-on-windows"),
        allow(dead_code)
    )]
    fn spawn_detached(&mut self) -> io::Result<()>;
}

impl CommandExt for Command {
    fn status_without_output(&mut self) -> io::Result<std::process::ExitStatus> {
        self.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }

    fn status_detached_with_timeout(&mut self, timeout: Duration) -> io::Result<LauncherStatus> {
        self.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // Reserve the waiter before launching anything. If creating a thread
        // fails, there is no live child to abandon or accidentally launch twice.
        let (sender, receiver) = std::sync::mpsc::sync_channel::<std::process::Child>(1);
        thread::Builder::new()
            .name("open-launcher-wait".into())
            // The waiter only receives a child and waits; avoid a default-sized stack.
            .stack_size(64 * 1024)
            .spawn(move || {
                if let Ok(mut child) = receiver.recv() {
                    let _ = child.wait();
                }
            })?;

        #[cfg(unix)]
        unsafe {
            use std::os::unix::process::CommandExt as _;
            // Keep the launcher as our child so its real exit status is
            // available. Unlike spawn_detached, there is no intermediate
            // child whose successful exit could mask a launcher failure.
            self.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            self.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
        }

        let mut child = self.spawn()?;
        let start = Instant::now();
        let mut initial_poll = true;
        let result = loop {
            // Always poll once, even for a zero timeout. After that, a delayed
            // wakeup must not observe a late failure and trigger fallback.
            // We do this primarily to avoid polling again and take more time
            // than allocated.
            if !initial_poll && start.elapsed() >= timeout {
                break LauncherStatus::Running;
            }
            initial_poll = false;
            match child.try_wait() {
                Ok(Some(status)) => return Ok(LauncherStatus::Exited(status)),
                Err(err) => break LauncherStatus::WaitError(err),
                Ok(None) => {
                    let remaining = timeout.saturating_sub(start.elapsed());
                    if remaining.is_zero() {
                        break LauncherStatus::Running;
                    }
                    thread::sleep(remaining.min(Duration::from_millis(10)));
                }
            }
        };

        // Retain ownership of the child after returning. Dropping Child does
        // not reap it on Unix and would leave a zombie when it eventually exits.
        sender.send(child).expect("waiter is waiting for the child");
        Ok(result)
    }

    #[cfg_attr(
        any(target_os = "macos", feature = "shellexecute-on-windows"),
        allow(dead_code)
    )]
    fn spawn_detached(&mut self) -> io::Result<()> {
        // Based on Alacritty's cross-platform daemon implementation:
        // https://github.com/alacritty/alacritty/blob/52031ea91980cee1ee0ec087d214e98b2b1de4bc/alacritty/src/daemon.rs

        self.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(unix)]
        {
            unsafe {
                use std::os::unix::process::CommandExt as _;

                self.pre_exec(|| {
                    // POSIX.1-2024 replaces `fork` with the async-signal-safe `_Fork`, but macOS
                    // does not support it. Since we do not register fork handlers, `fork` remains
                    // suitable for the POSIX.1-2017 systems supported here.
                    match libc::fork() {
                        -1 => return Err(io::Error::last_os_error()),
                        0 => (),
                        _ => libc::_exit(0),
                    }

                    if libc::setsid() == -1 {
                        return Err(io::Error::last_os_error());
                    }

                    Ok(())
                });
            }

            // Reap the intermediate child created by the double-fork.
            self.spawn()?.wait().map(|_| ())
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            self.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);

            self.spawn().map(|_| ())
        }
    }
}

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "visionos")]
mod ios;

#[cfg(target_os = "haiku")]
mod haiku;

#[cfg(target_os = "redox")]
mod redox;

#[cfg(any(
    test,
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "aix",
    target_os = "hurd"
))]
mod wsl;

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "aix",
    target_os = "hurd"
))]
mod unix;

#[cfg(test)]
mod command_ext_tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn failed_status() -> std::process::ExitStatus {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(1 << 8)
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(1)
        }
    }

    #[test]
    fn unreliable_failure_stops_fallback() {
        let launchers = vec![
            Launcher {
                command: Command::new("unreliable-opener"),
                exit_status: ExitStatusPolicy::StopOnFailure,
            },
            Launcher {
                command: Command::new("second-opener"),
                exit_status: ExitStatusPolicy::RetryOnFailure,
            },
        ];
        let mut attempts = 0;
        let err = try_launchers(launchers, |_| {
            attempts += 1;
            Ok(LauncherStatus::Exited(failed_status()))
        })
        .unwrap_err();
        assert_eq!(attempts, 1);
        assert!(err.to_string().contains("unreliable-opener"));
    }

    #[test]
    fn status_observation_error_does_not_launch_twice() {
        let launchers = vec![
            Launcher {
                command: Command::new("first-opener"),
                exit_status: ExitStatusPolicy::RetryOnFailure,
            },
            Launcher {
                command: Command::new("second-opener"),
                exit_status: ExitStatusPolicy::RetryOnFailure,
            },
        ];
        let mut attempts = 0;
        let err = try_launchers(launchers, |_| {
            attempts += 1;
            Ok(LauncherStatus::WaitError(io::Error::new(
                io::ErrorKind::Other,
                "could not observe the launcher's exit",
            )))
        })
        .unwrap_err();
        assert_eq!(attempts, 1);
        assert!(err.to_string().contains("observe"));
    }

    #[test]
    fn spawn_detached_does_not_wait_for_target() {
        const CHILD: &str = "OPEN_RS_DETACHED_TEST_CHILD";
        if std::env::var_os(CHILD).is_some() {
            std::thread::sleep(Duration::from_secs(3));
            return;
        }

        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "command_ext_tests::spawn_detached_does_not_wait_for_target",
            ])
            .env(CHILD, "1");

        let start = Instant::now();
        command.spawn_detached().unwrap();
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(2),
            "spawn_detached must return before its detached target exits, but took {:?}",
            elapsed
        );
    }
}
