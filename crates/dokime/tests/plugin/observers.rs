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
    Arc, LazyLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::time::Duration;
// Third Party Imports
use dokime::harmos::operations::{Host as _, ServingError, WireTable};
use dokime::harmos::plugin::contract::Declared;
use dokime::harmos::plugin::sidecar::{ObserverAttachment, Publisher};
use tokio_stream::StreamExt;
// Local Imports
use dokime::core::Value;
use dokime::execution::{
    CriterionContext, InitializationContext, ObserverContext, StepContext, Workflow,
};
use dokime::plugin::Configuration;
use dokime::plugin::testing::Harness;
use dokime::{Refusal, Supervisor};

/* === Constants === */
const SUITE: &str = "observer.checks";

static PROBES: LazyLock<std::sync::Mutex<BTreeMap<String, Arc<Probe>>>> =
    LazyLock::new(|| std::sync::Mutex::new(BTreeMap::new()));

/* === Macros === */
super::native::observer_control!(OwnerObserveControl, "observer-fixture");

/* === Type Aliases === */
type Publishing = Publisher<Owner>;

/* === Structs === */
#[derive(Default)]
struct Probe {
    starts: AtomicUsize,
    checks: AtomicUsize,
    credentials: AtomicBool,
    credential_ticks: AtomicUsize,
    stops: AtomicUsize,
    cleaned: AtomicBool,
    failed: AtomicUsize,
    entered: tokio::sync::Notify,
    dropped: AtomicBool,
    attempt_cleaned: AtomicBool,
    blocking: AtomicBool,
    release_blocking: AtomicBool,
    starved: AtomicBool,
}

#[dokime::section(
    label = "Probe",
    description = "The token a run names its registered [`Probe`] by, standing in for the live handle a setting cannot carry across the wire."
)]
pub struct Probes {
    #[setting(
        label = "Probe Token",
        default = "unregistered",
        description = "Never read as its declared default: every run in this file names a real registered token, but a required text setting still has to declare a nonempty one to pass catalog validation."
    )]
    pub token: String,
}

#[dokime::settings]
pub struct OwnerSettings {
    pub probes: Probes,
}

#[dokime::section(label = "Required Credentials")]
struct RequiredCredentials {
    #[setting(label = "Token", from_environment = "OBSERVER_FIXTURE_TOKEN")]
    token: dokime::plugin::Secret,
}

#[dokime::settings]
struct MissingCredentialSettings {
    credentials: RequiredCredentials,
}

struct MissingCredentials;

struct Shared {
    probe: Arc<Probe>,
    token: String,
    credentials: bool,
}

struct Owner {
    shared: Arc<Shared>,
    observers: Option<Arc<ObserverAttachment<Shared, Publishing>>>,
}

#[dokime::stream(id = "observer-fixture.status", label = "Observer Status", window = 64)]
pub struct ObservationRow {
    index: u64,
}

struct Watch(Arc<Probe>);

#[derive(Default)]
struct StatelessWatch;

struct GetCount(Arc<Shared>);

struct GetProbe(Arc<Shared>);

struct Checks;

struct AwaitEvent;

struct AttemptResources(tokio::sync::Notify);

struct Faults(Arc<Probe>);

struct Pending(Arc<Probe>);

struct PendingGuard(Arc<Probe>);

#[dokime::record(id = "observer.suite_observed", label = "Suite Observed")]
struct SuiteObserved {
    index: u64,
}

