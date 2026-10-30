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
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

// Third Party Imports
use harmos::plugin::{
    PluginArtifact, PluginBackend, PluginContributions, PluginDefinition, PluginDefinitions, PluginLifecycle, PluginLifecycleAuthority,
    PluginManager, PluginTransition, Router,
};
use harmos::sidecar::{Attach, Notice, Supervised};
use tonic::Status;

// Local Imports
use dokime_domain::model;
use dokime_runtime::operations::files::{AwaitFileWatchArmed, WatchFile};
use dokime_runtime::operations::hosting::{Host as Hosted, Serve, Served};
use dokime_runtime::transport::{
    EvidenceBatch, EvidenceReceipt, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, StepAdmission,
};
use {dokime_runtime::records::RecordPublication, model::records::publication::RecordPublishRequest};
/* === Constants === */
const BOUND: Duration = Duration::from_secs(120);
// Initialization carries the configuration and the host's operation assignments.
const SDK_BYTES: usize = 64 * 1024;
const PREPARATION_BYTES: usize = 4 * 1024 * 1024;

pub(super) const POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    timeout: BOUND,
    attempt_timeout: BOUND,
    cancellation_timeout: std::time::Duration::ZERO,
    retries: harmos::operations::RetryPolicy::Never,
};

/* === Macros === */
/* === Type Aliases === */
type Notices = Arc<dyn Fn(&harmos::ServiceContext<Fixture>, Notice) + Send + Sync>;

pub(super) type Operations = harmos::operations::OperationManager<Lanes>;

/* === Structs === */
pub(super) struct Fixture;

pub(super) struct Native<A: Attach> {
    pub(super) runtime: harmos::Runtime<Fixture>,
    pub(super) demands: harmos::sidecar::DemandSource,
    pub(super) host: Arc<HostLog>,
    plugins: PluginManager<Backend, ()>,
    lifecycle: Host<A>,
    retired: bool,
}

struct Host<A: Attach> {
    attach: A,
    notices: Notices,
    demand_keys: Vec<String>,
    demands: Option<harmos::sidecar::DemandSource>,
    operations: Operations,
}

#[derive(Default)]
pub(super) struct HostLog {
    records: std::sync::Mutex<Vec<(String, RecordPublishRequest)>>,
    filed: tokio::sync::Notify,
    evidence: std::sync::Mutex<Vec<EvidenceBatch>>,
}

// The host side of the operations a suite plugin calls back into: it files
// records and evidence, has no operator, grants no leases and watches no files.
struct Hosting(Arc<HostLog>);

#[derive(Default)]
struct Backend {
    services: Option<Operations>,
    lease: Option<harmos::operations::Running<harmos::operations::resident::Activation>>,
    registration: Option<harmos::operations::PluginRegistration>,
    sdk: Option<harmos::operations::PluginSdkRegistration>,
    initializing: Option<harmos::operations::EncodedInvocation>,
    shutting_down: Option<harmos::operations::EncodedInvocation>,
    registered: bool,
    initialized: bool,
    shutdown_attempted: bool,
}

/* === Enums === */
#[harmos::lanes]
pub(super) enum Lanes {
    #[default]
    Open,
}

/* === Traits === */

/* === Implementations === */
impl harmos::Application for Fixture {
    type Lanes = Lanes;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = ();
    type Streams = ();
}

#[derive(Default, serde::Deserialize)]
pub(super) struct FixtureConfig {
    #[serde(skip)]
    log: Option<Arc<HostLog>>,
}

#[harmos::catalog]
enum HostOperations {
    Records(RecordPublication),
    Evidence(EvidenceBatch),
    Questions(OperatorQuestion),
    Lease(LeaseAsk),
    Watch(WatchFile),
    Armed(AwaitFileWatchArmed),
    Publish(harmos::builtins::streams::Publish),
}

