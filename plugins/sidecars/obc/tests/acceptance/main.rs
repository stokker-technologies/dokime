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
use std::{
    collections::BTreeMap,
    num::NonZeroUsize,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use dokime::__harmos as harmos;
use dokime::execution::ExecutionScope;
use dokime::operations::hosting::{Host as Hosted, Serve, Served};
use dokime::transport::{
    DeliverSamplesRequest, EvidenceBatch, EvidenceReceipt, ExecutionSignal, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion,
    RunExecution, SidecarExecutionOutcome, StepAdmission,
};
use harmos::Context;
use harmos::plugin::{
    PluginArtifact, PluginBackend, PluginDefinition, PluginDefinitions, PluginLifecycle, PluginLifecycleAuthority, PluginLifecyclePhase,
    PluginManager, PluginScope, PluginTransition, Router,
};
use plugin_obc::ObcPlugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type NoticeHandler = Box<dyn Fn(&harmos::ServiceContext<Fixture>, harmos::sidecar::Notice) + Send + Sync>;
type Resident = harmos::sidecar::Supervised<Fixture, harmos::sidecar::Executable, NoticeHandler>;

/* === Structs === */

struct Fixture;

struct Backend {
    services: harmos::operations::OperationManager<Lanes>,
    lease: Option<harmos::Running<harmos::operations::resident::Activation>>,
    registration: Option<harmos::operations::PluginRegistration>,
    scope: Option<PluginScope>,
    sdk: Option<harmos::operations::PluginSdkRegistration>,
    registered: bool,
}

#[derive(Default)]
struct Captures {
    scopes: BTreeMap<String, dokime::RecordScope>,
    publications: Vec<(harmos::streams::ManagedPublication, dokime::RecordScope)>,
}

struct Hosting {
    captures: Arc<Mutex<Captures>>,
    pages: Mutex<BTreeMap<String, Vec<EvidenceBatch>>>,
    operations: OnceLock<harmos::operations::OperationManager<Lanes>>,
}

struct Host {
    resident: Option<Resident>,
    hosting: Arc<Hosting>,
    operations: harmos::operations::OperationManager<Lanes>,
}

struct Bench {
    runtime: harmos::Runtime<Fixture>,
    plugins: PluginManager<(), Backend, ()>,
    host: Host,
    declaration: harmos::plugin::inventory::Declaration,
}

/* === Enums === */

#[harmos::lanes(harmos_path = dokime::__harmos)]
enum Lanes {
    #[default]
    Open,
}

/* === Traits === */

/* === Implementations === */

impl dokime::__harmos::Application for Fixture {
    type Lanes = Lanes;
    type State = ();
    type Transaction = dokime::__harmos::state::Never;
    type Record = ();
    type Streams = ();
}

impl Hosting {
    fn scope_of<O: harmos::Operation>(context: &Context<'_, O>) -> harmos::Result<String> {
        Ok(context
            .call()
            .value::<ExecutionScope>()
            .ok()
            .flatten()
            .ok_or(harmos::Error::InvalidInput {
                field: "execution scope",
                reason: "the call is not part of a template run",
            })?
            .execution)
    }

    async fn replay(&self, token: &str) -> harmos::Result<()> {
        let published = {
            let mut captures = self.captures.lock().unwrap();
            let (mine, others) = std::mem::take(&mut captures.publications)
                .into_iter()
                .partition::<Vec<_>, _>(|(publication, _)| publication.context() == Some(token));
            captures.publications = others;
            mine
        };
        let operations = self.operations.get().expect("the host operations are set");
        for (publication, scope) in published {
            use dokime::telemetry::TelemetryRow;
            use harmos::Message;
            let row = plugin_obc::suites::checks::pulse::PulseRow::decode(publication.payload()).unwrap();
            let micros = u64::try_from(publication.published_unix_micros().unwrap()).unwrap();
            let instant = std::time::UNIX_EPOCH.checked_add(Duration::from_micros(micros)).unwrap();
            let mut sample = dokime::telemetry::TelemetrySample::new(
                dokime::model::identity::new_id(),
                dokime::Identifier::new(publication.stream()).unwrap(),
                publication.sequence(),
                dokime::Timestamp::from_system_time(instant),
                publication.owner(),
            );
            sample.sample_id = dokime::TelemetrySampleId::from_seed(format!(
                "{}/{}/{:?}/{}",
                publication.owner(),
                publication.stream(),
                publication.generation(),
                publication.sequence(),
            ));
            sample.values.extend(row.into_row());
            sample.scope = Some(scope.clone());
            let signal = ExecutionSignal {
                execution_token: token.to_owned(),
                samples: Some(DeliverSamplesRequest {
                    samples: vec![serde_json::to_string(&sample).unwrap()],
                    gaps: Vec::new(),
                    scope_json: serde_json::to_string(&scope).unwrap(),
                }),
                ..ExecutionSignal::default()
            };
            operations
                .call_encoded(
                    &owned(<dokime::execution::Signal<plugin_obc::Pulse> as harmos::operations::OperationDeclaration>::ID),
                    signal.encode(),
                )
                .await?;
        }
        Ok(())
    }
}

impl Serve<EvidenceBatch> for Hosting {
    fn serve<'a>(&'a self, batch: EvidenceBatch, context: Context<'a, EvidenceBatch>) -> Served<'a, EvidenceReceipt> {
        Box::pin(async move {
            let token = Hosting::scope_of(&context).map_err(refused)?;
            let mut admitted = Vec::new();
            for encoded in &batch.step_begins {
                let start: dokime::domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
                {
                    let mut captures = self.captures.lock().unwrap();
                    let scope = captures.scopes.get_mut(&token).unwrap();
                    scope.step_id = Some(start.step_id.clone());
                    scope.origin = dokime::EvidenceOrigin::Step { attempt_id: start.attempt_id.clone() };
                }
                admitted.push(StepAdmission {
                    attempt_id: start.attempt_id.to_string(),
                    accepted: true,
                    message: String::new(),
                });
            }
            let settled = batch.record_subscriptions_settling;
            if settled {
                self.replay(&token).await.map_err(refused)?;
            }
            self.pages.lock().unwrap().entry(token).or_default().push(batch);
            Ok(EvidenceReceipt { admitted, settled }.into())
        })
    }
}

impl Serve<OperatorQuestion> for Hosting {
    fn serve<'a>(&'a self, _: OperatorQuestion, _: Context<'a, OperatorQuestion>) -> Served<'a, OperatorAnswer> {
        Box::pin(std::future::ready(Err(refusal("the OBC checks never ask the operator"))))
    }
}

impl Serve<LeaseAsk> for Hosting {
    fn serve<'a>(&'a self, _: LeaseAsk, _: Context<'a, LeaseAsk>) -> Served<'a, LeaseResolution> {
        Box::pin(std::future::ready(Ok(LeaseResolution {
            granted: false,
            message: "the test host leases nothing".into(),
        }
        .into())))
    }
}

impl Serve<dokime::records::RecordPublication> for Hosting {
    fn serve<'a>(&'a self, _: dokime::records::RecordPublication, _: Context<'a, dokime::records::RecordPublication>) -> Served<'a, ()> {
        Box::pin(std::future::ready(Ok(().into())))
    }
}

// The step built-ins a suite plugin requires; this host watches no files.
impl Serve<dokime::operations::files::WatchFile> for Hosting {
    fn serve<'a>(
        &'a self, _: dokime::operations::files::WatchFile, _: Context<'a, dokime::operations::files::WatchFile>,
    ) -> Served<'a, dokime::domain::model::files::wire::WatchFileOutcome> {
        Box::pin(std::future::ready(Err(refusal("the test host watches no files"))))
    }
}

