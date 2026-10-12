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
use std::collections::BTreeMap;
use std::sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
// Third Party Imports
use dokime::harmos::artifact::plugin::Declared;
use dokime::harmos::operations::{ServingError, WireTable};
// Local Imports
use dokime::execution::{
    Caller, CleanupContext, InitializationContext, Run, StepContext, Workflow,
};
use dokime::plugin::Configuration;
use dokime::plugin::testing::{Harness, HarnessError};
use dokime::{Refusal, Supervisor};

/* === Constants === */
const SUITE: &str = "resources.checks";
const TEMPLATE: &str = "resources.checks.probe";
const BOUND: Duration = Duration::from_secs(5);

const WATCHED_FIRST_BYTES: usize = 128 * 1024;
const WATCHED_CONTENT: &str = "the chamber log, rewritten in place\n";

const WATCH_SETTLE: Duration = Duration::from_millis(500);

static DOUBLES: LazyLock<Mutex<BTreeMap<String, Arc<Observations>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct Observations {
    events: Mutex<Vec<&'static str>>,
    run_numbers: Mutex<Vec<usize>>,
    started: tokio::sync::Notify,
    keys: Mutex<Vec<Option<String>>>,
}

struct Device {
    observations: Arc<Observations>,
    runs: Mutex<usize>,
    token: String,
}

struct PartialInitialization(Arc<Observations>);

#[dokime::message]
#[derive(Clone, PartialEq)]
struct GetDeviceResponse {
    #[row(description = "The device's own address, which two instances never share.")]
    identity: u64,
    #[row(description = "The token its observations are registered under.")]
    token: String,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
struct StartRunResponse {
    run: u64,
}

#[dokime::section(
    label = "Observations",
    description = "The token a run names its registered [`Observations`] double by, standing in for the live handle a setting cannot carry across the wire."
)]
pub struct Double {
    #[setting(
        label = "Observations Token",
        default = "unregistered",
        description = "Never read as its declared default: every run in this file names a real registered token, but a required text setting still has to declare a nonempty one to pass catalog validation."
    )]
    pub token: String,
}

#[dokime::settings]
pub struct OwnerSettings {
    pub double: Double,
}

struct Owner {
    device: Arc<Device>,
}

struct GetDevice(Arc<Device>);

struct StartRun(Arc<Device>);

struct Checks;

struct Probe {
    observations: Arc<Observations>,
    identity: u64,
}

#[dokime::parameters]
struct ProbeParameters {
    #[parameter(label = "Wait for cancellation", default = false)]
    wait: bool,
    #[parameter(label = "Refuse initialization", default = false)]
    initialize_fail: bool,
    #[parameter(label = "Wait during initialization", default = false)]
    initialize_wait: bool,
    #[parameter(label = "Write into a watched directory", default = false)]
    watch_directory: bool,
}

struct CatalogOnly;

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("execution received a different resource handle")]
    ResourceChanged,
    #[error("initialization refused after acquiring a resource")]
    InitializationRefused,
    #[error("the execution could not write its evidence")]
    Evidence(#[from] std::io::Error),
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime::Error),
}

#[dokime::catalog]
enum Operations {
    GetDevice(GetDevice),
    StartRun(StartRun),
}

#[dokime::catalog]
enum OwnerSuites {
    Checks(Checks),
}

#[dokime::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[dokime::catalog]
enum ProbeRequired {
    GetDevice(GetDevice),
    StartRun(StartRun),
}

/* === Traits === */

/* === Implementations === */
impl Observations {
    fn record(&self, event: &'static str) {
        self.events.lock().unwrap().push(event);
    }

    fn events(&self) -> Vec<&'static str> {
        self.events.lock().unwrap().clone()
    }
}

impl Device {
    fn open(token: &str) -> Arc<Self> {
        Arc::new(Self {
            observations: double(token),
            runs: Mutex::new(0),
            token: token.to_owned(),
        })
    }

    fn describe(self: &Arc<Self>) -> GetDeviceResponse {
        GetDeviceResponse {
            identity: Arc::as_ptr(self) as u64,
            token: self.token.clone(),
        }
    }

