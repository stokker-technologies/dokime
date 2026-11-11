/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Plugin Selection and Staging.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

// Third Party Imports

// Local Imports
use super::plugins_implementation::stage;
use crate::{
    Error,
    bundle::{COMPATIBILITY_FILE, Compatibility},
};
use dokime::harmos::plugin::inventory::contracts;
use harmos_packaging::bundle::{Bundle, Facility, Limits};
use harmos_packaging::bundle::{Executable, Identity, Source};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn bundle(package: &str, target: &str, requirement: &str) -> Vec<u8> {
    const OWNER: &str = "fixture";
    const LIFECYCLE: [dokime::harmos::operations::OperationPolicy; 2] = [dokime::operations::STOP_SETTLEMENT; 2];
    const SIZE: usize = contracts::encoded_fragments_len("sidecar", OWNER, &[], &[], &LIFECYCLE);
    const CONTRACTS: [u8; SIZE] = contracts::encode_fragments("sidecar", OWNER, &[], &[], &LIFECYCLE);
    let owner = OWNER;
    let prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap();
    let records = [
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"Fixture","config":"()"}),
        serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}}),
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]}),
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]}),
    ];
    let mut executable = String::from_utf8(CONTRACTS.to_vec()).unwrap();
    for record in records {
        executable.push_str(&format!("{prefix}{record}\0"));
    }
    let compatibility = Compatibility {
        schema: 1,
        requires_dokime: requirement.parse().unwrap(),
    };
    let bytes = serde_json::to_vec(&compatibility).unwrap();
    Bundle::build(
        Identity {
            name: package.into(),
            version: "1.0.0".parse().unwrap(),
            description: "Fixture".into(),
            package: Default::default(),
        },
        &[
            Source {
                path: "bin/fixture",
                bytes: executable.as_bytes(),
                executable: Some(Executable {
                    facility: Facility::Sidecar,
                    target: target.into(),
                    registrations: vec![],
                }),
            },
            Source {
                path: COMPATIBILITY_FILE,
                bytes: &bytes,
                executable: None,
            },
            Source {
                path: "assets/probe.txt",
                bytes: b"verified resource",
                executable: None,
            },
        ],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

fn selected(paths: &[PathBuf], scratch: &Path) -> Result<BTreeMap<String, PathBuf>, Error> {
    stage(paths, scratch, dokime::plugins::NATIVE_TARGET, "0.1.1")
}

/* === Tests === */

#[test]
fn explicit_plugins_staged() {
    let input = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let host = input.path().join("plugin-host.dokime");
    let bench = input.path().join("bench.dokime");
    std::fs::write(&host, bundle("plugin-host", dokime::plugins::NATIVE_TARGET, "^0.1")).unwrap();
    std::fs::write(&bench, bundle("bench", dokime::plugins::NATIVE_TARGET, "^0.1")).unwrap();

    assert!(selected(&[], scratch.path()).unwrap().is_empty());
    assert!(!scratch.path().join("plugins").exists());
    let staged = selected(std::slice::from_ref(&host), scratch.path()).unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(staged["plugin-host.dokime"], scratch.path().join("plugins/plugin-host.dokime"));
    assert_eq!(std::fs::read(&staged["plugin-host.dokime"]).unwrap(), std::fs::read(&host).unwrap());
    assert!(!scratch.path().join("plugins/bench.dokime").exists());

    let repeated = tempfile::tempdir().unwrap();
    let staged = selected(&[bench.clone(), bench.with_file_name("plugin-host.dokime")], repeated.path()).unwrap();
    assert_eq!(staged.len(), 2);
    assert_eq!(std::fs::read(&staged["bench.dokime"]).unwrap(), std::fs::read(bench).unwrap());

    use std::io::{Cursor, Write};
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive.start_file("plugin-host.dokime", zip::write::SimpleFileOptions::default()).unwrap();
    archive.write_all(&std::fs::read(&host).unwrap()).unwrap();
    let bytes = archive.finish().unwrap().into_inner();
    let path = input.path().join("external-sidecars.zip");
    std::fs::write(&path, bytes).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let staged = selected(&[path], scratch.path()).unwrap();
    assert_eq!(staged["plugin-host.dokime"], scratch.path().join("plugins/plugin-host.dokime"));
    assert_eq!(std::fs::read(&staged["plugin-host.dokime"]).unwrap(), std::fs::read(&host).unwrap());
}

#[test]
fn invalid_plugins_refused() {
    let input = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let host = input.path().join("plugin-host.dokime");
    std::fs::write(&host, b"bundle").unwrap();

    assert!(selected(&[host.clone(), host.clone()], scratch.path()).is_err());
    assert!(selected(&[host.clone(), host.with_file_name("HOST.dokime")], scratch.path()).is_err());
    assert!(selected(&[input.path().to_owned()], scratch.path()).is_err());
    assert!(selected(&[host.with_extension("zip")], scratch.path()).is_err());
    assert!(selected(&[host.with_file_name("missing.dokime")], scratch.path()).is_err());
    assert!(!scratch.path().join("plugins").exists());
}

#[test]
fn organization_entries_selected() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path().join("base");
    let organization = root.path().join("company/organization");
    std::fs::create_dir_all(base.join("plugins")).unwrap();
    std::fs::create_dir_all(organization.join("plugins")).unwrap();
    let host = bundle("plugin-host", dokime::plugins::NATIVE_TARGET, "^0.1");
    std::fs::write(base.join("plugins/plugin-host.dokime"), b"unselected base bundle").unwrap();
    std::fs::write(organization.join("plugins/plugin-host.dokime"), &host).unwrap();
    std::fs::write(organization.join("plugins/unlisted.dokime"), b"unselected storage").unwrap();

    let path = root.path().join("company/installer.toml");
    let text = "[installer]\nversion='0.1.1'\nformats=['nsis']\norganization='organization'\n[dokime]\n[[plugin]]\npath='plugins/plugin-host.dokime'";
    std::fs::write(&path, text).unwrap();
    let manifest = super::manifest::Manifest::read(&path).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let paths = manifest.plugin_paths(&base).unwrap();
    let staged = selected(&paths, scratch.path()).unwrap();
    assert_eq!(staged.len(), 1);
    assert_eq!(std::fs::read(&staged["plugin-host.dokime"]).unwrap(), host);

    std::fs::remove_file(organization.join("plugins/plugin-host.dokime")).unwrap();
    assert!(manifest.plugin_paths(&base).is_err());
    std::fs::write(&path, text.replace("organization='organization'\n", "")).unwrap();
    let manifest = super::manifest::Manifest::read(&path).unwrap();
    assert_eq!(manifest.plugin_paths(&base).unwrap(), [base.canonicalize().unwrap().join("plugins/plugin-host.dokime")]);

    for entry in ["", "name='plugin-host'\npath='plugins/plugin-host.dokime'", "path='../outside.dokime'"] {
        std::fs::write(&path, format!("[installer]\nversion='0.1.1'\nformats=['nsis']\n[dokime]\n[[plugin]]\n{entry}")).unwrap();
        assert!(super::manifest::Manifest::read(&path).unwrap_err().to_string().contains("entry 1"));
    }
}

