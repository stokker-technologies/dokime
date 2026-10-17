/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod output;

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
/// The released `cargo-dokime` this crate's stories drive.
pub fn tool() -> std::path::PathBuf {
    artifact("cargo-dokime")
}

/// The released `dokime-server` the tool's stories host plugins in.
pub fn host() -> std::path::PathBuf {
    artifact("dokime-server")
}

/// The directory holding `Cargo.lock`, which every built artifact derives from.
pub fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|ancestor| ancestor.join("Cargo.lock").is_file())
        .expect("the crate lives in a Cargo workspace")
        .to_owned()
}

pub fn target_directory() -> std::path::PathBuf {
    let root = workspace_root();
    std::env::var_os("CARGO_TARGET_DIR").map(|path| root.join(path)).unwrap_or_else(|| root.join("target"))
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
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    let path = workspace_root().join("dist/build").join(platform).join("bin").join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    assert!(path.is_file(), "{} is missing: run mise run build", path.display());
    path
}

/* === Tests === */
