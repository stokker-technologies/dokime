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
pub fn tool() -> std::path::PathBuf {
    artifact("cargo-dokime")
}

pub fn host() -> std::path::PathBuf {
    artifact("dokime-server")
}

pub fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|ancestor| ancestor.join("Cargo.lock").is_file())
        .expect("the crate lives in a Cargo workspace")
        .to_owned()
}

pub fn build_root() -> std::path::PathBuf {
    workspace_root().join("target/build").join(if cfg!(debug_assertions) { "dev" } else { "release" }).join(format!(
        "{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ))
}

pub fn standalone_directory() -> tempfile::TempDir {
    let preferred = std::env::temp_dir().canonicalize().unwrap();
    let mut parent = preferred.as_path();
    for ancestor in preferred.ancestors() {
        if ancestor.join("Cargo.toml").is_file() {
            parent = ancestor.parent().expect("filesystem root cannot be a Cargo workspace");
        }
    }
    let directory = tempfile::Builder::new().prefix("dokime-plugin-").tempdir_in(parent).unwrap();
    // A generated standalone plugin must never join an enclosing workspace.
    assert!(
        directory.path().ancestors().all(|ancestor| !ancestor.join("Cargo.toml").exists()),
        "{} lies inside a Cargo workspace",
        directory.path().display()
    );
    directory
}

/* === Private Functions === */
fn artifact(name: &str) -> std::path::PathBuf {
    let path = build_root().join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    assert!(path.is_file(), "{} is missing: run mise run build", path.display());
    path
}

/* === Tests === */
