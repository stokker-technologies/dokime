/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authoring attributes compiled by a real `rustc` against a freshly built SDK:
// each story compiles one fixture and inspects its diagnostics or inventory.

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

struct SdkArtifact {
    library: std::path::PathBuf,
    serde: std::path::PathBuf,
    native_paths: Vec<std::path::PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn sdk_artifact() -> &'static SdkArtifact {
    static SDK: std::sync::OnceLock<SdkArtifact> = std::sync::OnceLock::new();
    SDK.get_or_init(|| {
        let manifest = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
            .join("../dokime/Cargo.toml")
            .canonicalize()
            .unwrap();
        let workspace = manifest.parent().unwrap().parent().unwrap().parent().unwrap();
        let target = std::env::var_os("CARGO_TARGET_DIR").map(|path| workspace.join(path)).unwrap_or_else(|| workspace.join("target"));
        // Both Graph Profiles Compile Tests In Cargo's Debug Directory.
        let built_tests = std::path::PathBuf::from(
            std::env::var_os("DOKIME_COMPILE_CONTRACTS_DIRECTORY").expect("the Build owner supplies its selected Tests export"),
        );
        let messages = built_tests.join("compiler.json");
        let built = std::fs::read_to_string(&messages)
            .unwrap_or_else(|error| panic!("run mise run build:rust:tests to produce {}: {error}", messages.display()));
        // Only the SDK's own dependency closure links into a plugin: another member's build script output (the
        // desktop shell's import stub libraries) would shadow the platform's runtime libraries on the search path.
        let closure = sdk_closure(&built_tests.join("metadata.json"), &manifest);
        let mut artifacts = Vec::new();
        let mut serde = None;
        let mut native_paths = Vec::new();
        for line in built.lines() {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };

            if value["reason"] == "build-script-executed" {
                if !value["package_id"].as_str().is_some_and(|id| closure.contains(id)) {
                    continue;
                }
                if let Some(paths) = value["linked_paths"].as_array() {
                    for path in paths.iter().filter_map(|path| path.as_str()) {
                        let Some(path) = path.strip_prefix("native=") else {
                            continue;
                        };
                        native_paths.push(std::path::PathBuf::from(path));
                    }
                }
                continue;
            }

            // A Standalone Plugin Compile Records Its Own Narrower Copies Beside The Workspace's; The
            // Workspace Unit Graph Is The Feature Superset, So Its SDK And Serde Link Together.
            if value["reason"] == "compiler-artifact" && value["target"]["name"] == "serde" {
                let features = value["features"].as_array().map_or(0, Vec::len);
                let library = value["filenames"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|name| name.as_str())
                    .find(|name| name.ends_with(".rlib"))
                    .map(std::path::PathBuf::from);
                if let Some(library) = library
                    && serde.as_ref().is_none_or(|(widest, _)| features > *widest)
                {
                    serde = Some((features, library));
                }
            }

            if value["reason"] != "compiler-artifact"
                || !value["manifest_path"].as_str().is_some_and(|path| same_path(&manifest, path))
                || value["target"]["name"] != "dokime"
            {
                continue;
            }
            let features = value["features"].as_array().unwrap();
            assert!(features.iter().any(|feature| feature == "native"));
            assert!(features.iter().any(|feature| feature == "derive"));
            artifacts.extend(
                value["filenames"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|name| name.as_str())
                    .map(std::path::PathBuf::from)
                    .filter(|path| path.extension().is_some_and(|extension| extension == "rlib"))
                    .map(|path| (features.len(), path)),
            );
        }
        native_paths.sort_unstable();
        native_paths.dedup();
        artifacts.sort();
        let widest = artifacts.last().map(|(features, _)| *features);
        assert_eq!(
            artifacts.iter().filter(|(features, _)| Some(*features) == widest).count(),
            1,
            "Cargo must identify exactly one workspace SDK rlib"
        );
        let library = artifacts.pop().unwrap().1;
        let serde = serde.expect("Cargo must identify the SDK serde library").1;
        let target = target.canonicalize().unwrap();
        assert!(library.canonicalize().unwrap().starts_with(&target));
        assert!(serde.canonicalize().unwrap().starts_with(&target));
        SdkArtifact { library, serde, native_paths }
    })
}

