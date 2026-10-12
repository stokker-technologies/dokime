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
use dokime::transport::{
    DeliverSamplesRequest, EvidenceBatch, EvidenceReceipt, ExecutionSignal, LeaseAsk,
    LeaseResolution, LeaseStream, OperatorAnswer, OperatorQuestion, PushEvidence,
    RunExecutionRequest, SidecarExecutionOutcome, StepAdmission,
};
use harmos::load::{
    PluginArtifact, PluginBackend, PluginContributions, PluginDefinition, PluginDefinitions,
    PluginLifecycle, PluginLifecycleAuthority, PluginLifecyclePhase, PluginManager, PluginScope,
    PluginTransition, Router,
};
use harmos::operations::{Context, Operation};
use mini_sidecar::MiniSidecar;

/* === Constants === */
const EXECUTABLE: &str = env!("CARGO_BIN_EXE_mini-sidecar");

/* === Macros === */

/* === Type Aliases === */
type NoticeHandler = Box<dyn Fn(&harmos::Service<Fixture>, harmos::sidecar::Notice) + Send + Sync>;
type Resident = harmos::sidecar::Supervised<Fixture, harmos::sidecar::Executable, NoticeHandler>;
type Publication = harmos::sidecar::Publish<Fixture, harmos::sidecar::Executable, NoticeHandler>;

/* === Structs === */
struct Fixture;

struct Backend {
    services: harmos::services::ServiceManager<Fixture>,
    lease: Option<harmos::services::ServiceLease<Fixture>>,
    registration: Option<harmos::operations::PluginRegistration>,
    scope: Option<PluginScope>,
    sdk: Option<harmos::operations::PluginSdkRegistration>,
    registered: bool,
}

#[derive(Default)]
struct Captures {
    scopes: BTreeMap<String, dokime::core::RecordScope>,
    publications: Vec<(harmos::ManagedPublication, dokime::core::RecordScope)>,
}

struct Hosting {
    captures: Arc<Mutex<Captures>>,
    pages: Mutex<BTreeMap<String, Vec<EvidenceBatch>>>,
    operations: OnceLock<harmos::OperationManager>,
    resident: OnceLock<Resident>,
}

struct Host {
    resident: Option<Resident>,
    hosting: Arc<Hosting>,
    services: harmos::services::ServiceManager<Fixture>,
    operations: harmos::OperationManager,
}

struct Bench {
    runtime: harmos::Runtime<Fixture>,
    plugins: PluginManager<(), Backend, ()>,
    host: Host,
    declaration: harmos::artifact::inventory::Declaration,
}

struct Evidence(Arc<Hosting>);

struct Question;

struct Filing;

struct Lease;

struct Publishing {
    hosting: Arc<Hosting>,
    native: Option<Publication>,
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
    type State = ();
    type Transaction = dokime::__harmos::Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

impl Hosting {
    fn scope_of(context: &Context<'_>) -> harmos::Result<String> {
        Ok(context
            .context()
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
            let row = mini_sidecar::suites::checks::pulse::PulseRow::decode(publication.payload())
                .unwrap();
            let micros = u64::try_from(publication.published_unix_micros().unwrap()).unwrap();
            let instant = std::time::UNIX_EPOCH
                .checked_add(Duration::from_micros(micros))
                .unwrap();
            let mut sample = dokime::telemetry::TelemetrySample::new(
                dokime::core::Identifier::new(publication.stream()).unwrap(),
                publication.sequence(),
                dokime::core::Timestamp::from_system_time(instant),
                publication.owner(),
            );
            sample.sample_id = dokime::core::TelemetrySampleId::from_seed(format!(
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
                    <dokime::execution::Signal<mini_sidecar::Pulse> as harmos::operations::OperationDeclaration>::ID,
                    signal.encode_to_vec(),
                )
                .await?;
        }
        Ok(())
    }
}

impl harmos::operations::Declares for Evidence {
    type Declaration = PushEvidence;
}

impl Operation for Evidence {
    type Declaration = PushEvidence;
    type Resources = Hosting;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(hosting: Arc<Hosting>) -> harmos::Result<Self> {
        Ok(Self(hosting))
    }

