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
use std::sync::Arc;
// Third Party Imports

// Local Imports
use dokime::core::Value;
use dokime::domains::OptionProviderOption;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy, ServingError, WireTable};
use dokime::plugin::settings::{Discovered, PluginSettings, SettingsSection};
use dokime::plugin::{Configuration, Secret};
use dokime::prelude::{OptionsResponse, Refusal, StatusResponse, Supervisor};
use dokime::settings::{SettingValues, SettingsEntry};

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Bench;

#[dokime::settings]
pub struct BenchSettings {
    pub workspace: Workspace,
    pub credentials: Credentials,
}

#[dokime::section(label = "Workspace", status = GetStatus, description = "The section the card shows first, with the probe it reads beside it.")]
pub struct Workspace {
    #[setting(label = "Workspace ID", description = "Bound For Sync.")]
    pub workspace_id: Option<String>,

    #[setting(label = "Environment", options = ListEnvironments)]
    pub environment: Environment,

    #[setting(default = 500, min = 1, max = 5_000, step = 1)]
    pub page_limit: u32,

    #[setting(label = "Enable Uploads", default = true)]
    pub upload_enabled: bool,

    #[setting(label = "Port", discovered = DetectPort)]
    pub port: Option<String>,

    #[setting(label = "Stream", discovered = DetectStream, description = "A discovered number, which is what says absence is a fact the host reports rather than one a value can be read off: the resolved values always carry a declared `0` here, so nothing about this field's value could tell a run whether anybody filled it in.")]
    pub stream: Option<u32>,

    #[setting(label = "Archive", visible_when(upload_enabled = true))]
    pub archive: Option<std::path::PathBuf>,
}