#[cfg(unix)]
#[test]
fn organization_plugin_escape_refused() {
    let root = tempfile::tempdir().unwrap();
    let organization = root.path().join("organization");
    let outside = root.path().join("outside");
    std::fs::create_dir(&organization).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("plugin-host.dokime"), b"outside").unwrap();
    std::os::unix::fs::symlink(&outside, organization.join("plugins")).unwrap();
    let manifest: super::manifest::Manifest =
        toml::from_str("[installer]\nversion='0.1.1'\nformats=['deb']\n[dokime]\n[[plugin]]\npath='plugins/plugin-host.dokime'").unwrap();
    assert!(manifest.plugin_paths(&organization).is_err());
}

#[test]
fn mismatches_name_bundle() {
    let input = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let path = input.path().join("plugin-host.dokime");
    for bytes in [
        bundle("plugin-host", "foreign-target", "^0.1"),
        bundle("plugin-host", dokime::plugins::NATIVE_TARGET, ">=99"),
        b"damaged archive".to_vec(),
    ] {
        std::fs::write(&path, bytes).unwrap();
        let error = selected(std::slice::from_ref(&path), scratch.path()).unwrap_err();
        assert!(error.to_string().contains("plugin-host.dokime"));
        assert!(!scratch.path().join("plugins").exists());
    }
    // A Valid Archive With An Unchanged Manifest Cannot Hide Altered Member Bytes.
    let mut bytes = bundle("plugin-host", dokime::plugins::NATIVE_TARGET, "^0.1");
    let offset = {
        let mut archive = tar::Archive::new(bytes.as_slice());
        let member =
            archive.entries().unwrap().map(Result::unwrap).find(|entry| entry.path().unwrap() == Path::new("assets/probe.txt")).unwrap();
        usize::try_from(member.raw_file_position()).unwrap()
    };
    bytes[offset] ^= 1;
    std::fs::write(&path, bytes).unwrap();
    let error = selected(&[path], scratch.path()).unwrap_err();
    assert!(error.to_string().contains("plugin-host.dokime"));
    assert!(!scratch.path().join("plugins").exists());
}
