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
use std::io::{BufRead, BufReader};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct StandaloneDirectory {
    path: PathBuf,
    owner: Child,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl StandaloneDirectory {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for StandaloneDirectory {
    fn drop(&mut self) {
        drop(self.owner.stdin.take());
        let _ = self.owner.wait();
    }
}

/* === Public Functions === */

pub fn tool() -> std::path::PathBuf {
    artifact("cargo-dokime")
}

pub fn host() -> std::path::PathBuf {
    artifact("dokime")
}

pub fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
        .ancestors()
        .find(|ancestor| ancestor.join("Cargo.lock").is_file())
        .expect("the crate lives in a Cargo workspace")
        .to_owned()
}

pub fn standalone_directory() -> StandaloneDirectory {
    let mut command = Command::new("uv");
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let mut owner = command
        .args(["run", "--locked", "--project", "tools/python", "python", "-m", "dokime_tools.fixture"])
        .current_dir(workspace_root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the repository tooling owns the temporary fixture");
    let output = owner.stdout.take().unwrap();
    let mut ready = String::new();
    BufReader::new(output).read_line(&mut ready).unwrap();
    let path = PathBuf::from(ready.trim_end());
    assert!(path.is_absolute() && path.is_dir(), "the fixture owner reported no temporary directory");
    StandaloneDirectory { path, owner }
}

/* === Private Functions === */

fn artifact(name: &str) -> std::path::PathBuf {
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    let profile = std::env::var("DOKIME_BUILD_PROFILE").unwrap_or_else(|_| "dev".to_owned());
    assert!(matches!(profile.as_str(), "dev" | "release"), "invalid graph profile");
    let target = format!("{platform}-{}", std::env::consts::ARCH);
    let path = workspace_root()
        .join("target/build")
        .join(profile)
        .join(target)
        .join("bin")
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    assert!(path.is_file(), "{} is missing: run mise run build", path.display());
    path
}

/* === Tests === */
