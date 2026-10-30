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
use dokime::harmos::plugin::inventory::{Declaration, Facility, contracts};
use harmos_packaging::bundle::{Bundle, Executable, Identity, Limits, Source};

// Local Imports

/* === Constants === */

const CONTRACT_COUNT: usize = 256;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Reading;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::operations::OperationDeclaration for Reading {
    type Request = ();
    type Response = ();
    type Input = dokime::harmos::operations::NoStream;
    type Output = dokime::harmos::operations::NoStream;
    const ID: &'static str = "fixture.read";
    const POLICY: dokime::harmos::operations::OperationPolicy = dokime::operations::STOP_SETTLEMENT;
}

/* === Public Functions === */

/* === Private Functions === */

fn metadata() -> Vec<u8> {
    use dokime::harmos::{catalog::Node, operations::WireDeclaration};
    const POLICY: [dokime::harmos::operations::OperationPolicy; 2] = [dokime::operations::STOP_SETTLEMENT; 2];
    const PROVIDES: &[Node<WireDeclaration>] = &[Node::Leaf(WireDeclaration::of::<Reading>())];
    const SIZE: usize = contracts::encoded_fragments_len("sidecar", "fixture", PROVIDES, &[], &POLICY);
    const CONTRACTS: [u8; SIZE] = contracts::encode_fragments("sidecar", "fixture", PROVIDES, &[], &POLICY);
    let mut parts: Vec<serde_json::Value> = dokime::harmos::artifacts::fragments(&CONTRACTS, &["contracts"]).unwrap();
    let provided = parts.pop().unwrap();
    assert_eq!(provided["entry"]["direction"], "provides");
    for index in 0..CONTRACT_COUNT {
        let mut contract = provided.clone();
        contract["entry"]["contract"]["operation"]["id"] = format!("fixture.read_{index}").into();
        contract["entry"]["contract"]["operation"]["description"] = "x".repeat(8192).into();
        parts.push(contract);
    }
    parts.extend([
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":"fixture","version":"1.0.0","description":"Large Fixture","config":"()"}),
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":"fixture","streams":[]}),
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":"fixture","observers":[]}),
    ]);
    let mut binary = Vec::new();
    for part in parts {
        let encoded = serde_json::to_vec(&part).unwrap();
        assert!(encoded.len() <= dokime::harmos::artifacts::FRAGMENT_LIMIT);
        binary.extend_from_slice(dokime::harmos::artifacts::PREFIX);
        binary.extend(encoded);
        binary.push(0);
    }
    for marker in
        dokime::harmos::schema::declaration::markers_many([WireDeclaration::of::<Reading>().request], &dokime::config::schema_defaults())
            .unwrap()
    {
        binary.extend_from_slice(marker);
    }
    binary
}

fn package(binary: &[u8], settings: &dokime::harmos::config::SchemaSettings) -> Result<Bundle, harmos_packaging::bundle::Error> {
    Bundle::build_with_settings(
        Identity {
            name: "large-fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: String::new(),
            package: Default::default(),
        },
        &[Source {
            path: "bin/fixture",
            bytes: binary,
            executable: Some(Executable {
                facility: harmos_packaging::bundle::Facility::Sidecar,
                target: dokime::plugins::NATIVE_TARGET.into(),
                registrations: vec![],
            }),
        }],
        Limits::default(),
        settings,
    )
}

/* === Tests === */

#[tokio::test]
async fn large_metadata_admitted() {
    let binary = metadata();
    assert!(Declaration::inspect(&binary, Facility::Sidecar).is_err());
    let settings = dokime::config::schema_defaults();
    let bundle = package(&binary, &settings).unwrap();
    assert_eq!(bundle.declaration("bin/fixture").unwrap().contracts.provides.len(), CONTRACT_COUNT);
    let (_, metadata) = dokime::plugins::inspect(bundle.bytes(), &settings).unwrap();
    assert_eq!(metadata.owners, ["fixture"]);
    let root = tempfile::tempdir().unwrap();
    let extracted = bundle.extract(root.path()).unwrap();
    let executable = dokime::harmos::sidecar::Executable::at_with_settings(extracted.path().join("bin/fixture"), &settings).unwrap();
    assert_eq!(executable.declaration(), bundle.declaration("bin/fixture").unwrap());
    let database = dokime::storage::sqlite::State::open(&root.path().join("data")).await.unwrap();
    let saved = dokime::config::Settings::open(dokime::paths::ApplicationPaths::new(root.path()), database.clone()).await.unwrap();
    let store = dokime::plugins::PluginStore::new(saved, settings);
    let installed = store.install(bundle.bytes(), &[], &Default::default()).await.unwrap();
    assert_eq!(installed.metadata.unwrap().owners, ["fixture"]);
    drop(store);
    database.close().await.unwrap();
}

#[test]
fn metadata_budget_refused() {
    use std::num::NonZeroUsize;
    let binary = metadata();
    let defaults = dokime::config::schema_defaults();
    let previous = dokime::harmos::config::SchemaSettings::default();
    let bundle = package(&binary, &defaults).unwrap();
    for budget in ["memory bytes", "work"] {
        let mut settings = defaults;
        let previous_limit = if budget == "work" { previous.preparation_work } else { previous.preparation_bytes };
        let allowance = NonZeroUsize::new(previous_limit.get() * 2).unwrap();
        match budget {
            "work" => settings.preparation_work = allowance,
            _ => settings.preparation_bytes = allowance,
        }
        let packaging = package(&binary, &settings).err().unwrap().to_string();
        let admission = dokime::plugins::inspect(bundle.bytes(), &settings).err().unwrap().to_string();
        for error in [packaging, admission] {
            assert!(error.contains("large-fixture"), "{error}");
            assert!(error.contains(budget), "{error}");
            assert!(error.contains(&format!("allowed {}", allowance.get())), "{error}");
            let used: usize = error.split("used ").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
            assert!(used > allowance.get(), "{error}");
        }
    }
}

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
