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

pub(crate) fn reads_regular_directories() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("plugin.dokime");
    std::fs::write(&file, b"bundle").unwrap();
    assert_eq!(dokime_packaging::bundle::read(&file).unwrap(), b"bundle");
    assert!(dokime_packaging::bundle::read(root.path()).is_err());
}

#[cfg(unix)]
pub(crate) fn refuses_existing_links() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("plugin.dokime");
    let link = root.path().join("link.dokime");
    std::fs::write(&file, b"bundle").unwrap();
    std::os::unix::fs::symlink(&file, &link).unwrap();
    assert!(dokime_packaging::bundle::read(&link).is_err());
    std::fs::remove_file(file).unwrap();
    assert!(dokime_packaging::bundle::read(&link).is_err());
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: reads_regular_directories");
        reads_regular_directories();
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: refuses_existing_links");
        refuses_existing_links();
    }
}