impl harmos::Lifecycle for Fixture {
    type Config = FixtureConfig;
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;
    async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().operation_settings(harmos::operations::Settings {
            preparation_bytes: Some(NonZeroUsize::new(PREPARATION_BYTES).unwrap()),
            ..Default::default()
        });
        let hosting = Arc::new(Hosting(config.log.expect("fixture host log")));
        setup.resource::<Hosted<RecordPublication>>(Hosted::<RecordPublication>::new(hosting.clone()));
        setup.resource::<Hosted<EvidenceBatch>>(Hosted::<EvidenceBatch>::new(hosting.clone()));
        setup.resource::<Hosted<OperatorQuestion>>(Hosted::<OperatorQuestion>::new(hosting.clone()));
        setup.resource::<Hosted<LeaseAsk>>(Hosted::<LeaseAsk>::new(hosting.clone()));
        setup.resource::<Hosted<WatchFile>>(Hosted::<WatchFile>::new(hosting.clone()));
        setup.resource::<Hosted<AwaitFileWatchArmed>>(Hosted::<AwaitFileWatchArmed>::new(hosting.clone()));
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

impl<A: Attach> Native<A> {
    pub(super) async fn initialize(
        attach: A, version: &str, configuration: Vec<u8>,
        notices: impl Fn(&harmos::ServiceContext<Fixture>, Notice) + Send + Sync + 'static, demand_keys: Vec<String>,
    ) -> Self {
        let contributions = PluginContributions {
            operations: attach.operations().unwrap(),
            wire_contracts: attach.operation_contracts().unwrap().unwrap(),
            required_contracts: attach.required_contracts().unwrap(),
            lifecycle: attach.sdk_lifecycle().unwrap(),
            streams: attach.publications(),
            observers: attach.observers(),
            listeners: attach.listener_declarations().unwrap().unwrap_or_default(),
            ..Default::default()
        };
        assert!(contributions.lifecycle.is_some(), "the fixture requires the original SDK lifecycle");
        let definitions = PluginDefinitions::new(
            [PluginDefinition {
                id: attach.owner().into(),
                version: version.parse().unwrap(),
                artifact: PluginArtifact::LocalSidecar { revision: "fixture".into() },
                configuration,
                contributions,
                requires: vec![],
            }],
            [],
        )
        .unwrap();
        let log = Arc::new(HostLog::default());
        let runtime = harmos::Runtime::<Fixture>::launch((), FixtureConfig { log: Some(log.clone()) }).await.unwrap();
        let mut host = Host {
            attach,
            notices: Arc::new(notices),
            demand_keys,
            demands: None,
            operations: runtime.operations.clone(),
        };
        let mut plugins = PluginManager::new(definitions).unwrap();
        plugins.initialize(&mut host).await.unwrap();
        let demands = host.demands.as_ref().unwrap().clone();
        Self {
            runtime,
            demands,
            host: log,
            plugins,
            lifecycle: host,
            retired: false,
        }
    }

    pub(super) async fn retire(&mut self) {
        if !self.retired {
            self.plugins.shutdown(&mut self.lifecycle).await.unwrap();
            self.retired = true;
        }
    }

    pub(super) async fn shutdown(mut self) {
        self.retire().await;
        self.runtime.shutdown().await.unwrap();
    }
}

impl<A: Attach> PluginLifecycle for Host<A> {
    type Sidecar = Backend;
    type Embedded = ();

    fn preflight(&mut self, candidate: &PluginDefinitions) -> harmos::Result<()> {
        self.operations.validate_requirements(candidate)
    }

    fn prepare(&mut self, _: &PluginDefinition) -> harmos::Result<PluginBackend<Backend, ()>> {
        Ok(PluginBackend::Sidecar(Backend::default()))
    }

    async fn fence(&mut self, _: PluginTransition, _: &[PluginDefinition]) -> harmos::Result<()> {
        Ok(())
    }

    async fn start(
        &mut self, definition: &PluginDefinition, router: &Router, authority: &PluginLifecycleAuthority<'_>,
        backend: &mut PluginBackend<Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else { unreachable!() };
        let notices = Arc::clone(&self.notices);
        let resident =
            Supervised::new(self.attach.clone(), move |service: &harmos::ServiceContext<Fixture>, notice: Notice| notices(service, notice))
                .demand_keys(self.demand_keys.clone())
                .routed(router);
        self.demands = Some(resident.demands());
        self.operations.register_service(
            harmos::services::ServiceDefinition {
                id: definition.id.clone(),
                policy: harmos::services::POLICY,
            },
            {
                let resident = resident.clone();
                move |_: Arc<harmos::ExecutionContext>| resident.clone()
            },
        )?;
        backend.services = Some(self.operations.clone());
        backend.registered = true;
        backend.lease = Some(self.operations.submit_resident(&definition.id).await?);
        let native = Arc::new(resident.staged_operations().await.unwrap());
        backend.registration = Some(self.operations.stage_plugin(native.scope().unwrap(), |bindings| {
            bindings.register_native(Lanes::Open, Arc::clone(&native), NonZeroUsize::new(128).unwrap())
        })?);
        backend.sdk = Some(native.stage_sdk(&self.operations, Lanes::Open)?);
        backend.initializing = Some(backend.sdk.as_ref().unwrap().submit_initialize(
            authority,
            &definition.configuration,
            NonZeroUsize::new(SDK_BYTES).unwrap(),
        )?);
        let outcome = backend.initializing.as_mut().unwrap().await;
        backend.initializing.take();
        outcome?;
        backend.initialized = true;
        Ok(())
    }

