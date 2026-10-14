/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod dokime {
    #[sdk::message]
    #[derive(Clone, PartialEq)]
    pub struct Payload {
        pub text: String,
    }

    #[allow(dead_code)]
    #[sdk::message]
    #[derive(Clone, PartialEq)]
    pub struct Ignored {
        pub text: String,
    }

    pub fn reply(request: Payload) -> Payload {
        request
    }
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use sdk::harmos::operations::{
    OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable,
};
use sdk::harmos::plugin::{Attachment, Declared};
use sdk::harmos::sidecar::{ObserverAttachment, ObserverContext, PublishError, Publisher};

// Local Imports

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */
type Observing = ObserverAttachment<Resources, Publisher<Renamed>>;

/* === Structs === */
struct Renamed {
    resources: Arc<Resources>,
    observers: Option<Arc<Observing>>,
}

struct Resources;

#[sdk::row]
#[derive(Clone)]
pub struct Row {
    value: i64,
}

#[sdk::stream(id = "renamed.observations", window = 64, row = Row, label = "Observations")]
pub struct ObservationStream;

#[sdk::stream(id = "renamed.direct", window = 8, row = Row)]
pub struct DirectStream;

#[sdk::record(id = "renamed.observed", label = "Observed")]
pub struct Observed {
    value: i64,
}

#[derive(Default)]
struct Observations;

#[sdk::section(
    label = "Chamber",
    description = "One settings section, written the way an author writes one."
)]
struct Chamber {
    #[setting(label = "Port", default = "/dev/ttyUSB0")]
    port: String,
}

#[sdk::settings]
struct RenamedSettings {
    chamber: Chamber,
}

struct GetEcho;

struct GetStatus;

struct GetSpareEcho;

struct ObserverControl;

#[cfg(test)]
struct SdkArtifact {
    library: std::path::PathBuf,
    serde: std::path::PathBuf,
    native_paths: Vec<std::path::PathBuf>,
}

/* === Enums === */
#[sdk::catalog]
enum Implemented {
    GetEcho(GetEcho),
    GetStatus(GetStatus),
    GetSpareEcho(GetSpareEcho),
}

#[sdk::catalog]
enum Operations {
    #[nested]
    Implemented(Implemented),
    Observe(ObserverControl),
}

#[sdk::catalog]
enum Observers {
    Observations(Observations),
}

#[sdk::catalog]
pub enum Streams {
    Observation(ObservationStream),
    Direct(DirectStream),
}

#[sdk::catalog]
pub enum Records {
    Observed(Observed),
}

/* === Traits === */

/* === Implementations === */
#[sdk::operation(id = "renamed.get_echo", version = 1, description = "Echoes.", policy = POLICY)]
impl GetEcho {
    pub async fn initialize(_: Arc<Resources>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &sdk::Supervisor,
        request: &self::dokime::Payload,
    ) -> Result<self::dokime::Payload, sdk::Refusal> {
        Ok(self::dokime::reply(request.clone()))
    }
}

#[sdk::operation(id = "renamed.get_status", version = 1, description = "Reports.", policy = POLICY)]
impl GetStatus {
    pub async fn initialize(_: Arc<Resources>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &sdk::Supervisor,
        _: &(),
    ) -> Result<sdk::domains::StatusResponse, sdk::Refusal> {
        Ok(sdk::domains::StatusResponse::ready().field(
            "Echo",
            self::dokime::reply(self::dokime::Payload {
                text: "author syntax survives".to_owned(),
            })
            .text,
        ))
    }
}

#[sdk::operation(id = "renamed.get_spare_echo", version = 1, description = "Echoes too.", policy = POLICY)]
impl GetSpareEcho {
    pub async fn initialize(_: Arc<Resources>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &sdk::Supervisor,
        request: &self::dokime::Ignored,
    ) -> Result<self::dokime::Ignored, sdk::Refusal> {
        Ok(request.clone())
    }
}

