/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(feature = "export")]

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, fs, io::Read, process::Command};
// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};
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
fn archive_descriptor_contract() {
    let root = tempfile::tempdir().unwrap();
    let export = |name: &str| {
        let destination = root.path().join(name);
        assert!(
            Command::new(env!("CARGO_BIN_EXE_export_protocol"))
                .arg(&destination)
                .arg("reproducibility-test")
                .status()
                .unwrap()
                .success()
        );
        let bytes = fs::read(&destination).unwrap();
        assert_eq!(
            fs::read_to_string(destination.with_extension("tar.sha256")).unwrap(),
            format!(
                "{}  {name}\n",
                Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
        );
        bytes
    };
    let first = export("first.tar");
    assert_eq!(first, export("second.tar"));
    let mut archive = tar::Archive::new(first.as_slice());
    let mut files = BTreeMap::new();
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        assert_eq!(entry.header().mtime().unwrap(), 0);
        let name = entry.path().unwrap().to_string_lossy().into_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        assert!(files.insert(name, bytes).is_none());
    }
    assert_eq!(files["dokime-v1.bin"], dokime_protocol::FILE_DESCRIPTOR_SET);
    let descriptor =
        prost_types::FileDescriptorSet::decode(files["dokime-v1.bin"].as_slice()).unwrap();
    for file in &descriptor.file {
        assert!(files.contains_key(&format!("proto/{}", file.name.as_ref().unwrap())));
        for dependency in &file.dependency {
            assert!(
                descriptor
                    .file
                    .iter()
                    .any(|file| file.name.as_ref() == Some(dependency))
            );
        }
    }
    let checksums = String::from_utf8(files.remove("SHA256SUMS").unwrap()).unwrap();
    let expected = files
        .iter()
        .map(|(name, bytes)| {
            format!(
                "{}  {name}\n",
                Sha256::digest(bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            )
        })
        .collect::<String>();
    assert_eq!(checksums, expected);
    let manifest: serde_json::Value = serde_json::from_slice(&files["manifest.json"]).unwrap();
    assert_eq!(manifest["source_revision"], "reproducibility-test");
    assert_eq!(
        manifest["api"]["descriptor_sha256"],
        Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
}

#[test]
fn failed_publication_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("blocked.tar");
    fs::create_dir(&destination).unwrap();
    let checksum = destination.with_extension("tar.sha256");
    fs::write(&checksum, "stale checksum").unwrap();
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_export_protocol"))
            .arg(&destination)
            .arg("publication-test")
            .status()
            .unwrap()
            .success()
    );
    assert!(!checksum.exists());
    assert!(destination.is_dir());
}