    fn start_run(&self) -> StartRunResponse {
        let run = {
            let mut runs = self.runs.lock().unwrap();
            *runs += 1;
            *runs
        };
        self.observations.run_numbers.lock().unwrap().push(run);
        self.observations.record("template initialized");

        StartRunResponse { run: run as u64 }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.observations.record("device dropped");
    }
}

impl Drop for PartialInitialization {
    fn drop(&mut self) {
        self.0.record("partial initialization dropped");
    }
}

#[dokime::operation(id = "resource-owner.get_device", version = 1, policy = super::native::POLICY)]
impl GetDevice {
    fn new(device: Arc<Device>) -> Self {
        Self(device)
    }

    async fn run(&mut self, ctx: &Supervisor, _: &()) -> Result<GetDeviceResponse, Refusal> {
        let key = dokime::execution::ExecutionScope::serving::<Self>(ctx)
            .map(|scope| scope.key().into_inner());
        self.0.observations.keys.lock().unwrap().push(key);
        Ok(self.0.describe())
    }
}

#[dokime::operation(id = "resource-owner.start_run", version = 1, policy = super::native::POLICY)]
impl StartRun {
    fn new(device: Arc<Device>) -> Self {
        Self(device)
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<StartRunResponse, Refusal> {
        Ok(self.0.start_run())
    }
}

impl dokime::harmos::operations::lifecycle::Policy for Owner {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime::plugin(
    id = "resource-owner",
    provides = Operations,
    requires(),
    contributions(dokime::Suites<OwnerSuites>, dokime::Settings<OwnerSettings>),
    restart = "never", description = "Typed resource lifecycle.")]
impl dokime::Plugin for Owner {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Device;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<OwnerSettings>())?;
        let device = Device::open(&settings.double.token);
        device.observations.record("plugin initialized");
        Ok(Self { device })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<Device> {
        Arc::clone(&self.device)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Operations, _>(self.resources())?;
        dokime::execution::serve::<Self, OwnerSuites>(table)
    }

    async fn shutdown(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.device.observations.record("plugin finalized");
        Ok(())
    }
}

#[dokime::suite(id = SUITE, label = "Resources", templates = ChecksTemplates)]
impl Checks {}

#[dokime::template(
    id = TEMPLATE,
    label = "Probe",
    requires = ProbeRequired,
    parameters = ProbeParameters)]
impl Probe {
    async fn initialize(
        parameters: &ProbeParameters,
        ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, Error> {
        let device = ctx.call::<GetDevice>(()).await?;
        let observations = double(&device.token);
        if parameters.initialize_fail {
            let _partial = PartialInitialization(observations);
            return Err(Error::InitializationRefused);
        }
        if parameters.initialize_wait {
            let partial = PartialInitialization(Arc::clone(&observations));
            observations.record("initialization waiting");
            observations.started.notify_one();
            std::future::pending::<()>().await;
            drop(partial);
        }
        ctx.call::<StartRun>(()).await?;

        Ok(Self {
            observations,
            identity: device.identity,
        })
    }

    #[dokime::step(label = "Use device")]
    async fn use_device(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        if ctx.call::<GetDevice>(()).await?.identity != self.identity {
            return Err(Error::ResourceChanged);
        }
        self.observations.record("step");
        self.observations.started.notify_one();
        if ctx.parameters().wait {
            std::future::pending::<()>().await;
        }
        Ok(())
    }

    #[dokime::step(
        label = "Write evidence",
        description = "Writes evidence the way a producer that never heard of Dokime does, and then rewrites it — which is the case the watch has to notice."
    )]
    async fn write_evidence(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        if !ctx.parameters().watch_directory {
            return Ok(());
        }
        let capture = ctx.artifacts().directory("capture")?;
        std::fs::write(capture.join("chamber.log"), vec![b'.'; WATCHED_FIRST_BYTES])?;
        tokio::time::sleep(WATCH_SETTLE).await;
        std::fs::write(capture.join("chamber.log"), WATCHED_CONTENT)?;

        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        if ctx.call::<GetDevice>(()).await?.identity != self.identity {
            return Err(Error::ResourceChanged);
        }
        tokio::task::yield_now().await;
        self.observations.record("cleanup finished");
        Ok(())
    }
}