#[dokime::record(id = "observer.observed", label = "Observed")]
pub struct Observed {
    index: u64,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
struct GetCountResponse {
    checks: u64,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
struct GetProbeResponse {
    token: String,
    credentials: bool,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum WatchError {
    #[error(transparent)]
    Publish(#[from] dokime::harmos::plugin::sidecar::PublishError),
    #[error(transparent)]
    Call(#[from] dokime::harmos::plugin::sidecar::CallError),
    #[error(transparent)]
    Sdk(#[from] dokime::Error),
}

#[dokime::catalog]
enum ObserveRecords {
    Observed(Observed),
}

#[dokime::catalog]
enum Implemented {
    GetCount(GetCount),
    GetProbe(GetProbe),
}

#[dokime::catalog]
enum Operations {
    #[nested]
    Implemented(Implemented),
    Observe(OwnerObserveControl),
}

#[dokime::catalog]
enum OwnerObservers {
    Watch(Watch),
    StatelessWatch(StatelessWatch),
}

#[dokime::catalog]
pub enum Records {
    Observed(Observed),
}

#[dokime::catalog]
enum OwnerSuites {
    Checks(Checks),
}

#[dokime::catalog]
enum ChecksTemplates {
    AwaitEvent(AwaitEvent),
    Faults(Faults),
    Pending(Pending),
}

#[dokime::catalog]
enum ChecksRecords {
    SuiteObserved(SuiteObserved),
}

#[dokime::catalog]
enum AwaitEventRequired {
    GetCount(GetCount),
}

#[dokime::catalog]
enum ProbeRequired {
    GetProbe(GetProbe),
}

/* === Traits === */

/* === Implementations === */
#[dokime::operation(id = "observer-fixture.get_count", version = 1, policy = super::native::POLICY)]
impl GetCount {
    async fn initialize(shared: Arc<Shared>, _: &Supervisor) -> Self {
        Self(shared)
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<GetCountResponse, Refusal> {
        Ok(GetCountResponse {
            checks: self.0.probe.checks.load(Ordering::SeqCst) as u64,
        })
    }
}

#[dokime::operation(id = "observer-fixture.get_probe", version = 1, policy = super::native::POLICY)]
impl GetProbe {
    async fn initialize(shared: Arc<Shared>, _: &Supervisor) -> Self {
        Self(shared)
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<GetProbeResponse, Refusal> {
        Ok(GetProbeResponse {
            token: self.0.token.clone(),
            credentials: self.0.credentials,
        })
    }
}

impl dokime::harmos::operations::lifecycle::Policy for Owner {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime::plugin(
    id = "observer-fixture",
    provides = Operations,
    requires(),
    contributions(
        dokime::Suites<OwnerSuites>,
        dokime::Records<Records>,
        dokime::Settings<OwnerSettings>,
    ),
    streams(ObservationRow),
    observers = OwnerObservers,
    observer_control = OwnerObserveControl,
    restart = "never", description = "Observer Lifecycle Fixture.")]
impl dokime::Plugin for Owner {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Shared;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<OwnerSettings>())?;
        Ok(Self {
            shared: Arc::new(Shared {
                probe: probe(&settings.probes.token),
                token: settings.probes.token,
                credentials: config.credentials_configured::<OwnerSettings>(),
            }),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<OwnerObservers>(
            self.resources(),
            <Self as Declared>::STREAMS,
        )?);
        Ok(())
    }

    fn resources(&self) -> Arc<Shared> {
        Arc::clone(&self.shared)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Implemented, _>(self.resources())?;
        dokime::execution::serve::<Self, OwnerSuites>(table)?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<OwnerObserveControl>(table)
    }

    fn attachment(
        &self,
    ) -> Option<Arc<dyn dokime::harmos::plugin::contract::Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| {
            Arc::clone(observers)
                as Arc<dyn dokime::harmos::plugin::contract::Attachment<Supervisor>>
        })
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        let probe = &self.shared.probe;
        assert_eq!(
            probe.starts.load(Ordering::SeqCst),
            probe.stops.load(Ordering::SeqCst),
            "observers finish their stop hooks before plugin cleanup"
        );
        probe.cleaned.store(true, Ordering::SeqCst);
        Ok(())
    }
}

#[dokime::observer(interval_ms = 5)]
impl StatelessWatch {
    async fn observe(
        &mut self,
        ctx: &dokime::harmos::plugin::sidecar::ObserverContext<'_, Shared, Publishing>,
    ) -> Result<(), WatchError> {
        let _ = (ctx.resources(), ctx.tick());
        Ok(())
    }
}

#[dokime::observer(interval_ms = 5, streams(ObservationRow))]
impl Watch {
    async fn initialize(shared: &Shared) -> Result<Self, WatchError> {
        shared.probe.starts.fetch_add(1, Ordering::SeqCst);
        Ok(Self(Arc::clone(&shared.probe)))
    }

    async fn observe(
        &mut self,
        ctx: &dokime::harmos::plugin::sidecar::ObserverContext<'_, Shared, Publishing>,
    ) -> Result<(), WatchError> {
        let host = ctx.capabilities().supervisor();
        let credentials = host
            .call_as::<GetProbeDeclaration>(())
            .await
            .map(|response| response.into_owned())?
            .credentials;
        self.0.credentials.store(credentials, Ordering::Release);
        self.0.credential_ticks.fetch_add(1, Ordering::Release);
        let index = ctx.tick().ordinal() - 1;
        ctx.publish_on::<ObservationRow>(&ObservationRow { index })
            .await?;
        dokime::plugin::Publisher::new(host)
            .publish(Observed { index })
            .await?;
        self.0.checks.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), WatchError> {
        self.0.stops.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl dokime::harmos::operations::lifecycle::Policy for MissingCredentials {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime::plugin(
    id = "missing-observer-credentials",
    provides = (),
    requires(),
    contributions(dokime::Settings<MissingCredentialSettings>),
    restart = "never", description = "A plugin whose card requires a secret nobody bound.")]
impl dokime::Plugin for MissingCredentials {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<MissingCredentialSettings>())?;
        let _ = settings.credentials.token;
        Ok(Self)
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, _: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

#[dokime::suite(
    id = SUITE,
    label = "Observer Checks",
    templates = ChecksTemplates,
    records = ChecksRecords
)]
impl Checks {}

#[dokime::template(
    id = "observer.checks.await",
    label = "Await Event",
    requires = AwaitEventRequired
)]
impl AwaitEvent {
    async fn initialize(
        _ctx: &mut InitializationContext<'_>,
    ) -> dokime::Result<(Self, AttemptResources)> {
        let template = Self;
        let shared = AttemptResources(tokio::sync::Notify::new());
        Ok((template, shared))
    }

