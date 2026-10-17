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
    PluginArtifact, PluginBackend, PluginContributions, PluginDefinition, PluginDefinitions,
    PluginLifecycle, PluginLifecycleAuthority, PluginManager, PluginTransition, Router,
};
use harmos::sidecar::{Attach, Notice, Supervised};

// Local Imports
use dokime_domain::model;
use {
    dokime_runtime::records::PublishRecord,
    dokime_runtime::records::RecordPublication,
    model::records::publication::RecordPublishRequest,
};
use dokime_runtime::transport::{
    AskOperator, EvidenceBatch, EvidenceReceipt, LeaseAsk, LeaseResolution, LeaseStream,
    OperatorAnswer, OperatorQuestion, PushEvidence, StepAdmission,
};

/* === Constants === */
const BOUND: Duration = Duration::from_secs(5);
const SDK_BYTES: usize = 4096;
const PREPARATION_BYTES: usize = 4 * 1024 * 1024;
const HOST: &str = "fixture.host";

pub(super) const POLICY: harmos::operations::OperationPolicy =
    harmos::operations::OperationPolicy {
        timeout: BOUND,
        attempt_timeout: BOUND,
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
    };

/* === Macros === */
macro_rules! observer_control {
    ($observe:ident, $owner:literal) => {
        struct $observe;

        impl dokime_runtime::harmos::operations::OperationDeclaration for $observe {
            type Request = dokime_runtime::harmos::operations::observers::Control;
            type Response = ();
            const ID: &'static str = concat!($owner, ".observers.observe");
            const VISIBILITY: dokime_runtime::harmos::operations::Visibility =
                dokime_runtime::harmos::operations::Visibility::Host;
            const REPLAY: dokime_runtime::harmos::operations::ReplaySafety =
                dokime_runtime::harmos::operations::ReplaySafety::Idempotent;
            const POLICY: dokime_runtime::harmos::operations::OperationPolicy =
                dokime_runtime::harmos::operations::observers::POLICY;
        }

        impl dokime_runtime::harmos::operations::WireOperation for $observe {
            const VERSION: u32 = 1;
        }
    };
}
pub(super) use observer_control;

/* === Type Aliases === */
type PreparedCall<'a> =
    std::pin::Pin<Box<dyn Future<Output = harmos::Result<Box<dyn PublicationCall>>> + Send + 'a>>;

type Notices = Arc<dyn Fn(&harmos::services::Service<Fixture>, Notice) + Send + Sync>;

/* === Structs === */
pub(super) struct Fixture;

pub(super) struct Native<A: Attach> {
    pub(super) runtime: harmos::Runtime<Fixture>,
    pub(super) demands: harmos::sidecar::DemandSource,
    pub(super) host: Arc<HostLog>,
    plugins: PluginManager<(), Backend, ()>,
    lifecycle: Host<A>,
    retired: bool,
}

struct Host<A: Attach> {
    attach: A,
    notices: Notices,
    demand_keys: Vec<String>,
    demands: Option<harmos::sidecar::DemandSource>,
    operations: harmos::operations::OperationManager,
    publications: Arc<Publications>,
}

#[derive(Default)]
pub(super) struct HostLog {
    records: std::sync::Mutex<Vec<(String, RecordPublishRequest)>>,
    filed: tokio::sync::Notify,
    evidence: std::sync::Mutex<Vec<EvidenceBatch>>,
}

struct Filing(Arc<HostLog>);

struct Evidence(Arc<HostLog>);

struct Question;

struct Lease;

// The step built-in a suite plugin requires; this host watches no files.
struct Watching;