#[dokime::section(
    label = "Credentials",
    description = "The section nothing but a binding fills in."
)]
pub struct Credentials {
    #[setting(label = "Bearer Token", from_environment = "BENCH_TOKEN")]
    pub token: Option<Secret>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct GetVoltageRequest {
    pub stream: u32,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct GetVoltageResponse {
    pub volts: f64,
}

/* === Enums === */
#[dokime::choice]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Environment {
    #[default]
    Development,
    Test,
    Production,
}

/* === Traits === */

/* === Implementations === */
impl dokime::harmos::operations::lifecycle::Policy for Bench {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime::catalog]
enum Operations {
    GetStatus(GetStatus),
    ListEnvironments(ListEnvironments),
    DetectPort(DetectPort),
    DetectStream(DetectStream),
    GetVoltage(GetVoltage),
}

#[dokime::plugin(
    id = "bench",
    provides = Operations,
    requires(),
    contributions(dokime::Settings<BenchSettings>),
    restart = "never",
)]
impl dokime::Plugin for Bench {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<BenchSettings>())?;
        assert_eq!(settings.workspace.page_limit, 500);

        Ok(Self)
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Operations, _>(self.resources())
    }

    async fn shutdown(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

struct GetStatus;

#[dokime::operation(id = "bench.get_status", version = 1, description = "Reports.", policy = POLICY)]
impl GetStatus {
    fn new(_: Arc<()>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<StatusResponse, Refusal> {
        Ok(StatusResponse::ready())
    }
}

struct ListEnvironments;

#[dokime::operation(id = "bench.list_environments", version = 1, description = "Offers.", policy = POLICY)]
impl ListEnvironments {
    fn new(_: Arc<()>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<OptionsResponse, Refusal> {
        Ok(OptionsResponse::of([OptionProviderOption {
            value: "development".to_owned(),
            label: "Offered".to_owned(),
            description: None,
            disabled: false,
            metadata: None,
        }]))
    }
}

struct DetectPort;

#[dokime::operation(id = "bench.detect_port", version = 1, description = "Finds.", policy = POLICY)]
impl DetectPort {
    fn new(_: Arc<()>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Discovered, Refusal> {
        Ok(Discovered::of("/dev/bench0"))
    }
}

struct DetectStream;

#[dokime::operation(id = "bench.detect_stream", version = 1, description = "Finds.", policy = POLICY)]
impl DetectStream {
    fn new(_: Arc<()>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<Discovered, Refusal> {
        Ok(Discovered::of(7_i64))
    }
}

struct GetVoltage;

#[dokime::operation(id = "bench.get_voltage", version = 1, description = "Reads.", policy = POLICY)]
impl GetVoltage {
    fn new(_: Arc<()>) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &Supervisor,
        request: &GetVoltageRequest,
    ) -> Result<GetVoltageResponse, Refusal> {
        Ok(GetVoltageResponse {
            volts: f64::from(request.stream),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn defaults() -> SettingValues {
    BenchSettings::catalog()
        .resolve(&SettingValues::new())
        .expect("the declared catalog resolves its own defaults")
}

/* === Tests === */

pub(crate) fn section_folds_shows() {
    let catalog = BenchSettings::catalog();
    let declared = catalog
        .contributions
        .iter()
        .map(SettingsEntry::id)
        .collect::<Vec<_>>();

    assert_eq!(
        declared,
        [
            "workspace_id",
            "environment",
            "page_limit",
            "upload_enabled",
            "port",
            "stream",
            "archive",
            "token",
        ]
    );
    catalog.validate().expect("the declared catalog validates");
}

pub(crate) fn field_type_required() {
    let catalog = BenchSettings::catalog();
    let entry = |id: &str| catalog.get(id).expect("the field was declared").clone();

    assert!(
        matches!(entry("workspace_id"), SettingsEntry::TextArea(field)
        if field.title == "Workspace ID"
            && field.description.as_deref() == Some("Bound For Sync.")
            && !field.required)
    );
    assert!(matches!(entry("upload_enabled"), SettingsEntry::Boolean(field) if field.default));
    assert!(matches!(entry("archive"), SettingsEntry::Path(field) if !field.required));
    assert!(
        matches!(entry("token"), SettingsEntry::SecretReference(field)
        if field.default
            == Some(dokime::settings::SecretReference::Environment {
                variable: "BENCH_TOKEN".to_owned(),
            }))
    );
    // A field with no label of its own is read by its own name, titled.
    assert!(matches!(entry("page_limit"), SettingsEntry::Number(field)
        if field.title == "Page Limit"
            && field.default == 500.0
            && field.min == Some(1.0)
            && field.max == Some(5_000.0)
            && field.step == Some(1.0)));
    // A closed set declares its own values, and the dropdown route that fills
    // it at run time is named beside them.
    assert!(
        matches!(entry("environment"), SettingsEntry::MultipleChoice(field)
        if field.default == "development"
            && field.choices.len() == 3
            && field.provider.as_deref() == Some("bench.list_environments"))
    );
}

pub(crate) fn section_links_default() {
    assert_eq!(
        <Workspace as SettingsSection>::status_operation(),
        Some("bench.get_status")
    );
    assert_eq!(<Credentials as SettingsSection>::status_operation(), None);
    assert_eq!(<Workspace as SettingsSection>::LABEL, "Workspace");
    let discovered = <BenchSettings as PluginSettings>::discoveries();
    assert_eq!(
        discovered
            .iter()
            .map(|discovery| (discovery.field.as_str(), discovery.operation.as_str()))
            .collect::<Vec<_>>(),
        [
            ("port", "bench.detect_port"),
            ("stream", "bench.detect_stream")
        ]
    );
}

pub(crate) fn admitted_values_handed() {
    let mut saved = defaults();
    saved.insert("workspace_id".to_owned(), Value::from("bench-workspace"));
    saved.insert("environment".to_owned(), Value::from("production"));
    saved.insert("page_limit".to_owned(), Value::from(1_000));
    saved.insert("archive".to_owned(), Value::from("/var/bench"));

    let configuration = Configuration {
        settings: saved.clone(),
        ..Configuration::default()
    };
    let settings = BenchSettings::decode(&saved, &configuration).expect("the saved values decode");

    assert_eq!(
        settings.workspace.workspace_id.as_deref(),
        Some("bench-workspace")
    );
    assert_eq!(settings.workspace.environment, Environment::Production);
    assert_eq!(settings.workspace.page_limit, 1_000);
    assert!(settings.workspace.upload_enabled);
    assert_eq!(settings.workspace.port, None);
    assert_eq!(settings.workspace.stream, Some(0));
    assert_eq!(
        settings.workspace.archive,
        Some(std::path::PathBuf::from("/var/bench"))
    );
    // A secret never travels in the values: it is read out of the binding the
    // host admitted, and an unbound one is absent rather than refused.
    assert_eq!(settings.credentials.token, None);
}

pub(crate) fn value_outside_refused() {
    let mut saved = defaults();
    saved.insert("page_limit".to_owned(), Value::from(10_000));

    let refusal = BenchSettings::catalog()
        .resolve(&saved)
        .expect_err("a page limit above the declared maximum is refused")
        .to_string();

    assert!(refusal.contains("page_limit"), "{refusal}");
}

pub(crate) fn settings_links_ids() {
    // Settings links use the IDs authored by the selected operation contracts.
    assert_eq!(BenchSettings::status_operation(), Some("bench.get_status"));
    assert_eq!(
        BenchSettings::option_providers()
            .iter()
            .map(|offered| offered.metadata.identifier.to_string())
            .collect::<Vec<_>>(),
        vec!["bench.list_environments".to_owned()]
    );
}

pub(crate) fn saved_values_empty() {
    // Nobody saved a port, so the field is absent: discovery is the host's to
    // run and a discovered value is a suggestion, never a setting the plugin
    // opens with.
    let opened =
        <Bench as dokime::Plugin>::prepare(Configuration::declared::<BenchSettings>()).map(|_| ());
    assert!(opened.is_ok(), "the bench opens on its declared defaults");
    let declared = Configuration::declared::<BenchSettings>()
        .settings::<BenchSettings>()
        .expect("the declared defaults decode");
    assert_eq!(declared.workspace.port, None);
    assert_eq!(declared.workspace.stream, Some(0));

    // A value an operator saved is theirs: it decodes as saved, and the
    // configuration says it was filled in, which is what the host reads to
    // decide that discovery must not overrule it.
    let mut saved = defaults();
    saved.insert("port".to_owned(), Value::from("/dev/saved"));
    saved.insert("stream".to_owned(), Value::from(3_u32));
    let mut configuration = Configuration {
        settings: saved,
        ..Configuration::default()
    };
    configuration.saved.insert("port".to_owned());
    configuration.saved.insert("stream".to_owned());
    let settings = configuration
        .settings::<BenchSettings>()
        .expect("the saved values decode");
    assert_eq!(settings.workspace.port.as_deref(), Some("/dev/saved"));
    assert_eq!(settings.workspace.stream, Some(3));
    assert!(configuration.is_saved("port") && configuration.is_saved("stream"));
    assert!(!Configuration::declared::<BenchSettings>().is_saved("port"));
}
