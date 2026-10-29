/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

fn command(cwd: &std::path::Path) -> std::process::Command {
    let server = super::support::host();
    // The dokime CLI built beside this tool launches the prepared server.
    let cli = super::support::tool().with_file_name(format!("dokime{}", std::env::consts::EXE_SUFFIX));
    let mut command = dokime::domain::process::command(super::support::tool());
    command
        .current_dir(cwd)
        .env_remove("DOKIME_DEV_ROOT")
        .env("DOKIME_HOME", server.parent().unwrap())
        .args(["--color", "never", "sidecar", "dev", "--headless", "--dokime"])
        .arg(cli)
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
    let line = output.lines().find(|line| line.contains("Development data")).expect(output);
    let path = line.split_once("Development data").unwrap().1.trim().trim_start_matches([':', '·']).trim();
    std::path::PathBuf::from(path)
}

#[test]
fn development_state_scoped() {
    let root = tempfile::tempdir().unwrap();
    let development = root.path().join("sessions");
    let run = || {
        let mut c = command(root.path());
        c.env("DOKIME_DEV_ROOT", &development);
        refusal(c)
    };
    let first = state(&run());
    let second = state(&run());
    assert!(first.starts_with(development.canonicalize().unwrap()), "{}", first.display());
    assert_ne!(first, second);
    assert!(!first.exists());
    assert!(!second.exists());
    let persistent = root.path().join("persistent");
    let invalid = root.path().join("file");
    std::fs::write(&invalid, "file").unwrap();
    let mut c = command(root.path());
    c.env("DOKIME_DEV_ROOT", &invalid).arg("--state-dir").arg(&persistent);
    assert_eq!(state(&refusal(c)), persistent.canonicalize().unwrap());
    assert!(persistent.is_dir());
    let mut c = command(root.path());
    c.env("DOKIME_DEV_ROOT", &invalid);
    assert!(!refusal(c).contains("Development data"));
}

#[test]
fn environment_precedence_ordered() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    let nested = nested.canonicalize().unwrap();
    std::fs::write(root.path().join(".env"), "DOKIME_DEV_ROOT=ancestor\n").unwrap();
    let chosen = root.path().join("chosen.env");
    std::fs::write(&chosen, "DOKIME_DEV_ROOT=chosen\nDOKIME_DEV_ROOT=ignored\n").unwrap();
    for (explicit, shell, expected) in [(false, None, "ancestor"), (true, None, "chosen"), (true, Some("shell"), "shell")] {
        let mut c = command(&nested);
        if explicit {
            c.arg("--env-file").arg(&chosen);
        }
        if let Some(value) = shell {
            c.env("DOKIME_DEV_ROOT", value);
        }
        let output = refusal(c);
        assert!(state(&output).starts_with(nested.join(expected)), "{output}");
    }
    std::fs::write(nested.join(".env"), "DOKIME_DEV_ROOT=nearest\n").unwrap();
    let output = refusal(command(&nested));
    assert!(state(&output).starts_with(nested.join("nearest")), "{output}");
    let mut c = command(&nested);
    c.env("DOKIME_DEV_ROOT", "");
    assert!(refusal(c).contains("DOKIME_DEV_ROOT must not be empty"));
}

#[test]
fn environment_paths_validated() {
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