#[derive(Default)]
struct Backend {
    services: Option<harmos::operations::OperationManager>,
    lease: Option<harmos::operations::OperationInstance<harmos::operations::resident::Activation>>,
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
enum Lanes {
    #[default]
    Open,
}

/* === Traits === */

/* === Implementations === */
impl harmos::Application for Fixture {
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

impl<A: Attach> Native<A> {
    pub(super) async fn initialize(
        attach: A,
        version: &str,
        configuration: Vec<u8>,
        notices: impl Fn(&harmos::services::Service<Fixture>, Notice) + Send + Sync + 'static,
        demand_keys: Vec<String>,
    ) -> Self {
        let contributions = PluginContributions {
            operations: attach.operations().unwrap(),
            wire_contracts: attach.operation_contracts().unwrap().unwrap(),
            required_contracts: attach.required_contracts().unwrap(),
            lifecycle: attach.sdk_lifecycle().unwrap(),
            streams: attach.publications(),
            observers: attach.observers(),
            observer_control: attach.observer_control().unwrap(),
            listeners: attach.listener_declarations().unwrap().unwrap_or_default(),
            ..Default::default()
        };
        assert!(
            contributions.lifecycle.is_some(),
            "the fixture requires the original SDK lifecycle"
        );
        let definitions = PluginDefinitions::new(
            [PluginDefinition {
                id: attach.owner().into(),
                version: version.parse().unwrap(),
                artifact: PluginArtifact::LocalSidecar {
                    revision: "fixture".into(),
                },
                configuration,
                contributions,
                requires: vec![],
            }],
            [],
        )
        .unwrap();
        let runtime = harmos::Runtime::<Fixture>::builder((), ())
            .operation_settings(harmos::operations::Settings {
                preparation_bytes: Some(NonZeroUsize::new(PREPARATION_BYTES).unwrap()),
                ..Default::default()
            })
            .register(Lanes::catalog())
            .runtime()
            .await
            .unwrap();
        let log = HostLog::register(&runtime.operations);
        let mut host = Host {
            attach,
            notices: Arc::new(notices),
            demand_keys,
            demands: None,
            operations: runtime.operations.clone(),
            publications: Publications::register(&runtime.operations),
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
        self.runtime.stop().await.unwrap();
    }
}

impl<A: Attach> PluginLifecycle for Host<A> {
    type Guest = ();
    type Sidecar = Backend;
    type Embedded = ();

    fn preflight(&mut self, candidate: &PluginDefinitions) -> harmos::Result<()> {
        self.operations.validate_requirements(candidate)
    }

    fn prepare(&mut self, _: &PluginDefinition) -> harmos::Result<PluginBackend<(), Backend, ()>> {
        Ok(PluginBackend::Sidecar(Backend::default()))
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
        let notices = Arc::clone(&self.notices);
        let resident = Supervised::new(
            self.attach.clone(),
            move |service: &harmos::services::Service<Fixture>, notice: Notice| {
                notices(service, notice)
            },
        )
        .demand_keys(self.demand_keys.clone())
        .routed(router);
        self.publications.insert(&definition.id, resident.clone());
        self.demands = Some(resident.demands());
        self.operations.register_service(
            harmos::services::ServiceDefinition {
                id: definition.id.clone(),
                policy: harmos::services::POLICY,
            },
            {
                let resident = resident.clone();
                move |_: Arc<()>| resident.clone()
            },
        )?;
        backend.services = Some(self.operations.clone());
        backend.registered = true;
        backend.lease = Some(self.operations.submit_resident(&definition.id).await?);
        let native = Arc::new(resident.staged_operations().await.unwrap());
        backend.registration = Some(self.operations.stage_plugin(
            native.scope().unwrap(),
            |bindings| {
                bindings.register_native(
                    Lanes::Open,
                    Arc::clone(&native),
                    NonZeroUsize::new(128).unwrap(),
                )
            },
        )?);
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
        definition: &PluginDefinition,
        authority: &PluginLifecycleAuthority<'_>,
        backend: &mut PluginBackend<(), Backend, ()>,
    ) -> harmos::Result<()> {
        let PluginBackend::Sidecar(backend) = backend else {
            unreachable!()
        };
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
            backend
                .services
                .as_ref()
                .expect("registered service owner")
                .unregister_resident(&definition.id)
                .await?;
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

#[derive(Default)]
pub(super) struct Publications {
    residents: std::sync::RwLock<std::collections::BTreeMap<String, Arc<dyn PublicationResident>>>,
}

trait PublicationResident: Send + Sync {
    fn prepare<'a>(self: Arc<Self>, context: harmos::operations::Context<'a>) -> PreparedCall<'a>;
}

trait PublicationCall: Send {
    fn run<'a>(
        &'a mut self,
        context: harmos::operations::Context<'a>,
        request: &'a mut harmos::operations::host::Publication,
    ) -> std::pin::Pin<Box<dyn Future<Output = harmos::Result<()>> + Send + 'a>>;
}

impl<T> PublicationCall for T
where
    T: for<'a> harmos::operations::Operation<
            Context<'a> = harmos::operations::Context<'a>,
            Endpoints = (),
            Error = harmos::Error,
            Declaration = harmos::operations::host::Publish,
        >,
{
    fn run<'a>(
        &'a mut self,
        context: harmos::operations::Context<'a>,
        request: &'a mut harmos::operations::host::Publication,
    ) -> std::pin::Pin<Box<dyn Future<Output = harmos::Result<()>> + Send + 'a>> {
        Box::pin(async move {
            harmos::operations::Operation::run(self, context, request, &mut ())
                .await
                .map(|_| ())
        })
    }
}

impl<App, S, H> PublicationResident for Supervised<App, S, H>
where
    App: harmos::Application,
    S: Attach,
    H: Fn(&harmos::services::Service<App>, Notice) + Send + Sync + 'static,
{
    fn prepare<'a>(self: Arc<Self>, context: harmos::operations::Context<'a>) -> PreparedCall<'a> {
        Box::pin(async move {
            let call =
                <harmos::sidecar::Publish<App, S, H> as harmos::operations::Operation>::initialize(
                    self, context,
                )
                .await?;
            Ok(Box::new(call) as Box<dyn PublicationCall>)
        })
    }
}