// Every package the SDK depends on, by Cargo package id, from the build's resolved metadata.
fn sdk_closure(metadata: &std::path::Path, manifest: &std::path::Path) -> std::collections::BTreeSet<String> {
    let metadata: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(metadata)
            .unwrap_or_else(|error| panic!("run mise run build:tests to produce {}: {error}", metadata.display())),
    )
    .unwrap();
    let root = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["manifest_path"].as_str().is_some_and(|path| same_path(manifest, path)))
        .and_then(|package| package["id"].as_str())
        .expect("the build's metadata names the SDK package")
        .to_owned();
    let nodes = metadata["resolve"]["nodes"].as_array().unwrap();
    let mut closure = std::collections::BTreeSet::new();
    let mut pending = vec![root];
    while let Some(id) = pending.pop() {
        if !closure.insert(id.clone()) {
            continue;
        }
        let node = nodes.iter().find(|node| node["id"] == id.as_str()).expect("every resolved package has a node");
        pending.extend(node["deps"].as_array().unwrap().iter().filter_map(|dependency| dependency["pkg"].as_str()).map(str::to_owned));
    }
    closure
}

fn same_path(left: &std::path::Path, right: &str) -> bool {
    match (left.canonicalize(), std::path::Path::new(right).canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == std::path::Path::new(right),
    }
}

fn compile(source: &str) -> (std::process::Output, std::path::PathBuf) {
    compile_configured(source, None)
}

