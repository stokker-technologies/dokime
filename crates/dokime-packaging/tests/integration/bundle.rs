/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[test]
fn reads_regular_directories() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("plugin.dokime");
    std::fs::write(&file, b"bundle").unwrap();
    assert_eq!(dokime_packaging::bundle::read(&file).unwrap(), b"bundle");
    assert!(dokime_packaging::bundle::read(root.path()).is_err());
}

#[cfg(unix)]
#[test]
fn refuses_existing_links() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("plugin.dokime");
    let link = root.path().join("link.dokime");
    std::fs::write(&file, b"bundle").unwrap();
    std::os::unix::fs::symlink(&file, &link).unwrap();
    assert!(dokime_packaging::bundle::read(&link).is_err());
    std::fs::remove_file(file).unwrap();
    assert!(dokime_packaging::bundle::read(&link).is_err());
}