    async fn stop(
        definition: &PluginDefinition, authority: &PluginLifecycleAuthority<'_>, backend: &mut PluginBackend<Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else { unreachable!() };
        if let Some(sdk) = &backend.sdk {
            sdk.settle().await?;
            if let Some(initializing) = &mut backend.initializing {
                let outcome = initializing.await;
                backend.initializing.take();
                outcome?;
                backend.initialized = true;
            }
            if backend.initialized && !backend.shutdown_attempted {
                backend.shutting_down = Some(sdk.submit_shutdown(authority)?);
                backend.shutdown_attempted = true;
            }
            if let Some(shutdown) = &mut backend.shutting_down {
                let outcome = shutdown.await;
                backend.shutting_down.take();
                outcome?;
                backend.initialized = false;
            }
        }
        if let Some(registration) = &mut backend.registration {
            registration.shutdown().await?;
        }
        if let Some(lease) = backend.lease.take() {
            lease.release().await?;
        }
        if backend.registered {
            backend.services.as_ref().expect("registered service owner").unregister_resident(&definition.id).await?;
        }
        backend.registered = false;
        if let Some(sdk) = &mut backend.sdk {
            sdk.close().await?;
        }
        backend.sdk.take();
        backend.registration.take();
        Ok(())
    }
}

impl HostLog {
    // Registers the host operations a suite plugin calls and Harmos's own
    // `streams.publish` built-in, which reaches the calling sidecar natively.
    pub(super) fn records(&self) -> Vec<(String, RecordPublishRequest)> {
        self.records.lock().unwrap().clone()
    }

    // Resolves once the host has filed at least one record.
    pub(super) async fn first_filed(&self) {
        loop {
            let filed = self.filed.notified();
            tokio::pin!(filed);
            filed.as_mut().enable();
            if !self.records.lock().unwrap().is_empty() {
                return;
            }
            filed.await;
        }
    }

    pub(super) fn evidence(&self) -> Vec<EvidenceBatch> {
        self.evidence.lock().unwrap().clone()
    }
}

impl Serve<RecordPublication> for Hosting {
    fn serve<'a>(&'a self, publication: RecordPublication, ctx: harmos::Context<'a, RecordPublication>) -> Served<'a, ()> {
        Box::pin(async move {
            let owner = ctx
                .resource::<harmos::operations::HostContext>()
                .borrow()
                .caller_scope()
                .map(|scope| scope.owner().to_owned())
                .unwrap_or_default();
            let request = serde_json::from_str(&publication.request_json)
                .map_err(|_| Status::invalid_argument("the publication is not a record publish request"))?;
            self.0.records.lock().unwrap().push((owner, request));
            self.0.filed.notify_waiters();
            Ok(().into())
        })
    }
}

impl Serve<EvidenceBatch> for Hosting {
    fn serve<'a>(&'a self, batch: EvidenceBatch, _: harmos::Context<'a, EvidenceBatch>) -> Served<'a, EvidenceReceipt> {
        Box::pin(async move {
            let receipt = EvidenceReceipt {
                admitted: batch
                    .step_begins
                    .iter()
                    .map(|encoded| {
                        let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
                        StepAdmission {
                            attempt_id: start.attempt_id.to_string(),
                            accepted: true,
                            message: String::new(),
                        }
                    })
                    .collect(),
                settled: batch.record_subscriptions_settling,
            };
            self.0.evidence.lock().unwrap().push(batch);
            Ok(receipt.into())
        })
    }
}

impl Serve<OperatorQuestion> for Hosting {
    fn serve<'a>(&'a self, _: OperatorQuestion, _: harmos::Context<'a, OperatorQuestion>) -> Served<'a, OperatorAnswer> {
        Box::pin(async { Err(Status::failed_precondition("the fixture host has no operator")) })
    }
}

impl Serve<LeaseAsk> for Hosting {
    fn serve<'a>(&'a self, _: LeaseAsk, _: harmos::Context<'a, LeaseAsk>) -> Served<'a, LeaseResolution> {
        Box::pin(async {
            Ok(LeaseResolution {
                granted: false,
                message: "the fixture host measures no streams".into(),
            }
            .into())
        })
    }
}

impl Serve<WatchFile> for Hosting {
    fn serve<'a>(&'a self, _: WatchFile, _: harmos::Context<'a, WatchFile>) -> Served<'a, model::files::wire::WatchFileOutcome> {
        Box::pin(async { Err(Status::failed_precondition("the fixture host watches no files")) })
    }
}

impl Serve<AwaitFileWatchArmed> for Hosting {
    fn serve<'a>(
        &'a self, _: AwaitFileWatchArmed, _: harmos::Context<'a, AwaitFileWatchArmed>,
    ) -> Served<'a, model::files::wire::FileWatchArmed> {
        Box::pin(async { Err(Status::failed_precondition("the fixture host watches no files")) })
    }
}