impl dokime::harmos::operations::lifecycle::Policy for CatalogOnly {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime::plugin(
    id = "catalog-only",
    provides = (),
    requires(),
    contributions(dokime::Suites<OwnerSuites>),
    restart = "never", description = "Static resource catalog.")]
impl dokime::Plugin for CatalogOnly {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(_: Configuration) -> Result<Self, Refusal> {
        panic!("catalog export must not prepare a plugin")
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        panic!("catalog export must not initialize a plugin")
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, _: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        Ok(())
    }

    async fn shutdown(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn registered(observations: &Arc<Observations>) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let token = format!("observations-{}", NEXT.fetch_add(1, Ordering::SeqCst));
    DOUBLES
        .lock()
        .expect("the double registry is never poisoned")
        .insert(token.clone(), Arc::clone(observations));

    token
}

fn double(token: &str) -> Arc<Observations> {
    Arc::clone(
        DOUBLES
            .lock()
            .expect("the double registry is never poisoned")
            .get(token)
            .expect("a run names a double this file registered"),
    )
}

fn resolved(
    wait: bool,
    initialize_fail: bool,
    initialize_wait: bool,
    watch_directory: bool,
) -> String {
    serde_json::json!({
        "wait": dokime::Value::Bool(wait),
        "initialize_fail": dokime::Value::Bool(initialize_fail),
        "initialize_wait": dokime::Value::Bool(initialize_wait),
        "watch_directory": dokime::Value::Bool(watch_directory),
    })
    .to_string()
}

fn start(token: &str, parameters_json: String) -> dokime::transport::RunExecutionRequest {
    super::native::run_request(token, SUITE, TEMPLATE, parameters_json)
}

fn answered(device: &Arc<Device>) -> Caller {
    let described = Arc::clone(device);
    let counted = Arc::clone(device);
    Caller::local(BTreeMap::from([
        super::native::answering::<GetDevice, _>(move |()| described.describe()),
        super::native::answering::<StartRun, _>(move |()| counted.start_run()),
    ]))
}

fn harnessed(device: &Arc<Device>) -> Harness {
    let described = Arc::clone(device);
    let counted = Arc::clone(device);
    Harness::template(TEMPLATE)
        .answering::<GetDevice, _>(move |()| described.describe())
        .answering::<StartRun, _>(move |()| counted.start_run())
}

async fn opened(
    observations: &Arc<Observations>,
) -> super::native::Native<dokime::harmos::sidecar::PluginNative<Owner>> {
    let configuration = Configuration {
        settings: BTreeMap::from([(
            "token".to_owned(),
            dokime::Value::Text(registered(observations)),
        )]),
        ..Configuration::default()
    };
    super::native::Native::initialize(
        super::native::attach::<Owner>(),
        <Owner as Declared>::VERSION,
        serde_json::to_vec(&configuration).unwrap(),
        |_, _| {},
        vec![],
    )
    .await
}

/* === Tests === */

pub(crate) async fn shared_within_between() {
    let first = Arc::new(Observations::default());
    let second = Arc::new(Observations::default());
    let a = opened(&first).await;
    let b = opened(&second).await;
    for token in ["one", "two"] {
        for native in [&a, &b] {
            let outcome = tokio::time::timeout(
                BOUND,
                super::native::call_wire::<Run<Probe>>(
                    &native.runtime.operations,
                    start(token, resolved(false, false, false, false)),
                ),
            )
            .await
            .expect("the served run settles")
            .unwrap();
            assert!(outcome.passed, "{outcome:?}");
        }
    }
    for native in [&a, &b] {
        assert!(
            native
                .host
                .evidence()
                .iter()
                .any(|batch| batch.outcome.as_ref().is_some_and(|outcome| outcome.passed)),
            "each served run pushed its outcome to its own host"
        );
    }
    a.shutdown().await;
    assert!(!second.events().contains(&"plugin finalized"));
    b.shutdown().await;
    let expected = [
        "plugin initialized",
        "template initialized",
        "step",
        "cleanup finished",
        "template initialized",
        "step",
        "cleanup finished",
        "plugin finalized",
        "device dropped",
    ];
    assert_eq!(first.events(), expected);
    assert_eq!(second.events(), expected);
    assert_eq!(*first.run_numbers.lock().unwrap(), [1, 2]);
    assert_eq!(*second.run_numbers.lock().unwrap(), [1, 2]);
}

pub(crate) async fn execution_scope_calls() {
    let observations = Arc::new(Observations::default());
    let mut native = opened(&observations).await;
    let operations = &native.runtime.operations;
    let mut context = harmos::operations::CallContext::default();
    context
        .insert_portable(dokime::execution::ExecutionScope::execution("keyed"))
        .unwrap();
    let answer = tokio::time::timeout(
        BOUND,
        operations.with_context(context).call_encoded(
            <Run<Probe> as harmos::operations::OperationDeclaration>::ID,
            harmos::Message::encode_to_vec(&start("keyed", resolved(false, false, false, false))),
        ),
    )
    .await
    .expect("the served run settles")
    .unwrap();
    let outcome =
        <dokime::transport::SidecarExecutionOutcome as harmos::Message>::decode(answer.as_slice())
            .unwrap();
    assert!(outcome.passed, "{outcome:?}");
    super::native::call_wire::<GetDeviceDeclaration>(operations, ())
        .await
        .unwrap();
    native.retire().await;

    let keys = observations.keys.lock().unwrap().clone();
    let (last, run) = keys.split_last().unwrap();
    assert_eq!(*last, None, "a call outside any run carries no key");
    assert_eq!(run.len(), 3, "initialization, the step and cleanup called");
    for key in run {
        let key = key.as_deref().expect("a call inside a run carries a key");
        assert!(key.starts_with("keyed/"), "{key}");
        assert!(key.ends_with("/resource-owner.get_device"), "{key}");
    }
    native.shutdown().await;
}

pub(crate) async fn shutdown_joins_cleanup() {
    let observations = Arc::new(Observations::default());
    let device = Device::open(&registered(&observations));
    let workflow = Workflow::<Probe>::new(SUITE, "resource-owner");
    workflow
        .start_on(
            start("cancelled", resolved(true, false, false, false)),
            None,
            answered(&device),
        )
        .unwrap();
    tokio::time::timeout(BOUND, async {
        let mut cursor = 0;
        loop {
            tokio::select! {
                () = observations.started.notified() => break,
                () = tokio::time::sleep(Duration::from_millis(2)) => {
                    let batch = workflow.drain("cancelled", cursor).unwrap();
                    cursor = batch.cursor;
                    for encoded in &batch.step_begins {
                        let start: dokime::execution::StepAttemptStart =
                            serde_json::from_str(encoded).unwrap();
                        workflow.admit("cancelled", &dokime::transport::StepAdmission {
                            attempt_id: start.attempt_id.to_string(),
                            accepted: true,
                            message: String::new(),
                        });
                    }
                }
            }
        }
    })
    .await
    .expect("the step starts");
    workflow.interrupt();
    tokio::time::timeout(BOUND, workflow.shutdown())
        .await
        .expect("shutdown cancels the step and joins cleanup");
    assert_eq!(
        observations.events(),
        ["template initialized", "step", "cleanup finished"]
    );
}

pub(crate) async fn unfinished_initialize_handles() {
    for cancel in [false, true] {
        let observations = Arc::new(Observations::default());
        let device = Device::open(&registered(&observations));
        let workflow = Workflow::<Probe>::new(SUITE, "resource-owner");
        workflow
            .start_on(
                start("initializing", resolved(false, !cancel, cancel, false)),
                None,
                answered(&device),
            )
            .unwrap();
        if cancel {
            tokio::time::timeout(BOUND, observations.started.notified())
                .await
                .expect("initialization acquires its handle before cancellation");
            workflow.cancel("initializing");
        }
        let outcome = super::native::settled(&workflow, "initializing", 0).await;
        assert!(!outcome.passed);
        let mut expected = Vec::new();
        if cancel {
            expected.push("initialization waiting");
        }
        expected.push("partial initialization dropped");
        assert_eq!(observations.events(), expected);
        assert!(observations.run_numbers.lock().unwrap().is_empty());
        workflow.shutdown().await;
    }
}

pub(crate) async fn harness_finalizes() {
    let observations = Arc::new(Observations::default());
    let device = Device::open(&registered(&observations));
    let report = harnessed(&device)
        .run::<OwnerSuites>()
        .await
        .expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());
    assert!(
        report.criteria().is_empty(),
        "this template declares no criterion, so the run decides none"
    );
    assert_eq!(
        observations.events(),
        ["template initialized", "step", "cleanup finished"]
    );
}