fn compile_configured(source: &str, configuration: Option<&str>) -> (std::process::Output, std::path::PathBuf) {
    let sdk = sdk_artifact();
    let dependencies = sdk.library.parent().unwrap();
    assert!(dependencies.is_dir(), "Cargo dependency directory missing");
    let directory = dependencies.join(format!("inventory-artifacts-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let stem = std::path::Path::new(source).file_stem().unwrap().to_str().unwrap();
    let output = directory.join(format!("{stem}{}", std::env::consts::EXE_SUFFIX));
    let mut rustc = sdk::domain::process::command("rustc");
    // Fixtures build as a plugin crate's runtime build, where Harmos serves operations.
    rustc.args(["--cfg", "feature=\"runtime\""]);
    if let Some(configuration) = configuration {
        rustc.args(["--cfg", configuration]);
    }
    let built = rustc
        .arg("--edition=2024")
        .arg("--crate-type=bin")
        .arg(
            std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
                .join("tests")
                .join(source),
        )
        .arg("--extern")
        .arg(format!("sdk={}", sdk.library.display()))
        .arg("--extern")
        .arg(format!("serde={}", sdk.serde.display()))
        .arg("-L")
        .arg(format!("dependency={}", dependencies.display()))
        .args(sdk.native_paths.iter().flat_map(|path| ["-L".to_owned(), format!("native={}", path.display())]))
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    (built, output)
}

fn compile_observer_catalogs(
    name: &str, arguments: &str, peer_stream: &str, peer_interval: u64,
) -> (std::process::Output, std::path::PathBuf) {
    let source = r#"
extern crate sdk;

use std::sync::Arc;

use sdk::harmos::plugin::{Attachment, Declared};
use sdk::harmos::operations::{OperationPolicy, RetryPolicy, ServingError, WireTable};
use sdk::harmos::sidecar::{ObserverAttachment, ObserverContext, PublishError, Publisher};

const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: RetryPolicy::Never,
};

#[sdk::stream(id = "shared.sample", window = 8)]
pub struct Sample { value: u64 }

// One stream id under another row: a module of its own keeps the two
// declarations apart until the plugin gathers them.
mod conflicting {
    #[sdk::stream(id = "shared.sample", window = 8)]
    pub struct Conflicting { pub other: f64 }
}
use conflicting::Conflicting;

#[sdk::catalog]
enum Observers { First(First), Second(Second) }

#[sdk::catalog]
enum Operations {}

type Observing = ObserverAttachment<(), Publisher<Shared>>;

struct Shared { observers: Option<Arc<Observing>> }


impl sdk::harmos::builtins::lifecycle::Policy for Shared {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "shared",
    provides = Operations,
    requires(),
    PLUGIN_ARGUMENTS
    observers = Observers,
    restart = "never",
)]
impl sdk::Plugin for Shared {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    type Resources = ();

    fn prepare(_: sdk::plugin::Configuration) -> Result<Self, sdk::Refusal> {
        Ok(Self { observers: None })
    }

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<Observers>(
            self.resources(),
            <Self as Declared>::STREAMS,
        )?);
        Ok(())
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, table: &mut WireTable) -> Result<(), ServingError> {
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<sdk::Supervisor>>> {
        self.observers
            .as_ref()
            .map(|observers| Arc::clone(observers) as Arc<dyn Attachment<sdk::Supervisor>>)
    }

    async fn stop(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}

#[derive(Default)]
struct First;
#[sdk::observer(interval_ms = 10, streams(Sample), activation = "demand")]
impl First {
    async fn observe(&mut self, _: &ObserverContext<'_, (), Publisher<Shared>>) -> Result<(), PublishError> {
        Ok(())
    }
}

#[derive(Default)]
struct Second;
#[sdk::observer(interval_ms = PEER_INTERVAL, streams(PEER_STREAM), activation = "demand")]
impl Second {
    async fn observe(&mut self, _: &ObserverContext<'_, (), Publisher<Shared>>) -> Result<(), PublishError> {
        Ok(())
    }
}

fn main() {
    assert_eq!(<Shared as Declared>::STREAMS.len(), 1);
    let _ = <Shared as Declared>::markers();
}
"#
    .replace("PLUGIN_ARGUMENTS", arguments)
    .replace("PEER_STREAM", peer_stream)
    .replace("PEER_INTERVAL", &peer_interval.to_string());
    let directory = sdk_artifact().library.parent().unwrap().join(format!("inventory-artifacts-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join(format!("{name}.rs"));
    std::fs::write(&source_path, source).unwrap();

    let result = compile(source_path.to_str().unwrap());
    std::fs::remove_file(source_path).unwrap();
    result
}

fn artifact(source: &str) -> Vec<u8> {
    let (built, output) = compile(source);
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let bytes = std::fs::read(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    bytes
}

fn declaration_of(source: &str) -> sdk::harmos::plugin::inventory::Declaration {
    sdk::harmos::plugin::inventory::Declaration::inspect(&artifact(source), sdk::harmos::plugin::inventory::Facility::Sidecar).unwrap()
}

/* === Tests === */

#[test]
fn catalog_identifiers_are_unique() {
    for (configuration, identifier) in [
        ("duplicate_suites", "checks"),
        ("duplicate_templates", "checks.probe"),
        ("duplicate_records", "checks.record"),
        ("duplicate_streams", "checks.samples"),
    ] {
        let (built, _) = compile_configured("fixtures/catalog_identifiers.rs", Some(configuration));
        let diagnostics = String::from_utf8_lossy(&built.stderr);
        assert!(!built.status.success(), "{configuration} compiled");
        // Rust 1.99 supports the single-str panic format in const evaluation,
        // so diagnostics must name the actual duplicate, even for const ids.
        assert!(diagnostics.contains(&format!("evaluation panicked: {identifier}")), "{diagnostics}");
        assert!(diagnostics.contains("assert_unique"), "{diagnostics}");
        assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
    }

    let (built, output) = compile("fixtures/catalog_identifiers.rs");
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    std::fs::remove_file(output).unwrap();
}

#[test]
fn alias_inventory_declared() {
    let declaration = declaration_of("integration/renamed_sdk.rs");
    assert_eq!(declaration.id, "renamed");
    assert_eq!(
        declaration.streams.iter().map(|row| (row.id.as_str(), row.schema, row.window)).collect::<Vec<_>>(),
        [("renamed.direct", 1, 8), ("renamed.observations", 1, 64)],
        "outputs appear once even when referenced by an observer",
    );
    assert_eq!(
        declaration.operations.iter().map(|operation| operation.id.as_str()).collect::<Vec<_>>(),
        [
            // Authored operations follow their selected catalog order, and the
            // SDK adds none of its own to a plugin that lists no suites.
            "renamed.get_echo",
            "renamed.get_status",
            "renamed.get_spare_echo",
        ]
    );
    assert_eq!(declaration.observe_contract().expect("observers serve their built-in control").operation().id.as_str(), "harmos_observe");
    assert_eq!(declaration.observers.len(), 1);
    assert_eq!(declaration.observers[0].streams, ["renamed.observations"]);
    let namespaces = declaration.sections.iter().map(|section| section.namespace.to_string()).collect::<Vec<_>>();
    assert_eq!(namespaces, ["dokime.records", "dokime.telemetry", "dokime.settings"]);
}

#[test]
fn empty_plugin_identity() {
    let declaration = declaration_of("fixtures/routeless.rs");
    assert_eq!(declaration.id, "routeless");
    assert!(declaration.operations.is_empty(), "no implicit operations");
    assert!(declaration.streams.is_empty());
    assert!(declaration.observers.is_empty());
    assert!(declaration.sections.is_empty());
}

#[test]
fn invalid_identity_refused() {
    let (built, _) = compile("fixtures/invalid_identity.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The plugin's identity is checked on the line it was written, because it is
    // the one every operation and stream it serves is scoped under.
    assert_eq!(diagnostics.matches("a plugin id is one owner segment").count(), 1, "{diagnostics}");
    assert!(diagnostics.contains("aborting due to 1 previous error"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn hyphenated_identity_refused() {
    let (built, _) = compile("fixtures/hyphenated_identity.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // Harmos qualifies served operations as `<plugin id>.<operation>`, so a
    // hyphen that an artifact id would allow is refused as an owner segment.
    assert_eq!(diagnostics.matches("a plugin id is one owner segment").count(), 1, "{diagnostics}");
    assert!(diagnostics.contains("aborting due to 1 previous error"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn suiteless_template_refused() {
    let (built, _) = compile("fixtures/template_identity.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // An omitted id cannot be derived, because the item name never spells the
    // suite; a written literal is read on its own line.
    assert!(diagnostics.contains("`id = …` is required: a template is referenced as `<suite id>.<segment>`"), "{diagnostics}");
    for refused in ["`checks.Probe` is not a template id", "`probe` is not a template id"] {
        assert!(diagnostics.contains(refused), "{diagnostics}");
    }
    assert!(diagnostics.contains("aborting due to 3 previous errors"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn suite_refuses_templates() {
    let (built, _) = compile("fixtures/suite_templates.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The suite owns the prefix, so it is the suite that refuses a template id
    // outside it and two templates answering to one reference.
    assert!(diagnostics.contains("a template id is its suite's id followed by `.` and one segment"), "{diagnostics}");
    assert!(diagnostics.contains("two templates of one suite declare the same id"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn duplicate_operations_refused() {
    let (built, output) = compile("fixtures/duplicate_operations.rs");
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let bytes = std::fs::read(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    let refused = sdk::harmos::plugin::inventory::Declaration::inspect(&bytes, sdk::harmos::plugin::inventory::Facility::Sidecar)
        .expect_err("two operations under one id are one operation too many");
    assert!(refused.to_string().contains("duplicate ordinary operation `duplicated.read`"), "{refused}");
}

#[test]
fn stream_declares_shape() {
    let (built, _) = compile("fixtures/stream_shapes.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    assert!(diagnostics.contains("a stream requires named row fields"), "{diagnostics}");
    assert!(diagnostics.contains("a foreign stream declares the columns this plugin reads"), "{diagnostics}");
    assert!(diagnostics.contains("a named stream with `row = Shape` must be a unit struct"), "{diagnostics}");
    assert!(diagnostics.contains("aborting due to 3 previous errors"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn foreign_stream_unpublished() {
    let (built, _) = compile("fixtures/unpublishable_streams.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    for unsatisfied in [
        "the trait bound `GatewayRssi: Publishable` is not satisfied",
        "the trait bound `GatewayRssi: OwnedStream` is not satisfied",
        "the trait bound `dokime::host::System: Publishable` is not satisfied",
        "the trait bound `dokime::host::System: OwnedStream` is not satisfied",
    ] {
        assert!(diagnostics.contains(unsatisfied), "{diagnostics}");
    }
    assert!(diagnostics.contains("aborting due to 4 previous errors"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn undeclared_call_refused() {
    let (built, _) = compile("fixtures/undeclared_call.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The plugin serves the operation; the template never said it would call it,
    // so the refusal is on the call and names the catalog to add it to.
    assert!(diagnostics.contains("ProbeRequired: dokime::catalog::Contains<GetCurrent>"), "{diagnostics}");
    assert_eq!(
        diagnostics.matches("ProbeRequired: dokime::catalog::Contains<GetVoltage>").count(),
        0,
        "a declared request is not refused: {diagnostics}"
    );
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn mount_refuses_unserved() {
    let (built, _) = compile("fixtures/unserved_calls.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The template says which operations it calls and the plugin says which it
    // requires or serves privately; binding the suite compares them once, as a
    // constant read after monomorphization.
    assert!(
        diagnostics.contains("a template requires an operation its plugin neither names in `requires(...)` nor provides privately"),
        "{diagnostics}"
    );
    assert!(diagnostics.contains("TemplateBindings<Plugin>"), "{diagnostics}");
    assert!(diagnostics.contains("aborting due to 1 previous error"), "{diagnostics}");
}

#[test]
fn listed_grant_refused() {
    let (built, _) = compile_observer_catalogs("listed_streams", "streams(Sample),", "Sample", 10);
    let diagnostics = String::from_utf8_lossy(&built.stderr);

    assert!(!built.status.success(), "a hand-written grant compiled");
    assert!(diagnostics.contains("remove `streams(...)`"), "{diagnostics}");
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn conflicting_streams_refused() {
    let (built, _) = compile_observer_catalogs("conflicting_streams", "", "Conflicting", 10);
    let diagnostics = String::from_utf8_lossy(&built.stderr);

    assert!(!built.status.success(), "two shapes of one stream compiled");
    assert!(
        diagnostics.contains("two declarations of the plugin name one stream id with different rows, schema versions, windows or metadata"),
        "{diagnostics}"
    );
    assert!(!diagnostics.contains("proc macro panicked"), "{diagnostics}");
}

#[test]
fn shared_stream_once() {
    let (built, output) = compile_observer_catalogs("shared_observer_stream", "", "Sample", 20);
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));

    let declaration = sdk::harmos::plugin::inventory::Declaration::inspect(
        &std::fs::read(&output).unwrap(),
        sdk::harmos::plugin::inventory::Facility::Sidecar,
    )
    .expect("shared producers produce a valid artifact");
    assert_eq!(declaration.streams.iter().filter(|stream| stream.id == "shared.sample").count(), 1,);
    assert_eq!(declaration.observers.len(), 2);
    let mut command = sdk::domain::process::command(&output);
    let ran = command.output().unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(ran.status.success(), "{}", String::from_utf8_lossy(&ran.stderr));
}