/* === Public Functions === */
pub(super) fn attach<P>() -> harmos::sidecar::PluginNative<P>
where
    P: harmos::Plugin<Context = harmos::sidecar::Supervisor> + harmos::plugin::Declared,
{
    harmos::sidecar::PluginNative::<P>::new(NonZeroUsize::new(128).unwrap()).unwrap()
}
// A sidecar serves its operations on the wire under `<plugin id>.<operation id>`.
pub(super) async fn call_wire<O>(operations: &Operations, owner: &str, request: O::Request) -> harmos::Result<O::Response>
where
    O: harmos::operations::OperationDeclaration,
{
    let payload = operations.call_encoded(&format!("{owner}.{}", O::ID), <O::Request as harmos::Message>::encode(&request)).await?;
    <O::Response as harmos::Message>::decode(&payload)
        .map_err(|source| harmos::Error::backend("decode fixture answer", harmos::BackendFailure::Corrupt, source))
}

pub(super) fn answering<O, F>(answer: F) -> (String, dokime_runtime::execution::Answering)
where
    O: harmos::operations::OperationDeclaration,
    F: Fn(O::Request) -> O::Response + Send + Sync + 'static,
{
    let answer = Arc::new(answer);
    let answering: dokime_runtime::execution::Answering = Arc::new(move |payload: Vec<u8>| {
        let answer = Arc::clone(&answer);
        Box::pin(async move {
            let request = <O::Request as harmos::Message>::decode(payload.as_slice()).map_err(|error| error.to_string())?;
            Ok(<O::Response as harmos::Message>::encode(&answer(request)))
        })
    });
    (O::ID.to_owned(), answering)
}

pub(super) fn run_request(token: &str, workflow: &str, template: &str, parameters_json: String) -> dokime_runtime::transport::RunExecution {
    dokime_runtime::transport::RunExecution {
        mode: "automatic".into(),
        execution_token: token.to_owned(),
        test_cycle_id: "cycle".into(),
        test_execution_id: token.to_owned(),
        template_identifier: template.to_owned(),
        workflow: workflow.to_owned(),
        secrets_json: "{}".into(),
        parameters_json,
    }
}

pub(super) async fn settled<T: dokime_runtime::execution::SidecarTemplate>(
    workflow: &dokime_runtime::execution::Workflow<T>, token: &str, mut cursor: u64,
) -> dokime_runtime::transport::SidecarExecutionOutcome {
    loop {
        let batch = workflow.next(token, cursor).await.unwrap();
        for encoded in &batch.step_begins {
            let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            workflow.admit(
                token,
                &StepAdmission {
                    attempt_id: start.attempt_id.to_string(),
                    accepted: true,
                    message: String::new(),
                },
            );
        }
        cursor = batch.cursor;
        if let Some(outcome) = batch.outcome {
            return outcome;
        }
    }
}

/* === Private Functions === */

/* === Tests === */