pub(crate) async fn harness_files_directory() {
    let device = Device::open(&registered(&Arc::new(Observations::default())));
    let report = harnessed(&device)
        .parameter("watch_directory", true)
        .run::<OwnerSuites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    let watched = report.artifact("capture/chamber.log");
    assert_eq!(watched.media_type, "text/plain");
    assert!(!watched.abandoned);
    assert_eq!(
        String::from_utf8(watched.bytes.clone()).unwrap(),
        WATCHED_CONTENT,
        "the rewrite replaced what the watch had already streamed"
    );
}

pub(crate) async fn harness_timeout_up() {
    let observations = Arc::new(Observations::default());
    let device = Device::open(&registered(&observations));
    let error = harnessed(&device)
        .parameter("wait", true)
        .timeout(Duration::from_millis(100))
        .run::<OwnerSuites>()
        .await
        .expect_err("waiting execution times out");
    assert!(matches!(error, HarnessError::Timeout(_)), "{error}");
    let events = observations.events();
    assert!(events.contains(&"step"));
    assert_eq!(events.last(), Some(&"cleanup finished"));
}

pub(crate) async fn harness_refuses_opening() {
    // The template is resolved out of the declared suites, which is why the
    // harness needs only its id. A name no suite declares is therefore a
    // question the harness can answer without running anything at all.
    let observations = Arc::new(Observations::default());
    let device = Device::open(&registered(&observations));
    let described = Arc::clone(&device);
    let error = Harness::template("missing")
        .answering::<GetDevice, _>(move |()| described.describe())
        .run::<OwnerSuites>()
        .await
        .expect_err("unknown template is refused");
    assert!(matches!(error, HarnessError::UnknownTemplate(_)), "{error}");
    assert!(
        observations.events().is_empty(),
        "nothing was opened, so nothing had to be released"
    );
}

