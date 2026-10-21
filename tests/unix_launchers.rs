#![cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]

use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

const CASE: &str = "OPEN_RS_TEST_LAUNCHER_CASE";
const LOG: &str = "OPEN_RS_TEST_LAUNCHER_LOG";

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "open-launchers-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn launcher(&self, name: &str, body: &str) {
        let path = self.directory.join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn log(&self) -> PathBuf {
        self.directory.join("attempts")
    }

    fn run(&self, case: &str) {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "launcher_fixture", "--nocapture"])
            .env(CASE, case)
            .env(LOG, self.log())
            .env("PATH", &self.directory)
            .env_remove("WSL_DISTRO_NAME")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "fixture {case} failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn attempts(&self) -> String {
        fs::read_to_string(self.log()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.directory).unwrap();
    }
}

// Each scenario runs in a subprocess so changing PATH cannot interfere with
// parallel tests. Only fixture launchers are reachable, never host handlers.
#[test]
fn launcher_fixture() {
    let case = match std::env::var(CASE) {
        Ok(case) => case,
        Err(_) => return,
    };
    let timeout = Duration::from_secs(1);
    match case.as_str() {
        "fallback" | "success" | "kde" => open::that("document.pdf").unwrap(),
        "detached-fallback" => open::that_detached_with_timeout("document.pdf", timeout).unwrap(),
        "outlive" => open::that_detached_with_timeout("document.pdf", Duration::ZERO).unwrap(),
        "failure" | "detached-failure" => {
            let result = if case == "failure" {
                open::that("document.pdf")
            } else {
                open::that_detached_with_timeout("document.pdf", timeout)
            };
            let err = result.unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::Other);
            assert!(err.to_string().contains("xdg-open"), "{}", err);
        }
        "missing" => {
            assert_eq!(
                open::that_detached_with_timeout("document.pdf", timeout)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::NotFound
            );
        }
        "timeout" | "zero-timeout" => {
            let duration = if case == "zero-timeout" {
                Duration::ZERO
            } else {
                Duration::from_millis(50)
            };
            let start = Instant::now();
            open::that_detached_with_timeout("document.pdf", duration).unwrap();
            assert!(start.elapsed() < Duration::from_secs(1));
            if !duration.is_zero() {
                assert!(start.elapsed() >= duration);
            }
            // The launcher sleeps for 1.5 seconds, then fails. The API must
            // return early, reap it later, and never try a second launcher.
            std::thread::sleep(Duration::from_secs(2));
            #[cfg(target_os = "linux")]
            {
                let children =
                    fs::read_to_string(format!("/proc/self/task/{}/children", std::process::id()))
                        .unwrap();
                assert!(
                    children.trim().is_empty(),
                    "unreaped children: {}",
                    children
                );
            }
        }
        "arguments" => {
            let bytes = b"--load-modules=a b;$(touch nope)\xff";
            open::that_detached_with_timeout(std::ffi::OsStr::from_bytes(bytes), timeout).unwrap();
        }
        _ => panic!("unknown fixture {}", case),
    }
}

const FAIL: &str = "printf 'xdg-open\\n' >> \"$OPEN_RS_TEST_LAUNCHER_LOG\"\nexit 4";
const SUCCEED: &str = "printf 'gio\\n' >> \"$OPEN_RS_TEST_LAUNCHER_LOG\"\nexit 0";

#[test]
fn retries_failed_launchers_and_stops_after_success() {
    for case in ["fallback", "detached-fallback"] {
        let fixture = Fixture::new();
        fixture.launcher("xdg-open", FAIL);
        fixture.launcher("gio", SUCCEED);
        fixture.launcher("gnome-open", "exit 99");
        fixture.run(case);
        assert_eq!(fixture.attempts(), "xdg-open\ngio\n");
    }
    let fixture = Fixture::new();
    fixture.launcher(
        "xdg-open",
        "printf 'success\\n' >> \"$OPEN_RS_TEST_LAUNCHER_LOG\"",
    );
    fixture.launcher("gio", SUCCEED);
    fixture.run("success");
    assert_eq!(fixture.attempts(), "success\n");
}