impl Serve<dokime::operations::files::AwaitFileWatchArmed> for Hosting {
    fn serve<'a>(
        &'a self, _: dokime::operations::files::AwaitFileWatchArmed, _: Context<'a, dokime::operations::files::AwaitFileWatchArmed>,
    ) -> Served<'a, dokime::domain::model::files::wire::FileWatchArmed> {
        Box::pin(std::future::ready(Err(refusal("the test host watches no files"))))
    }
}

impl PluginLifecycle for Host {
    type Guest = ();
    type Sidecar = Backend;
    type Embedded = ();

    fn preflight(&mut self, candidate: &PluginDefinitions) -> harmos::Result<()> {
        if self.resident.is_none() {
            let captures = Arc::clone(&self.hosting.captures);
            let handler: NoticeHandler = Box::new(move |_, notice| {
                let Some(publication) = notice.published() else {
                    return;
                };
                if publication.stream() != "obc.pulse" {
                    return;
                }
                assert_eq!(publication.owner(), "plugin_obc");
                assert_eq!(publication.schema(), 1);
                assert_eq!(publication.missed(), 0);
                let token = publication.context().unwrap();
                let mut captures = captures.lock().unwrap();
                let scope = captures.scopes.get(token).unwrap().clone();
                assert!(captures.publications.len() < 4096);
                captures.publications.push((publication.clone(), scope));
            });
            let executable = harmos::sidecar::Executable::at(executable()).expect("OBC executable has a valid declaration");
            let resident = Resident::new(executable, handler);
            self.resident = Some(resident);
        }
        self.operations.validate_requirements(candidate)
    }

