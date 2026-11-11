/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Native License and Copyright Staging.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};

// Third Party Imports

// Local Imports
use super::{
    distribution, license,
    manifest::Manifest,
    native_implementation::{Staging, stage_copyright, stage_licenses, validate_signing},
};

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
fn license_staging() {
    let source = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    std::fs::write(source.path().join("LICENSE"), "apache").unwrap();
    std::fs::write(source.path().join("terms.txt"), "acme terms").unwrap();
    let text = "[installer]
version='1.2.3'
formats=['deb']
[dokime]
target='x86_64-unknown-linux-gnu'";
    let mut manifest: Manifest = toml::from_str(text).unwrap();
    let empty = || Staging {
        resources: HashMap::new(),
        linux_files: HashMap::new(),
        files: BTreeMap::new(),
    };
    let prefix = "usr/lib/acme/";

    // Without A License Section Only Dokime's Apache-2.0 License Is Staged And Shown.
    let mut staging = empty();
    let (expression, page) = stage_licenses(&manifest, source.path(), scratch.path(), prefix, &mut staging).unwrap();
    assert_eq!(expression, "Apache-2.0");
    assert_eq!(std::fs::read(&page).unwrap(), b"apache");
    assert_eq!(staging.files, BTreeMap::from([(format!("{prefix}LICENSE"), distribution::digest(b"apache"))]));
    assert_eq!(staging.linux_files.keys().collect::<Vec<_>>(), [Path::new("/usr/lib/acme/LICENSE")]);
    assert_eq!(staging.resources.values().collect::<Vec<_>>(), ["LICENSE"]);

    // Pack Terms Are Added Beside The Unchanged Apache-2.0 License.
    let terms = source.path().join("terms.txt");
    manifest.license = Some(super::manifest::License {
        spdx: Some("LicenseRef-Acme-Proprietary".into()),
        file: Some(terms),
    });
    let scratch = tempfile::tempdir().unwrap();
    let mut staging = empty();
    let (expression, page) = stage_licenses(&manifest, source.path(), scratch.path(), prefix, &mut staging).unwrap();
    assert_eq!(expression, "LicenseRef-Acme-Proprietary AND Apache-2.0");
    assert_eq!(page.file_name().unwrap(), "EULA.txt");
    assert_eq!(std::fs::read(&page).unwrap(), b"acme terms");
    let expected = BTreeMap::from([
        (format!("{prefix}LICENSE"), distribution::digest(b"apache")),
        (format!("{prefix}EULA.txt"), distribution::digest(b"acme terms")),
    ]);
    assert_eq!(staging.files, expected);
    assert_eq!(staging.linux_files.get(Path::new("/usr/lib/acme/EULA.txt")), Some(&page));
    assert!(staging.linux_files.contains_key(Path::new("/usr/lib/acme/LICENSE")));
    let mut installed: Vec<_> = staging.resources.values().cloned().collect();
    installed.sort();
    assert_eq!(installed, ["EULA.txt", "LICENSE"]);
}

#[test]
fn copyright_staging() {
    let source = tempfile::tempdir().unwrap();
    std::fs::write(source.path().join("LICENSE"), "apache").unwrap();
    std::fs::write(source.path().join("terms.txt"), "acme terms").unwrap();
    let text = "[installer]
version='1.2.3'
formats=['deb', 'rpm']
[dokime]
target='x86_64-unknown-linux-gnu'";
    let mut manifest: Manifest = toml::from_str(text).unwrap();
    manifest.license = Some(super::manifest::License {
        spdx: Some("LicenseRef-Acme-Proprietary".into()),
        file: Some(source.path().join("terms.txt")),
    });
    let pack = dokime::config::organization::ResolvedPack {
        manifest: toml::from_str("[app.organization]\nname = 'Acme Corporation'").unwrap(),
        files: BTreeMap::new(),
    };
    let scratch = tempfile::tempdir().unwrap();
    let mut staging = Staging {
        resources: HashMap::new(),
        linux_files: HashMap::new(),
        files: BTreeMap::new(),
    };

    // The Linux Payload Maps The Rendered File Under The Package's Doc Folder And Hashes It.
    stage_copyright(&manifest, &pack, source.path(), scratch.path(), "acme", &mut staging).unwrap();
    let staged = &staging.linux_files[Path::new("/usr/share/doc/acme/copyright")];
    let bytes = std::fs::read(staged).unwrap();
    let expected = license::debian_copyright(
        &[license::DOKIME_HOLDER, "Acme Corporation"],
        "apache",
        Some(("LicenseRef-Acme-Proprietary", "acme terms")),
    );
    assert_eq!(bytes, expected.as_bytes());
    assert_eq!(staging.files, BTreeMap::from([("usr/share/doc/acme/copyright".to_owned(), distribution::digest(&bytes))]));
    assert!(staging.resources.is_empty());
}

#[test]
fn updater_versions() {
    let text = "[installer]\nversion='1.2.4'\nformats=['nsis']\n[dokime]\nversion='1.2.3'\ntarget='x86_64-pc-windows-msvc'\n[signing.updater]\nprivate_key_env='FIXTURE_KEY'\npassword_env='FIXTURE_PASSWORD'\nartifact_base_url='https://example.com'";
    let manifest: Manifest = toml::from_str(text).unwrap();
    assert!(validate_signing(&manifest, false).is_ok());
    let error = validate_signing(&manifest, true).unwrap_err();
    assert!(error.to_string().contains("matching installer and Dokime versions"));
}
