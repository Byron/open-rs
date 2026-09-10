#![cfg(windows)]
#![windows_subsystem = "windows"]

use std::{
    fs,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::Command,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

// This integration test is also its own harmless launch target. Keep it in a
// separate test binary so launching it without arguments runs only this test.
#[test]
fn powershell_opens_literal_paths() {
    const MARKER: &str = "OPEN_RS_LITERAL_PATH_TEST_MARKER";
    if let Some(marker) = std::env::var_os(MARKER) {
        let marker = PathBuf::from(marker);
        let pending = marker.with_extension("pending");
        fs::write(
            &pending,
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .as_bytes(),
        )
        .unwrap();
        fs::rename(pending, marker).unwrap();
        return;
    }

    let root = std::env::temp_dir().join(format!(
        "open-rs-literal-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    for (index, directory) in [
        "【Tiny Asa】 [77P7V-712MB]",
        "[ab]",
        "tick` and 'quote'; & $()",
        "-leading",
    ]
    .iter()
    .enumerate()
    {
        let parent = root.join(directory).join("child");
        fs::create_dir_all(&parent).unwrap();
        let target = parent.join("probe[1]`.exe");
        fs::copy(std::env::current_exe().unwrap(), &target).unwrap();
        for (relative, constrained) in [(false, false), (true, false), (false, true), (true, true)]
        {
            let marker = root.join(format!("marker-{index}-{relative}-{constrained}"));
            let path = if relative {
                target.strip_prefix(&root).unwrap()
            } else {
                &target
            };
            let mut command = open::commands(path)
                .into_iter()
                .find(|cmd| cmd.get_program() == "powershell.exe")
                .unwrap();
            if constrained {
                // Execute the production script after restricting the child
                // PowerShell session, without changing machine policy.
                let mut restricted = Command::new(command.get_program());
                restricted
                    .args(command.get_args().take(3))
                    .arg(format!(
                        "$ExecutionContext.SessionState.LanguageMode = 'ConstrainedLanguage';\n{}",
                        command.get_args().last().unwrap().to_str().unwrap()
                    ))
                    .envs(
                        command
                            .get_envs()
                            .filter_map(|(key, value)| value.map(|value| (key, value))),
                    )
                    .creation_flags(0x08000000);
                command = restricted;
            }
            let output = command
                .current_dir(&root)
                .env(MARKER, &marker)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{path:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let deadline = Instant::now() + Duration::from_secs(10);
            while !marker.exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(20));
            }
            let launched =
                PathBuf::from(fs::read_to_string(&marker).expect("the target executable must run"));
            assert_eq!(
                fs::canonicalize(launched).unwrap(),
                fs::canonicalize(&target).unwrap()
            );
        }
        // Windows may briefly keep the image mapped after the marker is written.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match fs::remove_file(&target) {
                Ok(()) => break,
                Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
                Err(error) => panic!("could not remove test executable: {}", error),
            }
        }
    }
    let mut missing = open::commands(root.join("missing.exe"))
        .into_iter()
        .find(|cmd| cmd.get_program() == "powershell.exe")
        .unwrap();
    assert!(
        !missing.output().unwrap().status.success(),
        "a failed shell launch must be reported as a nonzero exit status"
    );
    fs::remove_dir_all(root).unwrap();
}