    fn prepare(&mut self, _: &PluginDefinition) -> harmos::Result<PluginBackend<(), Backend, ()>> {
        Ok(PluginBackend::Sidecar(Backend {
            services: self.operations.clone(),
            lease: None,
            registration: None,
            scope: None,
            sdk: None,
            registered: false,
        }))
    }

    async fn fence(&mut self, _: PluginTransition, _: &[PluginDefinition]) -> harmos::Result<()> {
        Ok(())
    }

    async fn start(
        &mut self, definition: &PluginDefinition, router: &Router, authority: &PluginLifecycleAuthority<'_>,
        backend: &mut PluginBackend<(), Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else { unreachable!() };
        let PluginArtifact::Sidecar { path, sha256, .. } = &definition.artifact else { unreachable!() };
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(Sha256::digest(&bytes).as_slice(), sha256);
        let executable = harmos::sidecar::Executable::at(path).unwrap();
        let inspected: std::collections::BTreeMap<_, _> =
            executable.declaration().operations.iter().map(|operation| (&operation.id, operation)).collect();
        let approved: std::collections::BTreeMap<_, _> =
            definition.contributions.operations.iter().map(|operation| (&operation.id, operation)).collect();
        assert_eq!(inspected, approved);
        let restart = executable.declaration().restart().expect("OBC declares its restart policy").bounded();
        let policy = harmos::ServicePolicy {
            restart: harmos::operations::RestartPolicy::OnFailure {
                credit: usize::try_from(restart.credit).unwrap(),
                sustained_health: Duration::from_millis(restart.sustained_health_ms),
                backoff: harmos::operations::Backoff::Exponential {
                    initial: Duration::from_millis(restart.backoff_base_ms),
                    maximum: Duration::from_millis(restart.backoff_cap_ms),
                    jitter: harmos::operations::Jitter::Deterministic,
                },
            },
            ..harmos::services::POLICY
        };
        let resident = self
            .resident
            .as_ref()
            .expect("preflighted resident")
            .clone()
            .routed(router)
            .expected_registrations(&definition.contributions.registrations);
        self.operations.register_service(harmos::services::ServiceDefinition { id: definition.id.clone(), policy }, {
            let resident = resident.clone();
            move |_: Arc<harmos::ExecutionContext>| resident.clone()
        })?;
        backend.registered = true;
        backend.lease = Some(self.operations.submit_resident(&definition.id).await?);
        let native = Arc::new(resident.staged_operations().await.unwrap());
        let scope = native.scope().unwrap();
        assert!(authority.matches_scope(scope, definition.kind(), PluginLifecyclePhase::Initialize));
        backend.scope = Some(scope.clone());
        backend.registration =
            Some(self.operations.stage_plugin(scope, |bindings| {
                bindings.register_native(Lanes::Open, Arc::clone(&native), NonZeroUsize::new(128).unwrap())
            })?);
        backend.sdk = Some(native.stage_sdk(&self.operations, Lanes::Open)?);
        backend
            .sdk
            .as_ref()
            .unwrap()
            .submit_initialize(authority, &definition.configuration, NonZeroUsize::new(4 * 1024 * 1024).unwrap())?
            .await?;
        Ok(())
    }