    async fn run<'ctx>(
        &mut self,
        context: Self::Context<'ctx>,
        batch: &mut EvidenceBatch,
        _: &mut (),
    ) -> harmos::Result<EvidenceReceipt> {
        let token = Hosting::scope_of(&context)?;
        let batch = std::mem::take(batch);
        let mut admitted = Vec::new();
        for encoded in &batch.step_begins {
            let start: dokime::execution::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            {
                let mut captures = self.0.captures.lock().unwrap();
                let scope = captures.scopes.get_mut(&token).unwrap();
                scope.step_id = Some(start.step_id.clone());
                scope.origin = dokime::core::EvidenceOrigin::Step {
                    attempt_id: start.attempt_id.clone(),
                };
            }
            admitted.push(StepAdmission {
                attempt_id: start.attempt_id.to_string(),
                accepted: true,
                message: String::new(),
            });
        }
        let settled = batch.record_subscriptions_settling;
        if settled {
            self.0.replay(&token).await?;
        }
        self.0
            .pages
            .lock()
            .unwrap()
            .entry(token)
            .or_default()
            .push(batch);
        Ok(EvidenceReceipt { admitted, settled })
    }
}

impl harmos::operations::Declares for Question {
    type Declaration = dokime::transport::AskOperator;
}

impl Operation for Question {
    type Declaration = dokime::transport::AskOperator;
    type Resources = Hosting;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(_: Arc<Hosting>) -> harmos::Result<Self> {
        Ok(Self)
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        _: &mut OperatorQuestion,
        _: &mut (),
    ) -> harmos::Result<OperatorAnswer> {
        Err(harmos::Error::InvalidInput {
            field: "operator question",
            reason: "the mini checks never ask the operator",
        })
    }
}

impl harmos::operations::Declares for Filing {
    type Declaration = dokime::records::PublishRecord;
}

impl Operation for Filing {
    type Declaration = dokime::records::PublishRecord;
    type Resources = Hosting;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(_: Arc<Hosting>) -> harmos::Result<Self> {
        Ok(Self)
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        _: &mut dokime::records::RecordPublication,
        _: &mut (),
    ) -> harmos::Result<()> {
        Ok(())
    }
}

impl harmos::operations::Declares for Lease {
    type Declaration = LeaseStream;
}

impl Operation for Lease {
    type Declaration = LeaseStream;
    type Resources = Hosting;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(_: Arc<Hosting>) -> harmos::Result<Self> {
        Ok(Self)
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        _: &mut LeaseAsk,
        _: &mut (),
    ) -> harmos::Result<LeaseResolution> {
        Ok(LeaseResolution {
            granted: false,
            message: "the test host leases nothing".into(),
        })
    }
}

impl harmos::operations::Declares for Publishing {
    type Declaration = harmos::operations::host::Publish;
}

impl Operation for Publishing {
    type Declaration = harmos::operations::host::Publish;
    type Resources = Hosting;
    type Error = harmos::Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(hosting: Arc<Hosting>) -> harmos::Result<Self> {
        Ok(Self {
            hosting,
            native: None,
        })
    }

    async fn run<'ctx>(
        &mut self,
        context: Self::Context<'ctx>,
        request: &mut harmos::operations::host::Publication,
        _: &mut (),
    ) -> harmos::Result<()> {
        if self.native.is_none() {
            let resident =
                self.hosting
                    .resident
                    .get()
                    .cloned()
                    .ok_or(harmos::Error::InvalidInput {
                        field: "native publication",
                        reason: "the resident is not running",
                    })?;
            self.native = Some(Publication::new(Arc::new(resident))?);
        }
        self.native
            .as_mut()
            .expect("native publication initialized")
            .run(context, request, &mut ())
            .await
    }