    #[dokime::step(label = "Hear The Event")]
    async fn hear(&mut self, ctx: &mut StepContext<'_>) -> dokime::Result<()> {
        // An operation is the only road a step has to the plugin. Its type
        // belongs to the required catalog declared by this template.
        let counted = ctx.call::<GetCount>(()).await?;
        let _ = counted.checks;
        ctx.publish(Observed { index: 999 }).await;
        ctx.resources().0.notify_one();
        let observed = ctx
            .listen::<Observed>()
            .next()
            .await
            .expect("the observer publishes");
        assert_eq!(
            observed.index, 0,
            "a step does not hear its own publication"
        );
        ctx.publish(Observed { index: 777 }).await;
        assert!(
            tokio::time::timeout(
                Duration::from_millis(10),
                ctx.listen::<Observed>().skip(1).next()
            )
            .await
            .is_err(),
            "the observer event is delivered only once"
        );
        Ok(())
    }

    #[dokime::criterion(
        id = "observer.checks.observed",
        label = "Event Recorded",
        description = "A criterion calls too, which is what makes an operation reachable from every stage rather than from the ones that hold handles."
    )]
    async fn observed(ctx: &CriterionContext<'_>) -> dokime::Result<bool> {
        let counted = ctx.call::<GetCount>(()).await?;
        ctx.info(format!("the plugin has answered {} checks", counted.checks));

        Ok(ctx
            .records::<Observed>()?
            .iter()
            .map(|record| record.index)
            .collect::<Vec<_>>()
            == [999, 0, 777])
    }

    #[dokime::observer(interval_ms = 5, records = ObserveRecords)]
    async fn observe(ctx: &mut ObserverContext<'_>) -> dokime::Result<()> {
        if ctx.tick().index() == 0 {
            ctx.resources().0.notified().await;
            let index = ctx.tick().index();
            ctx.publish(Observed { index }).await;
        }
        Ok(())
    }
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        self.0.dropped.store(true, Ordering::SeqCst);
    }
}

#[dokime::template(
    id = "observer.checks.faults",
    label = "Observer Faults",
    requires = ProbeRequired
)]
impl Faults {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> dokime::Result<(Self, Arc<Probe>)> {
        let template = Self(probe(&ctx.call::<GetProbe>(()).await?.token));
        let shared = Arc::clone(&template.0);
        Ok((template, shared))
    }