    async fn stop(
        definition: &PluginDefinition, authority: &PluginLifecycleAuthority<'_>, backend: &mut PluginBackend<(), Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else { unreachable!() };
        if let Some(scope) = &backend.scope {
            assert!(authority.matches_scope(scope, definition.kind(), PluginLifecyclePhase::Shutdown));
        }
        if let Some(sdk) = &backend.sdk {
            sdk.settle().await?;
            sdk.submit_shutdown(authority)?.await?;
        }
        if let Some(registration) = &mut backend.registration {
            registration.shutdown().await?;
        }
        if let Some(lease) = backend.lease.take() {
            lease.release().await?;
        }
        if backend.registered {
            backend.services.unregister_resident(&definition.id).await?;
            backend.registered = false;
        }
        if let Some(sdk) = &mut backend.sdk {
            sdk.close().await?;
        }
        backend.sdk = None;
        backend.registration = None;
        backend.scope = None;
        Ok(())
    }
}

impl Bench {
    async fn new() -> Self {
        let path = std::path::PathBuf::from(executable());
        let configuration = serde_json::to_vec(&dokime::plugin::Configuration::declared::<plugin_obc::BenchSettings>()).unwrap();
        let mut registrar = harmos::plugin::Registrar::new();
        <ObcPlugin as harmos::Plugin>::register(&mut registrar).unwrap();
        let executable = harmos::sidecar::Executable::at(&path).unwrap();
        let declaration = executable.declaration().clone();
        let mut definition = PluginDefinition::sidecar(&executable, configuration).unwrap();
        definition.contributions.registrations = registrar.finish();
        let definitions = PluginDefinitions::new([definition], []).unwrap();
        let hosting = Arc::new(Hosting {
            captures: Arc::default(),
            pages: Mutex::default(),
            operations: OnceLock::new(),
        });
        let runtime = harmos::Runtime::<Fixture>::launch((), FixtureConfig { hosting: Some(hosting.clone()) }).await.unwrap();
        let mut host = Host {
            resident: None,
            hosting,
            operations: runtime.operations.clone(),
        };
        let _ = host.hosting.operations.set(host.operations.clone());
        let mut plugins = PluginManager::new(definitions).unwrap();
        plugins.initialize(&mut host).await.unwrap();
        let mut ready = host.resident.as_ref().unwrap().ready();
        ready.wait_for(|ready| *ready).await.unwrap();
        Self { runtime, plugins, host, declaration }
    }

    fn start(&self, token: &str, parameters: &BTreeMap<&str, dokime::Value>) -> tokio::task::JoinHandle<SidecarExecutionOutcome> {
        use harmos::Message;
        let cycle = dokime::TestCycleId::from_seed("native-cycle");
        self.host.hosting.captures.lock().unwrap().scopes.insert(
            token.to_owned(),
            dokime::RecordScope {
                test_cycle_id: Some(cycle.clone()),
                test_execution_id: Some(dokime::TestExecutionId::from_uuid(token.to_owned()).unwrap()),
                origin: dokime::EvidenceOrigin::Initialization,
                ..Default::default()
            },
        );
        let request = RunExecution {
            mode: "automatic".into(),
            execution_token: token.into(),
            test_execution_id: token.into(),
            test_cycle_id: cycle.to_string(),
            workflow: "obc.checks".into(),
            template_identifier: "obc.checks.pulse".into(),
            parameters_json: serde_json::to_string(parameters).unwrap(),
            secrets_json: "{}".into(),
        };
        let mut context = harmos::operations::CallContext::default();
        context.insert_portable(ExecutionScope::execution(token)).unwrap();
        let operations = self.runtime.operations.with_context(context);
        tokio::spawn(async move {
            let bytes = operations
                .call_encoded(
                    &owned(<dokime::execution::Run<plugin_obc::Pulse> as harmos::operations::OperationDeclaration>::ID),
                    request.encode(),
                )
                .await
                .unwrap();
            SidecarExecutionOutcome::decode(bytes.as_slice()).unwrap()
        })
    }