#[test]
fn detached_launcher_outlives_its_caller() {
    let fixture = Fixture::new();
    fixture.launcher(
        "xdg-open",
        "/bin/sleep 0.25\nprintf 'finished\\n' >> \"$OPEN_RS_TEST_LAUNCHER_LOG\"",
    );
    fixture.run("outlive");
    let start = Instant::now();
    while !fixture.log().exists() {
        assert!(start.elapsed() < Duration::from_secs(3));
        std::thread::sleep(Duration::from_millis(10));
    }
    // Read after the write completes, rather than racing the shell opening it.
    while fs::read(fixture.log()).unwrap().is_empty() {
        assert!(start.elapsed() < Duration::from_secs(3));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fixture.attempts(), "finished\n");
}

#[test]
fn launcher_output_cannot_fill_a_pipe_and_block_status_checks() {
    let fixture = Fixture::new();
    fixture.launcher(
        "xdg-open",
        "i=0\nwhile [ \"$i\" -lt 10000 ]; do\n  printf 'large launcher diagnostic message\\n' >&2\n  i=$((i + 1))\ndone\nexit 4",
    );
    fixture.launcher("gio", SUCCEED);
    fixture.run("detached-fallback");
    assert_eq!(fixture.attempts(), "gio\n");
}

#[test]
fn preserves_actual_failure_when_later_launchers_are_missing() {
    for case in ["failure", "detached-failure"] {
        let fixture = Fixture::new();
        fixture.launcher("xdg-open", FAIL);
        fixture.run(case);
        assert_eq!(fixture.attempts(), "xdg-open\n");
    }
    Fixture::new().run("missing");
}

#[test]
fn reaches_plasma_six_after_failed_and_missing_launchers() {
    let fixture = Fixture::new();
    fixture.launcher("xdg-open", FAIL);
    fixture.launcher(
        "kde-open6",
        "printf 'kde-open6\\n' >> \"$OPEN_RS_TEST_LAUNCHER_LOG\"",
    );
    fixture.run("kde");
    assert_eq!(fixture.attempts(), "xdg-open\nkde-open6\n");
}

#[test]
fn timeout_returns_early_without_retrying_and_reaps_the_launcher() {
    for case in ["timeout", "zero-timeout"] {
        let fixture = Fixture::new();
        fixture.launcher(
            "xdg-open",
            "/bin/sleep 1.5\nprintf 'finished\\n' >> \"$OPEN_RS_TEST_LAUNCHER_LOG\"\nexit 4",
        );
        fixture.launcher("gio", SUCCEED);
        fixture.run(case);
        assert_eq!(fixture.attempts(), "finished\n");
    }
}

#[test]
fn retries_without_reinterpreting_non_utf8_option_shaped_paths() {
    let fixture = Fixture::new();
    fixture.launcher("xdg-open", "exit 4");
    fixture.launcher(
        "kde-open6",
        "printf '%s\\n' \"$@\" > \"$OPEN_RS_TEST_LAUNCHER_LOG\"",
    );
    fixture.run("arguments");
    assert_eq!(
        fs::read(fixture.log()).unwrap(),
        b"--\n--load-modules=a b;$(touch nope)\xff\n"
    );
}

#[test]
fn cli_retries_failed_launchers() {
    let fixture = Fixture::new();
    fixture.launcher("xdg-open", FAIL);
    fixture.launcher("gio", SUCCEED);
    let output = Command::new(env!("CARGO_BIN_EXE_open"))
        .arg("document.pdf")
        .env("PATH", &fixture.directory)
        .env(LOG, fixture.log())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("gio"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("xdg-open"));
    assert_eq!(fixture.attempts(), "xdg-open\ngio\n");
}