    #[dokime::step(label = "Wait For Independent Observers")]
    async fn proceed(&mut self, ctx: &mut StepContext<'_>) -> dokime::Result<()> {
        ctx.listen::<SuiteObserved>()
            .next()
            .await
            .expect("the healthy observer continues");
        self.0.release_blocking.store(true, Ordering::SeqCst);
        while self.0.failed.load(Ordering::SeqCst) < 2 {
            tokio::task::yield_now().await;
        }
        Ok(())
    }

    #[dokime::observer(interval_ms = 5)]
    async fn fails(ctx: &mut ObserverContext<'_>) -> Result<(), std::io::Error> {
        ctx.resources().failed.fetch_add(1, Ordering::SeqCst);
        Err(std::io::Error::other("observer check refused"))
    }

    #[dokime::observer(interval_ms = 5)]
    async fn panics(ctx: &mut ObserverContext<'_>) -> dokime::Result<()> {
        ctx.resources().failed.fetch_add(1, Ordering::SeqCst);
        panic!("observer check panicked");
    }

    #[dokime::observer(interval_ms = 5)]
    async fn healthy(ctx: &mut ObserverContext<'_>) -> dokime::Result<()> {
        if !ctx.resources().blocking.load(Ordering::SeqCst) {
            return Ok(());
        }
        let index = ctx.tick().index();
        ctx.publish(SuiteObserved { index }).await;
        Ok(())
    }

    #[dokime::observer(interval_ms = 5)]
    async fn blocking(ctx: &mut ObserverContext<'_>) -> dokime::Result<()> {
        let probe = ctx.resources();
        probe.blocking.store(true, Ordering::SeqCst);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !probe.release_blocking.load(Ordering::SeqCst) {
            if std::time::Instant::now() >= deadline {
                probe.starved.store(true, Ordering::SeqCst);
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}

#[dokime::template(
    id = "observer.checks.pending",
    label = "Pending Observer",
    requires = ProbeRequired
)]
impl Pending {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> dokime::Result<(Self, Arc<Probe>)> {
        let template = Self(probe(&ctx.call::<GetProbe>(()).await?.token));
        let shared = Arc::clone(&template.0);
        Ok((template, shared))
    }

    #[dokime::step(label = "Wait For Cancellation")]
    async fn wait(&mut self, _: &mut StepContext<'_>) -> dokime::Result<()> {
        std::future::pending().await
    }

    #[dokime::observer(interval_ms = 5)]
    async fn blocked(ctx: &mut ObserverContext<'_>) -> dokime::Result<()> {
        let resources = ctx.resources();
        let _guard = PendingGuard(Arc::clone(resources));
        resources.entered.notify_one();
        std::future::pending().await
    }