    async fn shutdown(mut self) {
        self.plugins.shutdown(&mut self.host).await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

#[derive(Default, serde::Deserialize)]
struct FixtureConfig {
    #[serde(skip)]
    hosting: Option<Arc<Hosting>>,
}

#[harmos::catalog(harmos_path = dokime::harmos)]
enum HostOperations {
    Operation0(dokime::operations::files::AwaitFileWatchArmed),
    Operation1(dokime::operations::files::WatchFile),
    Operation2(dokime::records::RecordPublication),
    Operation3(EvidenceBatch),
    Operation4(OperatorQuestion),
    Operation5(LeaseAsk),
    Publish(harmos::builtins::streams::Publish),
}

impl harmos::Lifecycle for Fixture {
    type Config = FixtureConfig;
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;
    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().operation_settings(harmos::operations::Settings {
            preparation_bytes: Some(NonZeroUsize::new(4 * 1024 * 1024).unwrap()),
            ..Default::default()
        });
        let hosting = config.hosting.expect("fixture hosting resources");
        setup.resource::<Hosted<dokime::operations::files::AwaitFileWatchArmed>>(
            Hosted::<dokime::operations::files::AwaitFileWatchArmed>::new(hosting.clone()),
        );
        setup
            .resource::<Hosted<dokime::operations::files::WatchFile>>(Hosted::<dokime::operations::files::WatchFile>::new(hosting.clone()));
        setup.resource::<Hosted<dokime::records::RecordPublication>>(Hosted::<dokime::records::RecordPublication>::new(hosting.clone()));
        setup.resource::<Hosted<EvidenceBatch>>(Hosted::<EvidenceBatch>::new(hosting.clone()));
        setup.resource::<Hosted<OperatorQuestion>>(Hosted::<OperatorQuestion>::new(hosting.clone()));
        setup.resource::<Hosted<LeaseAsk>>(Hosted::<LeaseAsk>::new(hosting.clone()));
        setup.operations::<HostOperations>(Lanes::Open);
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn cleanup(&mut self) -> harmos::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

// The host operations a suite plugin requires, served by this fixture host.
// A refusal the operation's status error carries as an invalid argument.
fn refusal<S: From<std::io::Error>>(reason: impl Into<String>) -> S {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, reason.into()).into()
}

fn refused(error: harmos::Error) -> std::io::Error {
    refusal(error.to_string())
}

// The Runtime Serves Every Sidecar Operation Under The Plugin That Owns It.
fn owned(operation: &str) -> String {
    format!("{}.{operation}", plugin_obc::SOURCE)
}

async fn dispatch(operations: &harmos::operations::OperationManager<Lanes>, method: &str, request: Vec<u8>) {
    let response = operations.call_encoded(method, request).await;
    match response {
        Ok(_) => {}
        Err(harmos::Error::OperationFailed { source, .. })
            if matches!(
                source.downcast_ref::<harmos::operations::NativeError>(),
                Some(harmos::operations::NativeError::Remote { .. } | harmos::operations::NativeError::Preparation { .. })
            ) => {}
        Err(error) => panic!("{method} was advertised but did not dispatch: {error}"),
    }
}

/* === Tests === */

#[tokio::test]
async fn inventory_matches_dispatch() {
    use harmos::Message;
    use harmos::operations::OperationDeclaration;
    let bench = Bench::new().await;
    let runtime = &bench.runtime;
    let declared = &bench.declaration;
    let mut served = Vec::new();

    // Every template of every suite serves its run and its signal.
    for template in dokime::catalog::entries::<plugin_obc::suites::Suites, dokime::catalog::Suites>()
        .into_iter()
        .flat_map(|definition| definition().suite.test_case_templates)
        .map(|template| template.metadata.identifier.to_string())
    {
        let run = format!("{template}.run");
        let signal = format!("{template}.signal");
        dispatch(&runtime.operations, &owned(&run), RunExecution::default().encode()).await;
        dispatch(&runtime.operations, &owned(&signal), ExecutionSignal::default().encode()).await;
        served.push((run, <dokime::execution::Run<()> as OperationDeclaration>::POLICY.timeout.as_millis() as u64));
        served.push((signal, <dokime::execution::Signal<()> as OperationDeclaration>::POLICY.timeout.as_millis() as u64));
    }
    use plugin_obc::domain::operations::device::{ReadDevice, SetPower};
    use plugin_obc::domain::operations::{AskReadiness, GetStatus, ListConnections};
    for (operation, timeout) in [
        (<GetStatus as OperationDeclaration>::ID, <GetStatus as OperationDeclaration>::POLICY.timeout),
        (<AskReadiness as OperationDeclaration>::ID, <AskReadiness as OperationDeclaration>::POLICY.timeout),
        (<ListConnections as OperationDeclaration>::ID, <ListConnections as OperationDeclaration>::POLICY.timeout),
        (<ReadDevice as OperationDeclaration>::ID, <ReadDevice as OperationDeclaration>::POLICY.timeout),
    ] {
        // These Bench Operations Have Empty Native Requests.
        runtime.operations.call_encoded(&owned(operation), Vec::new()).await.unwrap();
        served.push((operation.to_owned(), timeout.as_millis() as u64));
    }
    runtime.operations.call_encoded(&owned(<SetPower as OperationDeclaration>::ID), SetPower { enabled: false }.encode()).await.unwrap();
    served.push((<SetPower as OperationDeclaration>::ID.to_owned(), <SetPower as OperationDeclaration>::POLICY.timeout.as_millis() as u64));

    // Demand drives the real host probe; two rows cover its retained reader
    // across ticks rather than testing sysinfo in an inline unit test.
    let system = runtime.streams.publication("obc.system", 1).unwrap();
    // The watch holds demand and yields each published tick.
    let mut demand = system.watch();
    let mut rows = Vec::new();
    while rows.len() < 2 {
        let item = futures_util::StreamExt::next(&mut demand).await.expect("the demanded host observer publishes consecutive ticks");
        if let harmos::streams::Item::Row(row) = item {
            rows.push(row);
        }
    }
    let decoder =
        harmos::schema::Validator::from_schema(system.definition().schema().unwrap(), &harmos::config::SchemaSettings::default()).unwrap();
    for row in rows {
        let mut used = None;
        decoder
            .visit(&row.value.payload, |path, event| {
                if let (
                    [harmos::schema::PathElement::Field { name: "system_memory_used_bytes", .. }],
                    harmos::schema::Event::Scalar(harmos::schema::Scalar::I64(bytes)),
                ) = (path, event)
                {
                    used = Some(bytes);
                }
                Ok::<(), std::convert::Infallible>(())
            })
            .unwrap();
        assert!(used.unwrap() > 0);
    }
    drop(demand);
    // The listener is reached with a record; an empty one is refused, which is
    // dispatch all the same.
    dispatch(&runtime.operations, &owned("obc.bench.connection"), Vec::new()).await;
    served.push(("obc.bench.connection".to_owned(), 30_000));
    // The built-in observe control is served but never declared; it refuses
    // an observer the sidecar does not serve.
    let observe = <harmos::operations::observers::Control as harmos::operations::OperationDeclaration>::ID;
    dispatch(&runtime.operations, &owned(observe), harmos::operations::observers::ControlRequest::default().encode()).await;
    // The ripple analysis is reached too; an empty request is refused all the same.
    let ripple = <plugin_obc::analyses::ripple::Ripple as dokime::analysis::Analysis>::ID;
    dispatch(&runtime.operations, &owned(ripple), Vec::new()).await;
    served.push((ripple.to_owned(), <plugin_obc::analyses::ripple::Ripple as dokime::analysis::Analysis>::TIMEOUT.as_millis() as u64));
    served.sort();
    let mut inspected = declared
        .operations
        .iter()
        .map(|operation| (operation.id.to_string(), operation.policy.timeout.as_millis() as u64))
        .collect::<Vec<_>>();
    inspected.sort();
    assert_eq!(inspected, served);
    bench.shutdown().await;
}

#[tokio::test]
async fn samples_keep_context() {
    use dokime::Value;
    use harmos::Message;

    let mut bench = Bench::new().await;
    let parameters = BTreeMap::from([
        ("sample_count", Value::Integer(3)),
        ("tolerance", Value::from(plugin_obc::DEFAULT_TOLERANCE)),
        ("capture_system", Value::from(false)),
        ("require_confirmation", Value::from(false)),
    ]);
    let tokens = ["native-a", "native-b"].map(|seed| dokime::TestExecutionId::from_seed(seed).to_string());
    let runs = tokens.iter().map(|token| bench.start(token, &parameters)).collect::<Vec<_>>();

    for (token, run) in tokens.iter().zip(runs) {
        let outcome = run.await.unwrap();
        assert!(outcome.passed, "local criteria must retain their evidence: {outcome:?}");
        assert!(<EvidenceBatch as dokime::schema::Described>::FIELDS.iter().all(|field| field.name != "samples"));
        let pages = bench.host.hosting.pages.lock().unwrap().get(token).cloned().expect("the host admitted this run's pages");
        let mut admitted = std::collections::BTreeSet::new();
        let mut completed = std::collections::BTreeSet::new();
        for page in &pages {
            for encoded in &page.step_begins {
                let start: dokime::domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
                assert!(admitted.insert(start.attempt_id.clone()));
            }
            for encoded in &page.step_attempts {
                let attempt: dokime::domain::model::execution::result::TestStepAttempt = serde_json::from_str(encoded).unwrap();
                assert!(admitted.contains(&attempt.attempt_id));
                assert!(completed.insert(attempt.attempt_id));
            }
        }
        let last = pages.last().unwrap();
        assert!(last.cleanup_settled);
        assert_eq!(admitted.len(), 3);
        assert_eq!(completed, admitted);
    }
    // Stop the actual plugin before reading its complete native stream.
    bench.plugins.shutdown(&mut bench.host).await.unwrap();

    let stream = bench.runtime.streams.publication("obc.pulse", 1).unwrap();
    assert_eq!(
        stream.definition().schema().unwrap(),
        &harmos::schema::graph::Schema::from_declaration(
            <plugin_obc::suites::checks::pulse::PulseRow as harmos::schema::Described>::DEFINITION,
            &harmos::config::SchemaSettings::default(),
        )
        .unwrap()
    );
    let mut samples: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for row in stream.query(..).await.unwrap() {
        assert!(row.value.published_unix_micros.is_some());
        let decoded = plugin_obc::suites::checks::pulse::PulseRow::decode(&row.value.payload).unwrap();
        let index = decoded.sample_index;
        samples.entry(row.value.context.clone().expect("native template rows carry context")).or_default().push(index);
    }
    assert_eq!(samples.len(), 2);
    for token in &tokens {
        let indices = &samples[token];
        assert!(indices.len() >= 3);
        assert_eq!(indices, &(0..indices.len() as i64).collect::<Vec<_>>());
    }
    bench.runtime.shutdown().await.unwrap();
}

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_plugin-obc")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_plugin-obc").map(str::to_owned))
        .expect("workspace executable")
}
