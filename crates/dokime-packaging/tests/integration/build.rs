/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Optional Debug Symbol Packaging.

/* === Modules === */

#[path = "../../src/build/mod.rs"]
pub mod implementation;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::harmos::plugin::inventory::contracts;
use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, PackageMetadata, Source};
use implementation::symbols;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

// A non-running executable carrying the minimal native declaration.
fn inert_executable() -> String {
    const OWNER: &str = "fixture";
    const POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
        timeout: Duration::from_secs(5),
        attempt_timeout: Duration::from_secs(5),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: dokime::harmos::operations::RetryPolicy::Never,
    };
    const SIZE: usize = contracts::encoded_fragments_len("sidecar", OWNER, &[], &[], &[POLICY; 2]);
    const CONTRACTS: [u8; SIZE] = contracts::encode_fragments("sidecar", OWNER, &[], &[], &[POLICY; 2]);
    let owner = OWNER;
    let sections = [
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"}),
        serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}}),
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]}),
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]}),
    ];
    let prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap();

    let executable = String::from_utf8(CONTRACTS.to_vec()).unwrap();
    sections.iter().fold(executable, |executable, section| format!("{executable}{prefix}{section}\0"))
}

/* === Tests === */

#[test]
fn oversized_symbols_are_skipped() {
    let root = tempfile::tempdir().unwrap();
    let pdb = root.path().join("plugin.pdb");
    std::fs::write(&pdb, vec![0xAB; 2048]).unwrap();
    let terminal = dokime::terminal::Terminal::default();
    let target = "x86_64-pc-windows-msvc";

    // Over The Limit The Symbols Are Left Out; Within It They Ship
    assert_eq!(symbols(&pdb, target, 1024, &terminal).unwrap(), None);
    let kept = symbols(&pdb, target, 4096, &terminal).unwrap().unwrap();
    assert_eq!(kept.0, format!("bin/{target}/plugin.pdb"));

    // The Bundle Still Builds from the Executable Alone
    let executable = inert_executable();
    let sources = [Source {
        path: "bin/x86_64-pc-windows-msvc/fixture.exe",
        bytes: executable.as_bytes(),
        executable: Some(Executable {
            facility: Facility::Sidecar,
            target: target.into(),
            registrations: vec![],
        }),
    }];
    let identity = Identity {
        name: "fixture".into(),
        version: "1.0.0".parse().unwrap(),
        description: "A fixture.".into(),
        package: PackageMetadata::default(),
    };
    assert!(Bundle::build(identity, &sources, Limits::default()).is_ok());
}

#[test]
fn debug_files_extract_beside_the_executable() {
    let root = tempfile::tempdir().unwrap();
    let terminal = dokime::terminal::Terminal::default();
    for (target, executable_name, symbol_names) in [
        ("x86_64-pc-windows-msvc", "fixture.exe", vec!["fixture.pdb"]),
        ("x86_64-unknown-linux-gnu", "fixture", vec!["fixture.dwp", "unit.dwo", "fixture.debug"]),
    ] {
        let executable = inert_executable();
        let executable_path = format!("bin/{target}/{executable_name}");
        let mut debug_files = Vec::new();
        for name in symbol_names {
            let path = root.path().join(name);
            std::fs::write(&path, name.as_bytes()).unwrap();
            debug_files.push(symbols(&path, target, 4096, &terminal).unwrap().unwrap());
        }
        let mut sources = vec![Source {
            path: &executable_path,
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: target.into(),
                registrations: vec![],
            }),
        }];
        sources.extend(debug_files.iter().map(|(path, bytes)| Source { path, bytes, executable: None }));
        let identity = Identity {
            name: "fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "A fixture.".into(),
            package: PackageMetadata::default(),
        };
        let bundle = Bundle::build(identity, &sources, Limits::default()).unwrap();
        let extracted = bundle.extract(root.path()).unwrap();
        let executable = extracted.path().join(&executable_path);
        assert!(executable.is_file());
        for (path, bytes) in debug_files {
            let symbol = extracted.path().join(path);
            assert_eq!(symbol.parent(), executable.parent());
            assert_eq!(std::fs::read(symbol).unwrap(), bytes);
        }
    }
}