    async fn cleanup(
        &mut self,
        _: &mut dokime::execution::CleanupContext<'_>,
    ) -> dokime::Result<()> {
        assert!(
            self.0.dropped.load(Ordering::SeqCst),
            "pending check dropped before cleanup"
        );
        self.0.attempt_cleaned.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn registered(probe: &Arc<Probe>) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let token = format!("probe-{}", NEXT.fetch_add(1, Ordering::SeqCst));
    PROBES
        .lock()
        .expect("the probe registry is never poisoned")
        .insert(token.clone(), Arc::clone(probe));

    token
}

fn probe(token: &str) -> Arc<Probe> {
    Arc::clone(
        PROBES
            .lock()
            .expect("the probe registry is never poisoned")
            .get(token)
            .expect("a run names a probe this file registered"),
    )
}

fn answered(token: String) -> impl Fn(()) -> GetProbeResponse + Send + Sync + 'static {
    move |()| GetProbeResponse {
        token: token.clone(),
        credentials: true,
    }
}

/* === Tests === */

pub(crate) async fn activation_starts_execution() {
    let probe = Arc::new(Probe::default());
    let configuration = Configuration {
        settings: BTreeMap::from([("token".to_owned(), Value::Text(registered(&probe)))]),
        ..Configuration::default()
    };
    let native = super::native::Native::initialize(
        super::native::attach::<Owner>(),
        <Owner as Declared>::VERSION,
        serde_json::to_vec(&configuration).unwrap(),
        |_, _| {},
        vec![],
    )
    .await;
    tokio::time::timeout(Duration::from_secs(5), async {
        while native.host.records().is_empty() {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("an observer publishes without any demand");
    let (owner, filed) = native.host.records().remove(0);
    assert_eq!(owner, "observer-fixture");
    assert_eq!(filed.record_type.as_str(), "observer.observed");

    let Err(refused) = <MissingCredentials as dokime::Plugin>::prepare(Configuration::declared::<
        MissingCredentialSettings,
    >()) else {
        panic!("the required credential is absent");
    };
    assert!(refused.to_string().contains("token"), "{refused}");
    let previous = probe.credential_ticks.load(Ordering::Acquire);
    tokio::time::timeout(Duration::from_secs(5), async {
        while probe.credential_ticks.load(Ordering::Acquire) <= previous {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("the original observer ticks after another preparation fails");
    assert!(probe.credentials.load(Ordering::Acquire));
    assert_eq!(probe.starts.load(Ordering::SeqCst), 1);
    native.shutdown().await;
    assert_eq!(probe.stops.load(Ordering::SeqCst), 1);
    assert!(probe.cleaned.load(Ordering::SeqCst));
    let settled = probe.checks.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(15)).await;
    assert_eq!(probe.checks.load(Ordering::SeqCst), settled);
}

pub(crate) async fn template_events_criterion() {
    let probe = Arc::new(Probe::default());
    let checks = Arc::clone(&probe);
    let report = Harness::template("observer.checks.await")
        .timeout(Duration::from_secs(5))
        .answering::<GetCount, _>(move |()| GetCountResponse {
            checks: checks.checks.load(Ordering::SeqCst) as u64,
        })
        .run::<OwnerSuites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.criteria().len(), 1);
    let records = report.records::<Observed>();
    assert_eq!(
        records
            .iter()
            .map(|record| record.index)
            .collect::<Vec<_>>(),
        [999, 0, 777]
    );
    assert_eq!(
        probe.starts.load(Ordering::SeqCst),
        0,
        "a template run starts no plugin observer"
    );
}

pub(crate) async fn observer_failure_event() {
    let probe = Arc::new(Probe::default());
    let report = Harness::template("observer.checks.faults")
        .timeout(Duration::from_secs(5))
        .answering::<GetProbe, _>(answered(registered(&probe)))
        .run::<OwnerSuites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    let diagnostics = report.diagnostics();
    assert!(
        diagnostics.contains("observer check refused"),
        "{diagnostics}"
    );
    assert!(
        diagnostics.contains("observer check panicked"),
        "{diagnostics}"
    );
    assert_eq!(probe.failed.load(Ordering::SeqCst), 2);
    assert!(
        !probe.starved.load(Ordering::SeqCst),
        "a blocking observer starved its sibling or the step"
    );
    assert!(!report.records::<SuiteObserved>().is_empty());
}

pub(crate) async fn cancellation_drops_cleanup() {
    let probe = Arc::new(Probe::default());
    let workflow = Workflow::<Pending>::new(SUITE, "observer-fixture");
    workflow
        .start_on(
            super::native::run_request("pending", SUITE, "observer.checks.pending", "{}".into()),
            None,
            dokime::execution::Caller::local(BTreeMap::from([super::native::answering::<
                GetProbe,
                _,
            >(answered(
                registered(&probe),
            ))])),
        )
        .unwrap();
    let mut cursor = 0;
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                () = probe.entered.notified() => break,
                () = tokio::time::sleep(Duration::from_millis(2)) => {
                    let batch = workflow.drain("pending", cursor).unwrap();
                    cursor = batch.cursor;
                    for encoded in &batch.step_begins {
                        let start: dokime::execution::StepAttemptStart =
                            serde_json::from_str(encoded).unwrap();
                        workflow.admit("pending", &dokime::transport::StepAdmission {
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
    .expect("the pending observer is entered");
    workflow.cancel("pending");
    let outcome = super::native::settled(&workflow, "pending", cursor).await;
    assert!(!outcome.passed);
    assert!(probe.dropped.load(Ordering::SeqCst));
    assert!(probe.attempt_cleaned.load(Ordering::SeqCst));
    workflow.shutdown().await;
}
