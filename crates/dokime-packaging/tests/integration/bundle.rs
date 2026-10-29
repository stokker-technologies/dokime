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

#[test]
fn bundle() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("plugin.dokime");
    std::fs::write(&file, b"bundle").unwrap();
    assert_eq!(dokime_packaging::bundle::read(&file).unwrap(), b"bundle");
    assert!(dokime_packaging::bundle::read(root.path()).is_err());
    #[cfg(unix)]
    links(root.path(), &file);
}

#[cfg(unix)]
fn links(root: &std::path::Path, file: &std::path::Path) {
    let link = root.join("link.dokime");
    std::os::unix::fs::symlink(file, &link).unwrap();
    assert!(dokime_packaging::bundle::read(&link).is_err());
    std::fs::remove_file(file).unwrap();
    assert!(dokime_packaging::bundle::read(&link).is_err());
}