pub(crate) async fn harness_preserves_failure() {
    let observations = Arc::new(Observations::default());
    let device = Device::open(&registered(&observations));
    let report = harnessed(&device)
        .parameter("initialize_fail", true)
        .run::<OwnerSuites>()
        .await
        .expect("authored failures are execution outcomes");
    assert!(!report.passed());
    assert!(
        report.diagnostics().contains("initialization refused"),
        "{}",
        report.diagnostics()
    );
    assert_eq!(observations.events(), ["partial initialization dropped"]);
}

pub(crate) async fn harness_caller_settles() {
    let observations = Arc::new(Observations::default());
    let device = Device::open(&registered(&observations));
    let harness = harnessed(&device)
        .parameter("wait", true)
        .timeout(Duration::from_millis(100));
    let caller = tokio::spawn(harness.run::<OwnerSuites>());
    observations.started.notified().await;
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    tokio::time::timeout(BOUND, async {
        while !observations.events().contains(&"cleanup finished") {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("owned helper task settles after its caller drops");
}

pub(crate) fn catalog_is_inert() {
    // A plugin whose preparation panics still exports its registration: the
    // declaration pass reads what was declared and opens nothing.
    let mut registrar = dokime::harmos::sidecar::Registrar::new();
    <CatalogOnly as dokime::Plugin>::register(&mut registrar)
        .expect("a catalog is exported without opening the plugin");
    let registrations = registrar.finish();
    let suites = registrations
        .iter()
        .find(|entry| entry.namespace == dokime::plugin::registration::SUITES_NAMESPACE)
        .unwrap();
    let decoded: dokime::definition::SuiteSet = serde_json::from_slice(&suites.payload).unwrap();
    assert_eq!(decoded.suites.len(), 1);
    assert_eq!(decoded.suites[0].suite_id, SUITE);
    assert_eq!(decoded.suites[0].suite.test_case_templates.len(), 1);
}
