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
    let path = std::path::PathBuf::from(
        std::env::var_os("DOKIME_TEST_TOOL")
            .unwrap_or_else(|| panic!("DOKIME_TEST_TOOL is unset: run mise run test:acceptance")),
    );
    assert!(
        path.is_file(),
        "release tool is missing: run mise run build:release ({})",
        path.display()
    );
    path
}

pub fn standalone_directory() -> tempfile::TempDir {
    standalone_directory_in(&std::env::temp_dir())
}

/* === Private Functions === */
fn standalone_directory_in(preferred: &std::path::Path) -> tempfile::TempDir {
    let preferred = preferred.canonicalize().unwrap();
    let mut parent = preferred.as_path();
    for ancestor in preferred.ancestors() {
        if ancestor.join("Cargo.toml").is_file() {
            parent = ancestor
                .parent()
                .expect("filesystem root cannot be a Cargo workspace");
        }
    }
    tempfile::Builder::new()
        .prefix("dokime-plugin-")
        .tempdir_in(parent)
        .unwrap()
}

/* === Tests === */
pub(super) fn nested_scratch_independent() {
    let repository = tempfile::tempdir().unwrap();
    let manifest = repository.path().join("Cargo.toml");
    let original = "[workspace]\nmembers = []\n";
    std::fs::write(&manifest, original).unwrap();
    let scratch = repository.path().join(".tmp");
    std::fs::create_dir(&scratch).unwrap();
    let standalone = standalone_directory_in(&scratch);
    assert!(!standalone.path().starts_with(repository.path()));
    assert!(
        standalone
            .path()
            .ancestors()
            .all(|ancestor| !ancestor.join("Cargo.toml").exists())
    );
    assert_eq!(std::fs::read_to_string(manifest).unwrap(), original);
}