impl sdk::harmos::operations::OperationDeclaration for ObserverControl {
    type Request = sdk::harmos::operations::observers::Control;
    type Response = ();
    const ID: &'static str = "renamed.observers.observe";
    const VISIBILITY: sdk::harmos::operations::Visibility =
        sdk::harmos::operations::Visibility::Host;
    const POLICY: OperationPolicy = sdk::harmos::operations::observers::POLICY;
}

impl sdk::harmos::operations::WireOperation for ObserverControl {
    const VERSION: u32 = 1;
}

#[sdk::observer(interval_ms = 10, streams(ObservationStream), activation = "demand")]
impl Observations {
    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Resources, Publisher<Renamed>>,
    ) -> Result<(), PublishError> {
        let text = self::dokime::reply(self::dokime::Payload {
            text: "one".to_owned(),
        });

        ctx.publish_on::<ObservationStream>(&Row {
            value: text.text.len() as i64,
        })
        .await
    }
}

impl sdk::harmos::operations::lifecycle::Policy for Renamed {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "renamed",
    provides = Operations,
    requires(),
    contributions(
        sdk::Records<Records, ()>,
        sdk::Evidence<Streams>,
        sdk::Settings<RenamedSettings>
    ),
    observers = Observers,
    observer_control = ObserverControl,
    restart = "never",
)]
impl sdk::Plugin for Renamed {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    type Resources = Resources;