    async fn stop(&mut self) -> harmos::Result<()> {
        if let Some(native) = &mut self.native {
            native.stop().await?;
        }
        Ok(())
    }

    async fn finalize(&mut self) -> harmos::Result<()> {
        if let Some(native) = &mut self.native {
            native.finalize().await?;
        }
        Ok(())
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
                if publication.stream() != "mini.pulse" {
                    return;
                }
                assert_eq!(publication.owner(), "mini-sidecar");
                assert_eq!(publication.schema(), 1);
                assert_eq!(publication.missed(), 0);
                let token = publication.context().unwrap();
                let mut captures = captures.lock().unwrap();
                let scope = captures.scopes.get(token).unwrap().clone();
                assert!(captures.publications.len() < 4096);
                captures.publications.push((publication.clone(), scope));
            });
            let executable = harmos::sidecar::Executable::at(EXECUTABLE)
                .expect("mini executable has a valid declaration");
            let resident = Resident::new(executable, handler);
            let _ = self.hosting.resident.set(resident.clone());
            let _ = self.hosting.operations.set(self.operations.clone());
            for registered in [
                self.operations.register_wire_operation::<Evidence>(
                    "fixture.execution",
                    Lanes::Open,
                    Arc::clone(&self.hosting),
                ),
                self.operations.register_wire_operation::<Question>(
                    "fixture.execution",
                    Lanes::Open,
                    Arc::clone(&self.hosting),
                ),
                self.operations.register_wire_operation::<Lease>(
                    "fixture.execution",
                    Lanes::Open,
                    Arc::clone(&self.hosting),
                ),
                self.operations.register_wire_operation::<Filing>(
                    "fixture.records",
                    Lanes::Open,
                    Arc::clone(&self.hosting),
                ),
                self.operations.register_wire_operation::<Publishing>(
                    "fixture.publications",
                    Lanes::Open,
                    Arc::clone(&self.hosting),
                ),
            ] {
                registered?;
            }
            self.resident = Some(resident);
        }
        self.operations.validate_requirements(candidate)
    }

    fn prepare(&mut self, _: &PluginDefinition) -> harmos::Result<PluginBackend<(), Backend, ()>> {
        Ok(PluginBackend::Sidecar(Backend {
            services: self.services.clone(),
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
        &mut self,
        definition: &PluginDefinition,
        router: &Router,
        authority: &PluginLifecycleAuthority<'_>,
        backend: &mut PluginBackend<(), Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else {
            unreachable!()
        };
        let PluginArtifact::Sidecar { path, sha256, .. } = &definition.artifact else {
            unreachable!()
        };
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(Sha256::digest(&bytes).as_slice(), sha256);
        let executable = harmos::sidecar::Executable::at(path).unwrap();
        let inspected: std::collections::BTreeMap<_, _> = executable
            .declaration()
            .operations
            .iter()
            .map(|operation| (&operation.id, operation))
            .collect();
        let approved: std::collections::BTreeMap<_, _> = definition
            .contributions
            .operations
            .iter()
            .map(|operation| (&operation.id, operation))
            .collect();
        assert_eq!(inspected, approved);
        let restart = executable
            .declaration()
            .restart()
            .expect("mini declares its restart policy")
            .bounded();
        let policy = harmos::ServicePolicy {
            credit: usize::try_from(restart.credit).unwrap(),
            sustained_health: Duration::from_millis(restart.sustained_health_ms),
            backoff_base: Duration::from_millis(restart.backoff_base_ms),
            backoff_cap: Duration::from_millis(restart.backoff_cap_ms),
            stop_timeout: Duration::from_secs(5),
        };
        let resident = self
            .resident
            .as_ref()
            .expect("preflighted resident")
            .clone()
            .routed(router)
            .expected_registrations(&definition.contributions.registrations);
        self.services.register(
            harmos::services::ServiceDefinition {
                id: definition.id.clone(),
                policy,
            },
            {
                let resident = resident.clone();
                move |_: Arc<()>| resident.clone()
            },
        )?;
        backend.registered = true;
        backend.lease = Some(
            self.services
                .activate(&definition.id, &self.services.root_scope())
                .await?,
        );
        let native = Arc::new(
            tokio::time::timeout(Duration::from_secs(5), resident.staged_operations())
                .await
                .unwrap()
                .unwrap(),
        );
        let scope = native.scope().unwrap();
        assert!(authority.matches_scope(
            scope,
            definition.kind(),
            PluginLifecyclePhase::Initialize
        ));
        backend.scope = Some(scope.clone());
        backend.registration = Some(self.operations.stage_plugin(scope, |bindings| {
            bindings.register_native(
                Lanes::Open,
                Arc::clone(&native),
                NonZeroUsize::new(128).unwrap(),
            )
        })?);
        backend.sdk = Some(native.stage_sdk(&self.operations, Lanes::Open)?);
        backend
            .sdk
            .as_ref()
            .unwrap()
            .submit_initialize(
                authority,
                &definition.configuration,
                NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            )?
            .await?;
        Ok(())
    }

    async fn stop(
        definition: &PluginDefinition,
        authority: &PluginLifecycleAuthority<'_>,
        backend: &mut PluginBackend<(), Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else {
            unreachable!()
        };
        if let Some(scope) = &backend.scope {
            assert!(authority.matches_scope(
                scope,
                definition.kind(),
                PluginLifecyclePhase::Shutdown
            ));
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
            backend.services.unregister(&definition.id).await?;
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
        let path = std::path::PathBuf::from(EXECUTABLE);
        let bytes = std::fs::read(&path).unwrap();
        let declaration = harmos::sidecar::Executable::at(&path)
            .unwrap()
            .declaration()
            .clone();
        let configuration = serde_json::to_vec(&dokime::plugin::Configuration::declared::<
            mini_sidecar::BenchSettings,
        >())
        .unwrap();
        let mut registrar = harmos::artifact::Registrar::new();
        <MiniSidecar as harmos::Plugin>::register(&mut registrar).unwrap();
        let definition = PluginDefinition {
            id: declaration.id.clone(),
            version: declaration.version.clone(),
            artifact: PluginArtifact::Sidecar {
                path,
                sha256: Sha256::digest(&bytes).into(),
                arguments: Vec::new(),
                directory: None,
                environment: Default::default(),
            },
            configuration,
            contributions: PluginContributions {
                lifecycle: Some(declaration.contracts.lifecycle.clone()),
                operations: declaration.operations.clone(),
                wire_contracts: declaration.contracts.provides.clone(),
                required_contracts: Some(declaration.contracts.requires.clone()),
                streams: declaration.streams.clone(),
                observers: declaration.observers.clone(),
                observer_controls: match &declaration.carrier {
                    harmos::artifact::inventory::CarrierSection::Sidecar {
                        observer_controls,
                        ..
                    } => observer_controls.as_deref().cloned(),
                    harmos::artifact::inventory::CarrierSection::Guest => None,
                },
                listeners: declaration.listeners.clone(),
                sections: declaration.sections.clone(),
                registrations: registrar.finish(),
            },
            requires: Vec::new(),
        };
        let definitions = PluginDefinitions::new([definition], []).unwrap();
        let runtime = harmos::Runtime::<Fixture>::builder((), ())
            .register(Lanes::catalog())
            .operation_preparation_bytes(NonZeroUsize::new(4 * 1024 * 1024).unwrap())
            .runtime()
            .await
            .unwrap();
        let mut host = Host {
            resident: None,
            hosting: Arc::new(Hosting {
                captures: Arc::default(),
                pages: Mutex::default(),
                operations: OnceLock::new(),
                resident: OnceLock::new(),
            }),
            services: runtime.services.clone(),
            operations: runtime.operations.clone(),
        };
        let mut plugins = PluginManager::new(definitions).unwrap();
        plugins.initialize(&mut host).await.unwrap();
        let mut ready = host.resident.as_ref().unwrap().ready();
        tokio::time::timeout(Duration::from_secs(5), ready.wait_for(|ready| *ready))
            .await
            .unwrap()
            .unwrap();
        Self {
            runtime,
            plugins,
            host,
            declaration,
        }
    }

    fn start(
        &self,
        token: &str,
        parameters: &BTreeMap<&str, dokime::core::Value>,
    ) -> tokio::task::JoinHandle<SidecarExecutionOutcome> {
        use harmos::Message;
        let cycle = dokime::core::TestCycleId::from_seed("native-cycle");
        self.host.hosting.captures.lock().unwrap().scopes.insert(
            token.to_owned(),
            dokime::core::RecordScope {
                test_cycle_id: Some(cycle.clone()),
                test_execution_id: Some(
                    dokime::core::TestExecutionId::from_uuid(token.to_owned()).unwrap(),
                ),
                origin: dokime::core::EvidenceOrigin::Initialization,
                ..Default::default()
            },
        );
        let request = RunExecutionRequest {
            mode: "automatic".into(),
            execution_token: token.into(),
            test_execution_id: token.into(),
            test_cycle_id: cycle.to_string(),
            workflow: "mini.checks".into(),
            template_identifier: "mini.checks.pulse".into(),
            parameters_json: serde_json::to_string(parameters).unwrap(),
            secrets_json: "{}".into(),
        };
        let mut context = harmos::operations::CallContext::default();
        context
            .insert_portable(ExecutionScope::execution(token))
            .unwrap();
        let operations = self.runtime.operations.with_context(context);
        tokio::spawn(async move {
            let bytes = operations
                .call_encoded(
                    <dokime::execution::Run<mini_sidecar::Pulse> as harmos::operations::OperationDeclaration>::ID,
                    request.encode_to_vec(),
                )
                .await
                .unwrap();
            SidecarExecutionOutcome::decode(bytes.as_slice()).unwrap()
        })
    }

    async fn shutdown(mut self) {
        self.plugins.shutdown(&mut self.host).await.unwrap();
        self.runtime.stop().await.unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn dispatch(operations: &harmos::OperationManager, method: &str, request: Vec<u8>) {
    let response = operations.call_encoded(method, request).await;
    match response {
        Ok(_) => {}
        Err(harmos::Error::OperationFailed { source, .. })
            if matches!(
                source.downcast_ref::<harmos::operations::NativeError>(),
                Some(
                    harmos::operations::NativeError::Remote { .. }
                        | harmos::operations::NativeError::Preparation { .. }
                )
            ) => {}
        Err(error) => panic!("{method} was advertised but did not dispatch: {error}"),
    }
}

/* === Tests === */
async fn the_inert_mini_inventory_matches_live_dispatch() {
    use harmos::Message;
    use harmos::operations::OperationDeclaration;
    let bench = Bench::new().await;
    let runtime = &bench.runtime;
    let declared = &bench.declaration;
    let mut served = Vec::new();

    // Every template of every suite serves its run and its signal.
    for template in
        dokime::catalog::entries::<mini_sidecar::suites::Suites, dokime::catalog::Suites>()
            .into_iter()
            .flat_map(|definition| definition().suite.test_case_templates)
            .map(|template| template.metadata.identifier.to_string())
    {
        let run = format!("{template}.run");
        let signal = format!("{template}.signal");
        dispatch(
            &runtime.operations,
            &run,
            RunExecutionRequest::default().encode_to_vec(),
        )
        .await;
        dispatch(
            &runtime.operations,
            &signal,
            ExecutionSignal::default().encode_to_vec(),
        )
        .await;
        served.push((
            run,
            <dokime::execution::Run<()> as OperationDeclaration>::POLICY
                .timeout
                .as_millis() as u64,
        ));
        served.push((
            signal,
            <dokime::execution::Signal<()> as OperationDeclaration>::POLICY
                .timeout
                .as_millis() as u64,
        ));
    }
    use mini_sidecar::operations::{
        connections::ListConnections, readiness::AskReadiness, status::GetStatus,
    };
    for (operation, timeout) in [
        (
            <GetStatus as harmos::operations::Declares>::Declaration::ID,
            <GetStatus as harmos::operations::Declares>::Declaration::POLICY.timeout,
        ),
        (
            <AskReadiness as harmos::operations::Declares>::Declaration::ID,
            <AskReadiness as harmos::operations::Declares>::Declaration::POLICY.timeout,
        ),
        (
            <ListConnections as harmos::operations::Declares>::Declaration::ID,
            <ListConnections as harmos::operations::Declares>::Declaration::POLICY.timeout,
        ),
    ] {
        // All three bench operations have an empty native request.
        runtime
            .operations
            .call_encoded(operation, Vec::new())
            .await
            .unwrap();
        served.push((operation.to_owned(), timeout.as_millis() as u64));
    }

    // Demand drives the real host probe; two rows cover its retained reader
    // across ticks rather than testing sysinfo in an inline unit test.
    let system = runtime
        .streams
        .publication("mini-sidecar.system", 1)
        .unwrap();
    let demand = system.watch();
    let rows = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let rows = system.query(..).await.unwrap().collect::<Vec<_>>();
            if rows.len() >= 2 {
                break rows;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the demanded host observer publishes consecutive ticks");
    let decoder = harmos::schema::Validator::new(system.definition().fields().unwrap()).unwrap();
    for row in rows {
        let mut used = None;
        decoder
            .visit(&row.value.payload, |path, event| {
                if let (
                    [
                        harmos::schema::PathElement::Field {
                            name: "system_memory_used_bytes",
                            ..
                        },
                    ],
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
    dispatch(&runtime.operations, "mini-sidecar.connection", Vec::new()).await;
    served.push(("mini-sidecar.connection".to_owned(), 30_000));
    for method in [
        "mini-sidecar.observers.start",
        "mini-sidecar.observers.stop",
    ] {
        dispatch(
            &runtime.operations,
            method,
            harmos::operations::observers::Control::default().encode_to_vec(),
        )
        .await;
        served.push((method.to_owned(), 1000));
    }
    served.sort();
    let mut inspected = declared
        .operations
        .iter()
        .map(|operation| {
            (
                operation.id.to_string(),
                operation.policy.timeout.as_millis() as u64,
            )
        })
        .collect::<Vec<_>>();
    inspected.sort();
    assert_eq!(inspected, served);
    bench.shutdown().await;
}

async fn samples_use_native_streams_and_keep_execution_context() {
    use dokime::core::Value;
    use harmos::Message;

    let mut bench = Bench::new().await;
    let parameters = BTreeMap::from([
        ("sample_count", Value::Integer(3)),
        ("tolerance", Value::from(mini_sidecar::DEFAULT_TOLERANCE)),
        ("capture_system", Value::from(false)),
    ]);
    let tokens = ["native-a", "native-b"]
        .map(|seed| dokime::core::TestExecutionId::from_seed(seed).to_string());
    let runs = tokens
        .iter()
        .map(|token| bench.start(token, &parameters))
        .collect::<Vec<_>>();

    for (token, run) in tokens.iter().zip(runs) {
        let outcome = tokio::time::timeout(Duration::from_secs(10), run)
            .await
            .expect("the run settles")
            .unwrap();
        assert!(
            outcome.passed,
            "local criteria must retain their evidence: {outcome:?}"
        );
        assert!(
            <EvidenceBatch as dokime::schema::Described>::FIELDS
                .iter()
                .all(|field| field.name != "samples")
        );
        let pages = bench
            .host
            .hosting
            .pages
            .lock()
            .unwrap()
            .get(token)
            .cloned()
            .expect("the host admitted this run's pages");
        let mut admitted = std::collections::BTreeSet::new();
        let mut completed = std::collections::BTreeSet::new();
        for page in &pages {
            for encoded in &page.step_begins {
                let start: dokime::execution::StepAttemptStart =
                    serde_json::from_str(encoded).unwrap();
                assert!(admitted.insert(start.attempt_id.clone()));
            }
            for encoded in &page.step_attempts {
                let attempt: dokime::execution::TestStepAttempt =
                    serde_json::from_str(encoded).unwrap();
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

    let stream = bench.runtime.streams.publication("mini.pulse", 1).unwrap();
    assert_eq!(
        stream.definition().fields().unwrap(),
        <mini_sidecar::suites::checks::pulse::PulseRow as harmos::schema::Described>::FIELDS
    );
    let mut samples: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for row in stream.query(..).await.unwrap() {
        assert!(row.value.published_unix_micros.is_some());
        let decoded =
            mini_sidecar::suites::checks::pulse::PulseRow::decode(&row.value.payload).unwrap();
        let index = decoded.sample_index;
        samples
            .entry(
                row.value
                    .context
                    .clone()
                    .expect("native template rows carry context"),
            )
            .or_default()
            .push(index);
    }
    assert_eq!(samples.len(), 2);
    for token in &tokens {
        let indices = &samples[token];
        assert!(indices.len() >= 3);
        assert_eq!(indices, &(0..indices.len() as i64).collect::<Vec<_>>());
    }
    bench.runtime.stop().await.unwrap();
}

async fn harmos_and_dokime_share_noncloneable_resources() {
    use dokime::__harmos::Plugin;

    let mut plugin = <MiniSidecar as Plugin>::prepare(Default::default()).unwrap();
    let first = Plugin::resources(&plugin);
    let second = Plugin::resources(&plugin);
    assert!(std::sync::Arc::ptr_eq(&first, &second));

    let weak = std::sync::Arc::downgrade(&first);
    drop((first, second));
    plugin.cleanup().await.unwrap();
    drop(plugin);
    assert!(weak.upgrade().is_none());
}

fn executable_declares_identity_and_execution_routes() {
    let declared = harmos::sidecar::Executable::at(EXECUTABLE)
        .expect("the mini-sidecar executable declares itself")
        .declaration()
        .clone();

    assert_eq!(declared.id, "mini-sidecar");
    assert_eq!(declared.version.to_string(), "0.1.0");
    let operations = declared
        .operations
        .iter()
        .map(|operation| operation.id.as_str().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        operations,
        std::collections::BTreeSet::from(
            [
                // The bench's own operations and its connection listener.
                "mini-sidecar.check_readiness",
                "mini-sidecar.connection",
                "mini-sidecar.get_status",
                "mini-sidecar.list_connections",
                "mini-sidecar.observers.start",
                "mini-sidecar.observers.stop",
                // Every template of every suite serves a run and a signal.
                "mini.charger.protection.overcurrent.run",
                "mini.charger.protection.overcurrent.signal",
                "mini.charger.recovery.power.run",
                "mini.charger.recovery.power.signal",
                "mini.charger.startup.sequence.run",
                "mini.charger.startup.sequence.signal",
                "mini.charger.steady.measurement.run",
                "mini.charger.steady.measurement.signal",
                "mini.checks.pulse.run",
                "mini.checks.pulse.signal",
            ]
            .map(str::to_owned)
        )
    );
    let streams = declared
        .streams
        .iter()
        .map(|stream| stream.id.as_str().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        streams,
        [
            "mini-sidecar.system",
            "mini.charger.protection.samples",
            "mini.charger.recovery.samples",
            "mini.charger.startup.samples",
            "mini.charger.steady.events",
            "mini.charger.steady.samples",
            "mini.pulse",
        ]
    );
    assert_eq!("mini-sidecar", declared.id);
}

#[tokio::test]
async fn native_plugin_contract() {
    the_inert_mini_inventory_matches_live_dispatch().await;
    samples_use_native_streams_and_keep_execution_context().await;
    harmos_and_dokime_share_noncloneable_resources().await;
    executable_declares_identity_and_execution_routes();
}
