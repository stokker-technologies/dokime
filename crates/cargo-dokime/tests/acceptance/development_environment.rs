/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

fn command(cwd: &std::path::Path) -> std::process::Command {
    let backend = std::path::PathBuf::from(
        std::env::var_os("DOKIME_TEST_HOST").expect("mise selects the exact prepared server"),
    );
    assert!(
        backend.is_file(),
        "Prepared Server Must Provide {}",
        backend.display()
    );
    let mut command = std::process::Command::new(super::support::tool());
    command
        .current_dir(cwd)
        .env_remove("DOKIME_DEV_ROOT")
        .args(["--color", "never", "dev", "--headless", "--dokime"])
        .arg(backend)
        .arg("--manifest-path")
        .arg(cwd.join("missing-Cargo.toml"));
    command
}

fn refusal(mut command: std::process::Command) -> String {
    let output = command.output().unwrap();
    assert!(!output.status.success());
    String::from_utf8(output.stderr).unwrap()
}

fn state(output: &str) -> std::path::PathBuf {
    let line = output
        .lines()
        .find(|line| line.contains("Development data"))
        .expect(output);
    let path = line
        .split_once("Development data")
        .unwrap()
        .1
        .trim()
        .trim_start_matches([':', '·'])
        .trim();
    std::path::PathBuf::from(path)
}

fn temporary_state_is_unique_and_removed_persistent_state_stays() {
    let root = tempfile::tempdir().unwrap();
    let development = root.path().join("sessions");
    let run = || {
        let mut c = command(root.path());
        c.env("DOKIME_DEV_ROOT", &development);
        refusal(c)
    };
    let first = state(&run());
    let second = state(&run());
    assert!(
        first.starts_with(development.canonicalize().unwrap()),
        "{}",
        first.display()
    );
    assert_ne!(first, second);
    assert!(!first.exists());
    assert!(!second.exists());
    let persistent = root.path().join("persistent");
    let invalid = root.path().join("file");
    std::fs::write(&invalid, "file").unwrap();
    let mut c = command(root.path());
    c.env("DOKIME_DEV_ROOT", &invalid)
        .arg("--state-dir")
        .arg(&persistent);
    assert_eq!(state(&refusal(c)), persistent.canonicalize().unwrap());
    assert!(persistent.is_dir());
    let mut c = command(root.path());
    c.env("DOKIME_DEV_ROOT", &invalid);
    assert!(!refusal(c).contains("Development data"));
}

fn root_and_dotenv_precedence_preserve_shell_then_nearest_file() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(root.path().join(".env"), "DOKIME_DEV_ROOT=ancestor\n").unwrap();
    let chosen = root.path().join("chosen.env");
    std::fs::write(&chosen, "DOKIME_DEV_ROOT=chosen\nDOKIME_DEV_ROOT=ignored\n").unwrap();
    for (explicit, shell, expected) in [
        (false, None, "ancestor"),
        (true, None, "chosen"),
        (true, Some("shell"), "shell"),
    ] {
        let mut c = command(&nested);
        if explicit {
            c.arg("--env-file").arg(&chosen);
        }
        if let Some(value) = shell {
            c.env("DOKIME_DEV_ROOT", value);
        }
        assert!(state(&refusal(c)).starts_with(nested.join(expected)));
    }
    std::fs::write(nested.join(".env"), "DOKIME_DEV_ROOT=nearest\n").unwrap();
    assert!(state(&refusal(command(&nested))).starts_with(nested.join("nearest")));
    let mut c = command(&nested);
    c.env("DOKIME_DEV_ROOT", "");
    assert!(refusal(c).contains("DOKIME_DEV_ROOT must not be empty"));
}

fn cwd_and_environment_paths_are_validated_before_building() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("file");
    std::fs::write(&file, "").unwrap();
    let mut c = command(root.path());
    c.arg("--cwd").arg(&file);
    assert!(refusal(c).contains("does not exist or is not a directory"));
    let mut c = command(root.path());
    c.arg("--env-file").arg(root.path().join("missing.env"));
    assert!(refusal(c).contains("does not exist"));
    let mut c = command(root.path());
    c.arg("--env-file").arg(&file);
    let output = refusal(c);
    assert!(output.contains(file.to_str().unwrap()));
    assert!(output.contains(root.path().to_str().unwrap()));
    assert!(output.contains("Development data"));
}

pub(super) fn environment_contract() {
    temporary_state_is_unique_and_removed_persistent_state_stays();
    root_and_dotenv_precedence_preserve_shell_then_nearest_file();
    cwd_and_environment_paths_are_validated_before_building();
}