impl Publications {
    pub(super) fn register(operations: &harmos::operations::OperationManager) -> Arc<Self> {
        let owner = Arc::new(Self::default());
        operations
            .register_wire_operation::<PublicationInvocation>(
                "fixture.native-publications",
                Lanes::Open,
                owner.clone(),
            )
            .unwrap();
        owner
    }

    pub(super) fn insert<App, S, H>(&self, owner: &str, resident: Supervised<App, S, H>)
    where
        App: harmos::Application,
        S: Attach,
        H: Fn(&harmos::services::Service<App>, Notice) + Send + Sync + 'static,
    {
        assert!(
            self.residents
                .write()
                .unwrap()
                .insert(owner.to_owned(), Arc::new(resident))
                .is_none()
        );
    }
}

struct PublicationInvocation {
    owner: Arc<Publications>,
    call: Option<Box<dyn PublicationCall>>,
}

impl harmos::operations::Declares for PublicationInvocation {
    type Declaration = harmos::operations::host::Publish;
}

impl harmos::operations::Operation for PublicationInvocation {
    type Declaration = harmos::operations::host::Publish;
    type Resources = Publications;
    type Error = harmos::Error;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(
        owner: Arc<Publications>,
        _: Self::Context<'ctx>,
    ) -> harmos::Result<Self> {
        Ok(Self { owner, call: None })
    }

    async fn run<'ctx>(
        &mut self,
        context: Self::Context<'ctx>,
        request: &mut harmos::operations::host::Publication,
        _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<()>> {
        let scope = context.caller_scope().ok_or(harmos::Error::InvalidInput {
            field: "fixture publication",
            reason: "original caller is required",
        })?;
        if self.call.is_none() {
            let resident = self
                .owner
                .residents
                .read()
                .unwrap()
                .get(scope.owner())
                .cloned()
                .ok_or_else(|| harmos::Error::NotRegistered {
                    domain: "fixture publication owner",
                    name: scope.owner().to_owned(),
                })?;
            self.call = Some(resident.prepare(context.clone()).await?);
        }
        self.call
            .as_mut()
            .unwrap()
            .run(context, request)
            .await
            .map(harmos::operations::Response::new)
    }
}

impl HostLog {
    fn register(operations: &harmos::operations::OperationManager) -> Arc<Self> {
        let log = Arc::new(Self::default());
        operations
            .register_wire_operation::<Filing>(HOST, Lanes::Open, Arc::clone(&log))
            .unwrap();
        operations
            .register_wire_operation::<Evidence>(HOST, Lanes::Open, Arc::clone(&log))
            .unwrap();
        operations
            .register_wire_operation::<Question>(HOST, Lanes::Open, Arc::new(()))
            .unwrap();
        operations
            .register_wire_operation::<Lease>(HOST, Lanes::Open, Arc::new(()))
            .unwrap();
        operations
            .register_wire_operation::<Watching>(HOST, Lanes::Open, Arc::new(()))
            .unwrap();
        log
    }

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

impl harmos::operations::Declares for Filing {
    type Declaration = PublishRecord;
}

impl harmos::operations::Operation for Filing {
    type Declaration = PublishRecord;
    type Resources = HostLog;
    type Error = harmos::Error;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(log: Arc<HostLog>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self(log))
    }

    async fn run<'ctx>(
        &mut self,
        context: Self::Context<'ctx>,
        publication: &mut RecordPublication,
        _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<()>> {
        let owner = context
            .caller_scope()
            .map(|scope| scope.owner().to_owned())
            .unwrap_or_default();
        let request = serde_json::from_str(&publication.request_json).map_err(|_| {
            harmos::Error::InvalidInput {
                field: "record publication",
                reason: "the publication is not a record publish request",
            }
        })?;
        self.0.records.lock().unwrap().push((owner, request));
        self.0.filed.notify_waiters();
        Ok(harmos::operations::Response::new(()))
    }
}

impl harmos::operations::Declares for Evidence {
    type Declaration = PushEvidence;
}