    fn prepare(config: sdk::plugin::Configuration) -> Result<Self, sdk::Refusal> {
        let settings = sdk::plugin::refusal(config.settings::<RenamedSettings>())?;
        let _ = settings.chamber.port;

        Ok(Self {
            resources: Arc::new(Resources),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<Observers>(
            self.resources(),
            <Self as Declared>::STREAMS,
        )?);

        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable<sdk::Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Implemented, _>(self.resources())?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<ObserverControl>(table)
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
        self.observers.take();

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[cfg(not(test))]
fn main() {
    let _ = <Renamed as Declared>::markers();
}

#[cfg(test)]
fn sdk_artifact() -> &'static SdkArtifact {
    static SDK: std::sync::OnceLock<SdkArtifact> = std::sync::OnceLock::new();
    SDK.get_or_init(|| {
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../dokime/Cargo.toml")
            .canonicalize()
            .unwrap();
        let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/fixtures/macros-native-sdk");
        // Keep The Nested SDK Build In Its Own Checkout-Local Namespace.
        // Cargo JSON Identifies Exact Libraries And Native Link Paths There.
        let built =
            std::process::Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
                .args(["build", "--locked", "--manifest-path"])
                .arg(&manifest)
                .args([
                    "--package",
                    "dokime",
                    "--lib",
                    "--no-default-features",
                    // The fixtures are written with the attributes, which ship
                    // behind `derive` the way serde ships its own.
                    "--features",
                    "native,derive",
                    "--message-format=json",
                    "--target-dir",
                ])
                .arg(&target)
                .output()
                .unwrap();
        assert!(
            built.status.success(),
            "native SDK fixture build failed: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let mut artifacts = Vec::new();
        let mut serde = None;
        let mut native_paths = Vec::new();
        for line in String::from_utf8(built.stdout).unwrap().lines() {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };

            if value["reason"] == "build-script-executed" {
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

            if value["reason"] == "compiler-artifact" && value["target"]["name"] == "serde" {
                serde = value["filenames"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|name| name.as_str())
                    .find(|name| name.ends_with(".rlib"))
                    .map(std::path::PathBuf::from);
            }

            if value["reason"] != "compiler-artifact"
                || !value["manifest_path"]
                    .as_str()
                    .is_some_and(|path| same_path(&manifest, path))
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
                    .filter(|path| {
                        path.extension()
                            .is_some_and(|extension| extension == "rlib")
                    }),
            );
        }
        native_paths.sort_unstable();
        native_paths.dedup();
        assert_eq!(
            artifacts.len(),
            1,
            "Cargo must identify exactly one native SDK rlib"
        );
        let library = artifacts.pop().unwrap();
        let serde = serde.expect("Cargo must identify the SDK serde library");
        let target = target.canonicalize().unwrap();
        assert!(library.canonicalize().unwrap().starts_with(&target));
        assert!(serde.canonicalize().unwrap().starts_with(&target));
        SdkArtifact {
            library,
            serde,
            native_paths,
        }
    })
}

#[cfg(test)]
fn same_path(left: &std::path::Path, right: &str) -> bool {
    match (
        left.canonicalize(),
        std::path::Path::new(right).canonicalize(),
    ) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == std::path::Path::new(right),
    }
}

#[cfg(test)]
fn compile(source: &str) -> (std::process::Output, std::path::PathBuf) {
    let sdk = sdk_artifact();
    let dependencies = sdk.library.parent().unwrap().join("deps");
    assert!(dependencies.is_dir(), "Cargo dependency directory missing");
    let directory = dependencies.join(format!("inventory-artifacts-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join(std::path::Path::new(source).file_stem().unwrap());
    let mut rustc = std::process::Command::new("rustc");
    // These inspection fixtures have empty mains; retain their uncalled
    // sidecar declarations instead of letting MSVC discard the metadata.
    #[cfg(target_env = "msvc")]
    rustc.args(["-C", "link-arg=/OPT:NOREF"]);
    let built = rustc
        .arg("--edition=2024")
        .arg("--crate-type=bin")
        .arg(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join(source),
        )
        .arg("--extern")
        .arg(format!("sdk={}", sdk.library.display()))
        .arg("--extern")
        .arg(format!("serde={}", sdk.serde.display()))
        .arg("-L")
        .arg(format!("dependency={}", dependencies.display()))
        .args(
            sdk.native_paths
                .iter()
                .flat_map(|path| ["-L".to_owned(), format!("native={}", path.display())]),
        )
        .arg("-o")
        .arg(&output)
        .output()
        .unwrap();
    (built, output)
}

#[cfg(test)]
fn compile_observer_catalogs(
    name: &str,
    arguments: &str,
    peer_stream: &str,
    peer_interval: u64,
) -> (std::process::Output, std::path::PathBuf) {
    let source = r#"
extern crate sdk;

use std::sync::Arc;

use sdk::harmos::plugin::{Attachment, Declared};
use sdk::harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable};
use sdk::harmos::sidecar::{ObserverAttachment, ObserverContext, PublishError, Publisher};

const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
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
enum Operations { Observe(ObserverControl) }

type Observing = ObserverAttachment<(), Publisher<Shared>>;

struct Shared { observers: Option<Arc<Observing>> }
struct ObserverControl;

impl sdk::harmos::operations::OperationDeclaration for ObserverControl {
    type Request = sdk::harmos::operations::observers::Control;
    type Response = ();
    const ID: &'static str = "shared.observers.observe";
    const VISIBILITY: sdk::harmos::operations::Visibility = sdk::harmos::operations::Visibility::Host;
    const POLICY: OperationPolicy = sdk::harmos::operations::observers::POLICY;
}
impl sdk::harmos::operations::WireOperation for ObserverControl {
    const VERSION: u32 = 1;
}

impl sdk::harmos::operations::lifecycle::Policy for Shared {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "shared",
    provides = Operations,
    requires(),
    PLUGIN_ARGUMENTS
    observers = Observers,
    observer_control = ObserverControl,
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

    fn bind_operations(&self, table: &mut WireTable<sdk::Supervisor>) -> Result<(), ServingError> {
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<ObserverControl>(table)
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
    let directory = sdk_artifact()
        .library
        .parent()
        .unwrap()
        .join("deps")
        .join(format!("inventory-artifacts-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join(format!("{name}.rs"));
    std::fs::write(&source_path, source).unwrap();

    let result = compile(source_path.to_str().unwrap());
    std::fs::remove_file(source_path).unwrap();
    result
}

#[cfg(test)]
fn artifact(source: &str) -> Vec<u8> {
    let (built, output) = compile(source);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let bytes = std::fs::read(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    bytes
}

#[cfg(test)]
fn operation_ids(
    nodes: &[sdk::catalog::Node<sdk::harmos::operations::WireDeclaration>],
) -> Vec<&'static str> {
    nodes
        .iter()
        .flat_map(|node| match node {
            sdk::catalog::Node::Leaf(declaration) => vec![declaration.operation.id],
            sdk::catalog::Node::Nested(nested) => operation_ids(nested),
        })
        .collect()
}

#[cfg(test)]
fn declaration_of(source: &str) -> sdk::harmos::plugin::inventory::Declaration {
    sdk::harmos::plugin::inventory::Declaration::inspect(
        &artifact(source),
        sdk::harmos::plugin::inventory::Facility::Sidecar,
    )
    .unwrap()
}

/* === Tests === */

#[cfg(test)]
pub(crate) fn alias_compiles_operations() {
    // Renamed Facade Uses One Shared Codec For Every Domain Shape.
    let record = Observed { value: 19 };
    let encoded = sdk::Message::encode_to_vec(&record);
    let recovered = <Observed as sdk::Message>::decode(&encoded).unwrap();
    assert_eq!(recovered.value, 19);
    assert_eq!(
        <Observed as sdk::records::DeclaredRecord>::RECORD_TYPE,
        "renamed.observed"
    );
    let row = Row { value: 23 };
    let encoded = sdk::Message::encode_to_vec(&row);
    assert_eq!(<Row as sdk::Message>::decode(&encoded).unwrap().value, 23);
    assert_eq!(Row::column_value().field(), "value");
    assert_eq!(<Renamed as Declared>::ID, "renamed");
    let mut served = operation_ids(<Renamed as Declared>::PROVIDES);
    served.sort_unstable();
    assert_eq!(
        served,
        [
            "renamed.get_echo",
            "renamed.get_spare_echo",
            "renamed.get_status",
            "renamed.observers.observe",
        ]
    );
    let streams = <Renamed as Declared>::STREAMS
        .iter()
        .map(|stream| stream.id)
        .collect::<Vec<_>>();
    assert_eq!(streams, ["renamed.observations", "renamed.direct"]);
    assert_eq!(<Renamed as Declared>::OBSERVERS.len(), 1);
    let namespaces = <Renamed as Declared>::CONTRIBUTIONS
        .iter()
        .map(|section| section.namespace.as_ref())
        .collect::<Vec<_>>();
    assert!(namespaces.contains(&"dokime.settings"), "{namespaces:?}");
}

#[cfg(test)]
fn alias_inventory_contains_only_declared_operations() {
    let declaration = declaration_of("template/renamed_sdk.rs");
    assert_eq!(declaration.id, "renamed");
    assert_eq!(
        declaration
            .streams
            .iter()
            .map(|row| (row.id.as_str(), row.schema, row.window))
            .collect::<Vec<_>>(),
        [("renamed.direct", 1, 8), ("renamed.observations", 1, 64)],
        "outputs appear once even when referenced by an observer",
    );
    assert_eq!(
        declaration
            .operations
            .iter()
            .map(|operation| operation.id.as_str())
            .collect::<Vec<_>>(),
        [
            // Authored operations follow their selected catalog order, and the
            // SDK adds none of its own to a plugin that lists no suites.
            "renamed.get_echo",
            "renamed.get_status",
            "renamed.get_spare_echo",
            "renamed.observers.observe",
        ]
    );
    assert_eq!(declaration.observers.len(), 1);
    assert_eq!(declaration.observers[0].streams, ["renamed.observations"]);
    let namespaces = declaration
        .sections
        .iter()
        .map(|section| section.namespace.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        namespaces,
        ["dokime.records", "dokime.telemetry", "dokime.settings"]
    );
}

#[cfg(test)]
fn a_plugin_with_nothing_declares_only_its_identity() {
    let declaration = declaration_of("fixtures/routeless.rs");
    assert_eq!(declaration.id, "routeless");
    assert!(declaration.operations.is_empty(), "no implicit operations");
    assert!(declaration.streams.is_empty());
    assert!(declaration.observers.is_empty());
    assert!(declaration.sections.is_empty());
}

#[cfg(test)]
fn plugin_identity_that_could_not_travel_is_refused() {
    let (built, _) = compile("fixtures/invalid_identity.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The plugin's identity is checked on the line it was written, because it is
    // the one every operation and stream it serves is scoped under.
    assert_eq!(
        diagnostics
            .matches("an artifact id is lowercase ASCII")
            .count(),
        1,
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("aborting due to 1 previous error"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn template_id_that_names_no_suite_is_refused() {
    let (built, _) = compile("fixtures/template_identity.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // An omitted id cannot be derived, because the item name never spells the
    // suite; a written literal is read on its own line.
    assert!(
        diagnostics
            .contains("`id = …` is required: a template is referenced as `<suite id>.<segment>`"),
        "{diagnostics}"
    );
    for refused in [
        "`checks.Probe` is not a template id",
        "`probe` is not a template id",
    ] {
        assert!(diagnostics.contains(refused), "{diagnostics}");
    }
    assert!(
        diagnostics.contains("aborting due to 3 previous errors"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn suite_refuses_foreign_and_repeated_template_ids() {
    let (built, _) = compile("fixtures/suite_templates.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The suite owns the prefix, so it is the suite that refuses a template id
    // outside it and two templates answering to one reference.
    assert!(
        diagnostics.contains("a template id is its suite's id followed by `.` and one segment"),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("two templates of one suite declare the same id"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn operations_under_one_id_are_refused_when_inspected() {
    let (built, output) = compile("fixtures/duplicate_operations.rs");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let bytes = std::fs::read(&output).unwrap();
    std::fs::remove_file(output).unwrap();
    let refused = sdk::harmos::plugin::inventory::Declaration::inspect(
        &bytes,
        sdk::harmos::plugin::inventory::Facility::Sidecar,
    )
    .expect_err("two operations under one id are one operation too many");
    assert!(
        refused
            .to_string()
            .contains("duplicate ordinary operation `duplicated.read`"),
        "{refused}"
    );
}

#[cfg(test)]
fn a_stream_declares_the_shape_its_form_calls_for() {
    let (built, _) = compile("fixtures/stream_shapes.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    assert!(
        diagnostics.contains("a stream requires named row fields"),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("a foreign stream declares the columns this plugin reads"),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("a named stream with `row = Shape` must be a unit struct"),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("aborting due to 3 previous errors"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn a_stream_this_plugin_does_not_own_is_never_published() {
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
    assert!(
        diagnostics.contains("aborting due to 4 previous errors"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn call_outside_a_template_s_requires_is_refused() {
    let (built, _) = compile("fixtures/undeclared_call.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The plugin serves the operation; the template never said it would call it,
    // so the refusal is on the call and names the catalog to add it to.
    assert!(
        diagnostics.contains("ProbeRequired: dokime::catalog::Contains<GetCurrent>"),
        "{diagnostics}"
    );
    assert_eq!(
        diagnostics
            .matches("ProbeRequired: dokime::catalog::Contains<GetVoltage>")
            .count(),
        0,
        "a declared request is not refused: {diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn mount_refuses_unserved_template_operations() {
    let (built, _) = compile("fixtures/unserved_calls.rs");
    assert!(!built.status.success());
    let diagnostics = String::from_utf8_lossy(&built.stderr);
    // The template says which operations it calls and the plugin says which it
    // requires or serves privately; binding the suite compares them once, as a
    // constant read after monomorphization.
    assert!(
        diagnostics.contains(
            "a template requires an operation its plugin neither names in `requires(...)` nor provides privately"
        ),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("TemplateBindings<Plugin>"),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("aborting due to 1 previous error"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn a_plugin_refuses_a_hand_written_stream_grant() {
    let (built, _) = compile_observer_catalogs("listed_streams", "streams(Sample),", "Sample", 10);
    let diagnostics = String::from_utf8_lossy(&built.stderr);

    assert!(!built.status.success(), "a hand-written grant compiled");
    assert!(
        diagnostics.contains("remove `streams(...)`"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn declarations_disagreeing_about_one_stream_are_refused() {
    let (built, _) = compile_observer_catalogs("conflicting_streams", "", "Conflicting", 10);
    let diagnostics = String::from_utf8_lossy(&built.stderr);

    assert!(!built.status.success(), "two shapes of one stream compiled");
    assert!(
        diagnostics.contains(
            "two declarations of the plugin name one stream id with different rows, schema versions, windows or metadata"
        ),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("proc macro panicked"),
        "{diagnostics}"
    );
}

#[cfg(test)]
fn observers_sharing_a_stream_register_its_output_once() {
    let (built, output) = compile_observer_catalogs("shared_observer_stream", "", "Sample", 20);
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );

    let declaration = sdk::harmos::plugin::inventory::Declaration::inspect(
        &std::fs::read(&output).unwrap(),
        sdk::harmos::plugin::inventory::Facility::Sidecar,
    )
    .expect("shared producers produce a valid artifact");
    assert_eq!(
        declaration
            .streams
            .iter()
            .filter(|stream| stream.id == "shared.sample")
            .count(),
        1,
    );
    assert_eq!(declaration.observers.len(), 2);
    let ran = std::process::Command::new(&output).output().unwrap();
    std::fs::remove_file(output).unwrap();
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
}

#[cfg(test)]
pub(crate) fn compiled_authoring_contracts() {
    eprintln!("compile contract: alias_inventory_contains_only_declared_operations");
    alias_inventory_contains_only_declared_operations();

    eprintln!("compile contract: a_plugin_with_nothing_declares_only_its_identity");
    a_plugin_with_nothing_declares_only_its_identity();

    eprintln!("compile contract: plugin_identity_that_could_not_travel_is_refused");
    plugin_identity_that_could_not_travel_is_refused();

    eprintln!("compile contract: template_id_that_names_no_suite_is_refused");
    template_id_that_names_no_suite_is_refused();

    eprintln!("compile contract: suite_refuses_foreign_and_repeated_template_ids");
    suite_refuses_foreign_and_repeated_template_ids();

    eprintln!("compile contract: operations_under_one_id_are_refused_when_inspected");
    operations_under_one_id_are_refused_when_inspected();

    eprintln!("compile contract: a_stream_declares_the_shape_its_form_calls_for");
    a_stream_declares_the_shape_its_form_calls_for();

    eprintln!("compile contract: a_stream_this_plugin_does_not_own_is_never_published");
    a_stream_this_plugin_does_not_own_is_never_published();

    eprintln!("compile contract: call_outside_a_template_s_requires_is_refused");
    call_outside_a_template_s_requires_is_refused();

    eprintln!("compile contract: mount_refuses_unserved_template_operations");
    mount_refuses_unserved_template_operations();

    eprintln!("compile contract: a_plugin_refuses_a_hand_written_stream_grant");
    a_plugin_refuses_a_hand_written_stream_grant();

    eprintln!("compile contract: declarations_disagreeing_about_one_stream_are_refused");
    declarations_disagreeing_about_one_stream_are_refused();

    eprintln!("compile contract: observers_sharing_a_stream_register_its_output_once");
    observers_sharing_a_stream_register_its_output_once();
}
