/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Terms File Containment Through The Public Manifest Reader

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_packaging::{Error, installer::Manifest};

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
fn license_file_containment() {
    let pack = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir(pack.path().join("terms")).unwrap();
    std::fs::write(pack.path().join("terms/EULA.txt"), "terms").unwrap();
    std::fs::write(outside.path().join("EULA.txt"), "terms").unwrap();
    let escape = format!("../{}/EULA.txt", outside.path().file_name().unwrap().to_str().unwrap());

    let manifest = pack.path().join("installer.toml");
    let read = |file: &str| {
        let text = format!(
            "[installer]\nversion='1.2.3'\nformats=[]\n[dokime]\nversion='1.2.3'\n[license]\nspdx='LicenseRef-Acme-Proprietary'\nfile={file:?}"
        );
        std::fs::write(&manifest, text).unwrap();
        Manifest::read(&manifest)
    };
    let license = read("terms/EULA.txt").unwrap().license.unwrap();
    assert_eq!(license.file.as_deref(), Some(pack.path().join("terms/EULA.txt").canonicalize().unwrap().as_path()));

    for file in [escape.as_str(), "/EULA.txt", "C:/EULA.txt", r"terms\EULA.txt", "missing.txt", "terms"] {
        let Err(Error::License { field, .. }) = read(file) else { panic!("{file}") };
        assert_eq!(field, "file", "{file}");
    }
}