impl harmos::operations::Operation for Evidence {
    type Declaration = PushEvidence;
    type Resources = HostLog;
    type Error = harmos::Error;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(log: Arc<HostLog>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self(log))
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        batch: &mut EvidenceBatch,
        _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<EvidenceReceipt>> {
        let batch = std::mem::take(batch);
        let receipt = EvidenceReceipt {
            admitted: batch
                .step_begins
                .iter()
                .map(|encoded| {
                    let start: dokime_runtime::execution::StepAttemptStart =
                        serde_json::from_str(encoded).unwrap();
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
        Ok(harmos::operations::Response::new(receipt))
    }
}

impl harmos::operations::Declares for Question {
    type Declaration = AskOperator;
}

impl harmos::operations::Operation for Question {
    type Declaration = AskOperator;
    type Resources = ();
    type Error = harmos::Error;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self)
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        _: &mut OperatorQuestion,
        _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<OperatorAnswer>> {
        Err(harmos::Error::InvalidInput {
            field: "operator question",
            reason: "the fixture host has no operator",
        })
    }
}

impl harmos::operations::Declares for Lease {
    type Declaration = LeaseStream;
}

impl harmos::operations::Operation for Lease {
    type Declaration = LeaseStream;
    type Resources = ();
    type Error = harmos::Error;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self)
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        _: &mut LeaseAsk,
        _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<LeaseResolution>> {
        Ok(harmos::operations::Response::new(LeaseResolution {
            granted: false,
            message: "the fixture host measures no streams".into(),
        }))
    }
}

impl harmos::operations::Operation for Watching {
    type Declaration = dokime_runtime::operations::files::WatchFile;
    type Resources = ();
    type Error = harmos::Error;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> harmos::Result<Self> {
        Ok(Self)
    }

    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        _: &mut dokime_runtime::operations::files::WatchFile,
        _: &mut (),
    ) -> harmos::Result<harmos::operations::Response<model::files::wire::WatchFileOutcome>>
    {
        Err(harmos::Error::InvalidInput {
            field: "file watch",
            reason: "the fixture host watches no files",
        })
    }
}

/* === Public Functions === */
pub(super) fn attach<P>() -> harmos::sidecar::PluginNative<P>
where
    P: harmos::Plugin<Context = harmos::sidecar::Supervisor> + harmos::plugin::Declared,
{
    harmos::sidecar::PluginNative::<P>::new(NonZeroUsize::new(128).unwrap()).unwrap()
}

pub(super) async fn call_wire<O>(
    operations: &harmos::operations::OperationManager,
    request: O::Request,
) -> harmos::Result<O::Response>
where
    O: harmos::operations::WireOperation,
    O::Request: harmos::Message,
    O::Response: harmos::Message,
{
    let payload = operations
        .call_encoded(O::ID, harmos::Message::encode_to_vec(&request))
        .await?;
    <O::Response as harmos::Message>::decode(&payload).map_err(|source| {
        harmos::Error::backend(
            "decode fixture answer",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })
}

pub(super) fn answering<O, F>(answer: F) -> (String, dokime_runtime::execution::Answering)
where
    O: harmos::operations::Declares,
    <O::Declaration as harmos::operations::OperationDeclaration>::Request: harmos::Message,
    <O::Declaration as harmos::operations::OperationDeclaration>::Response: harmos::Message,
    F: Fn(
            <O::Declaration as harmos::operations::OperationDeclaration>::Request,
        ) -> <O::Declaration as harmos::operations::OperationDeclaration>::Response
        + Send
        + Sync
        + 'static,
{
    let answer = Arc::new(answer);
    let answering: dokime_runtime::execution::Answering = Arc::new(move |payload: Vec<u8>| {
        let answer = Arc::clone(&answer);
        Box::pin(async move {
            let request = harmos::Message::decode(payload.as_slice())
                .map_err(|error: harmos::MessageError| error.to_string())?;
            Ok(harmos::Message::encode_to_vec(&answer(request)))
        })
    });
    (
        <O::Declaration as harmos::operations::OperationDeclaration>::ID.to_owned(),
        answering,
    )
}

pub(super) fn run_request(
    token: &str,
    workflow: &str,
    template: &str,
    parameters_json: String,
) -> dokime_runtime::transport::RunExecution {
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
    workflow: &dokime_runtime::execution::Workflow<T>,
    token: &str,
    mut cursor: u64,
) -> dokime_runtime::transport::SidecarExecutionOutcome {
    loop {
        let batch = workflow.next(token, cursor).await.unwrap();
        for encoded in &batch.step_begins {
            let start: dokime_runtime::execution::StepAttemptStart = serde_json::from_str(encoded).unwrap();
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
