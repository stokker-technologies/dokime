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
use dokime_runtime::harmos::operations::{Host as _, ServingError, WireTable};
use dokime_runtime::harmos::plugin::Declared;
use dokime_runtime::harmos::sidecar::{ObserverAttachment, Publisher};
use tokio_stream::StreamExt;
// Local Imports
use dokime_runtime::Value;
use dokime_runtime::execution::{CriterionContext, InitializationContext, ObserverContext, StepContext, Workflow};
use dokime_runtime::plugin::Configuration;
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */
const SUITE: &str = "observer.checks";

static PROBES: LazyLock<std::sync::Mutex<BTreeMap<String, Arc<Probe>>>> = LazyLock::new(|| std::sync::Mutex::new(BTreeMap::new()));

/* === Macros === */
/* === Type Aliases === */
type Publishing = Publisher<Owner>;

/* === Structs === */
#[derive(Default)]
struct Probe {
    starts: AtomicUsize,
    checks: AtomicUsize,
    secrets: AtomicBool,
    secret_ticks: AtomicUsize,
    stops: AtomicUsize,
    cleaned: AtomicBool,
    failed: AtomicUsize,
    entered: tokio::sync::Notify,
    dropped: AtomicBool,
    attempt_cleaned: AtomicBool,
    blocking: AtomicBool,
    released: std::sync::Mutex<bool>,
    release: std::sync::Condvar,
    changed: tokio::sync::Notify,
}

#[dokime_runtime::section(
    label = "Probe",
    description = "The token a run names its registered [`Probe`] by, standing in for the live handle a setting cannot carry across \
        the wire."
)]
pub struct Probes {
    #[setting(
        label = "Probe Token",
        default = "unregistered",
        description = "Never read as its declared default: every run in this file names a real registered token, but a required text \
        setting still has to declare a nonempty one to pass catalog validation."
    )]
    pub token: String,
}

#[dokime_runtime::settings]
pub struct OwnerSettings {
    pub probes: Probes,
}

#[dokime_runtime::section(label = "Required Secrets")]
struct RequiredSecrets {
    #[setting(label = "Token", from_environment = "OBSERVER_FIXTURE_TOKEN")]
    token: dokime_runtime::plugin::Secret,
}

#[dokime_runtime::settings]
struct MissingSecretSettings {
    secrets: RequiredSecrets,
}

struct MissingSecrets;

struct Shared {
    probe: Arc<Probe>,
    token: String,
    secrets: bool,
}

struct Owner {
    shared: Arc<Shared>,
    observers: Option<Arc<ObserverAttachment<Shared, Publishing>>>,
}

#[dokime_runtime::stream(id = "observer_fixture.status", label = "Observer Status", window = 64)]
pub struct ObservationRow {
    index: u64,
}

struct Watch(Arc<Probe>);

#[derive(Default)]
struct StatelessWatch;
struct Checks;

struct AwaitEvent;

struct AttemptResources(tokio::sync::Notify);

struct Faults(Arc<Probe>);

struct Pending(Arc<Probe>);

struct PendingGuard(Arc<Probe>);

#[dokime_runtime::record(id = "observer.suite_observed", label = "Suite Observed")]
struct SuiteObserved {
    index: u64,
}

#[dokime_runtime::record(id = "observer.observed", label = "Observed")]
pub struct Observed {
    index: u64,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct GetCountResponse {
    checks: u64,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct GetProbeResponse {
    token: String,
    secrets: bool,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum WatchError {
    #[error(transparent)]
    Publish(#[from] dokime_runtime::harmos::sidecar::PublishError),
    #[error(transparent)]
    Call(#[from] dokime_runtime::harmos::sidecar::CallError),
    #[error(transparent)]
    Sdk(#[from] dokime_runtime::Error),
}

#[dokime_runtime::catalog]
enum ObserveRecords {
    Observed(Observed),
}

#[dokime_runtime::catalog]
enum Implemented {
    GetCount(GetCount),
    GetProbe(GetProbe),
}

#[dokime_runtime::catalog]
enum OwnerObservers {
    Watch(Watch),
    StatelessWatch(StatelessWatch),
}

#[dokime_runtime::catalog]
pub enum Records {
    Observed(Observed),
}

#[dokime_runtime::catalog]
enum OwnerSuites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum ChecksTemplates {
    AwaitEvent(AwaitEvent),
    Faults(Faults),
    Pending(Pending),
}

#[dokime_runtime::catalog]
enum ChecksRecords {
    SuiteObserved(SuiteObserved),
}

#[dokime_runtime::catalog]
enum AwaitEventRequired {
    GetCount(GetCount),
}

#[dokime_runtime::catalog]
enum ProbeRequired {
    GetProbe(GetProbe),
}

/* === Traits === */

/* === Implementations === */
impl Probe {
    // Counts one event and wakes every story waiting on the probe.
    fn bump(&self, counter: &AtomicUsize) {
        counter.fetch_add(1, Ordering::SeqCst);
        self.changed.notify_waiters();
    }

    // Resolves once `done` holds, re-checking after every counted event.
    async fn until(&self, done: impl Fn(&Self) -> bool) {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if done(self) {
                return;
            }
            changed.await;
        }
    }
}
#[dokime_runtime::operation(id = "observer_fixture.get_count", policy = super::native::POLICY, response = GetCountResponse, resources = [Shared])]
#[derive(Clone, PartialEq)]
struct GetCount {}
impl dokime_runtime::Operation for GetCount {
    type Error = Refusal;

    async fn run(
        self, context: dokime_runtime::harmos::Context<'_, Self>,
    ) -> Result<dokime_runtime::harmos::Response<GetCountResponse>, Refusal> {
        let resource = context.resource::<Shared>();
        let answered: Result<GetCountResponse, Refusal> = async {
            Ok(GetCountResponse {
                checks: resource.probe.checks.load(Ordering::SeqCst) as u64,
            })
        }
        .await;
        answered.map(Into::into)
    }
}
#[dokime_runtime::operation(id = "observer_fixture.get_probe", policy = super::native::POLICY, response = GetProbeResponse, resources = [Shared])]
#[derive(Clone, PartialEq)]
struct GetProbe {}
impl dokime_runtime::Operation for GetProbe {
    type Error = Refusal;

    async fn run(
        self, context: dokime_runtime::harmos::Context<'_, Self>,
    ) -> Result<dokime_runtime::harmos::Response<GetProbeResponse>, Refusal> {
        let resource = context.resource::<Shared>();
        let answered: Result<GetProbeResponse, Refusal> = async {
            Ok(GetProbeResponse {
                token: resource.token.clone(),
                secrets: resource.secrets,
            })
        }
        .await;
        answered.map(Into::into)
    }
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for Owner {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "observer_fixture",
    provides = Implemented,
    requires(),
    contributions(
        dokime_runtime::Suites<OwnerSuites>,
        dokime_runtime::Records<Records>,
        dokime_runtime::Settings<OwnerSettings>,
    ),
    observers = OwnerObservers,
    restart = "never", description = "Observer Lifecycle Fixture.")]
impl dokime_runtime::Plugin for Owner {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Shared;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime_runtime::plugin::refusal(config.settings::<OwnerSettings>())?;
        Ok(Self {
            shared: Arc::new(Shared {
                probe: probe(&settings.probes.token),
                token: settings.probes.token,
                secrets: config.secrets_configured::<OwnerSettings>(),
            }),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<OwnerObservers>(self.resources(), <Self as Declared>::STREAMS)?);
        Ok(())
    }

    fn resources(&self) -> Arc<Shared> {
        Arc::clone(&self.shared)
    }

    fn bind_operations(&self, table: &mut WireTable) -> Result<(), ServingError> {
        table.register(Implemented::registration({
            let mut dependencies = dokime_runtime::harmos::Dependencies::new();
            dependencies.insert(self.resources()).map_err(|_| dokime_runtime::harmos::sidecar::operations::ServingError::Catalog)?;
            dependencies
        }))?;
        dokime_runtime::execution::serve::<Self, OwnerSuites>(table)?;
        self.observers.as_ref().ok_or(ServingError::Catalog)?.bind(table)
    }

    fn attachment(&self) -> Option<Arc<dyn dokime_runtime::harmos::plugin::Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn dokime_runtime::harmos::plugin::Attachment<Supervisor>>)
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

#[dokime_runtime::observer(interval_ms = 5)]
impl StatelessWatch {
    async fn observe(&mut self, ctx: &dokime_runtime::harmos::sidecar::ObserverContext<'_, Shared, Publishing>) -> Result<(), WatchError> {
        let _ = (ctx.resources(), ctx.tick());
        Ok(())
    }
}

#[dokime_runtime::observer(interval_ms = 5, streams(ObservationRow))]
impl Watch {
    async fn initialize(shared: &Shared) -> Result<Self, WatchError> {
        shared.probe.starts.fetch_add(1, Ordering::SeqCst);
        Ok(Self(Arc::clone(&shared.probe)))
    }

    async fn observe(&mut self, ctx: &dokime_runtime::harmos::sidecar::ObserverContext<'_, Shared, Publishing>) -> Result<(), WatchError> {
        let host = ctx.host().supervisor();
        let secrets = host.call::<GetProbe>(GetProbe {}).await.map(|response| response.into_owned())?.secrets;
        self.0.secrets.store(secrets, Ordering::Release);
        self.0.bump(&self.0.secret_ticks);
        let index = ctx.tick().ordinal() - 1;
        ctx.publish_on::<ObservationRow>(&ObservationRow { index }).await?;
        dokime_runtime::plugin::Publisher::new(host).publish(Observed { index }).await?;
        self.0.bump(&self.0.checks);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), WatchError> {
        self.0.stops.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for MissingSecrets {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "missing_observer_secrets",
    provides = (),
    requires(),
    contributions(dokime_runtime::Settings<MissingSecretSettings>),
    restart = "never", description = "A plugin whose card requires a secret nobody bound.")]
impl dokime_runtime::Plugin for MissingSecrets {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime_runtime::plugin::refusal(config.settings::<MissingSecretSettings>())?;
        let _ = settings.secrets.token;
        Ok(Self)
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, _: &mut WireTable) -> Result<(), ServingError> {
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

#[dokime_runtime::suite(
    id = SUITE,
    label = "Observer Checks",
    templates = ChecksTemplates,
    records = ChecksRecords
)]
impl Checks {}

#[dokime_runtime::template(
    id = "observer.checks.await",
    label = "Await Event",
    requires = AwaitEventRequired
)]
impl AwaitEvent {
    async fn initialize(_ctx: &mut InitializationContext<'_>) -> dokime_runtime::Result<(Self, AttemptResources)> {
        let template = Self;
        let shared = AttemptResources(tokio::sync::Notify::new());
        Ok((template, shared))
    }

    #[dokime_runtime::step(label = "Hear The Event")]
    async fn hear(&mut self, ctx: &mut StepContext<'_>) -> dokime_runtime::Result<()> {
        // An operation is the only road a step has to the plugin. Its type
        // belongs to the required catalog declared by this template.
        let counted = ctx.call(GetCount {}).await?;
        let _ = counted.checks;
        ctx.publish(Observed { index: 999 }).await;
        ctx.resources().0.notify_one();
        let observed = ctx.listen::<Observed>().next().await.expect("the observer publishes");
        assert_eq!(observed.index, 0, "a step does not hear its own publication");
        ctx.publish(Observed { index: 777 }).await;
        // The Observer Owns Another Runtime. Wait For Its Event Before Pausing
        // This Clock, Then Simulate Only The Duplicate-Delivery Window.
        tokio::time::pause();
        let mut repeated = ctx.listen::<Observed>().skip(1);
        let duplicate = tokio::select! {
            biased;
            repeated = repeated.next() => Some(repeated.map(|row| row.index)),
            () = tokio::time::sleep(Duration::from_millis(10)) => None,
        };
        tokio::time::resume();
        assert!(duplicate.is_none(), "the observer event is delivered only once: {duplicate:?}");
        Ok(())
    }

    #[dokime_runtime::criterion(
        id = "observer.checks.observed",
        label = "Event Recorded",
        description = "A criterion calls too, which is what makes an operation reachable from every stage rather than from the ones \
        that hold handles."
    )]
    async fn observed(ctx: &CriterionContext<'_>) -> dokime_runtime::Result<bool> {
        let counted = ctx.call(GetCount {}).await?;
        ctx.info(format!("the plugin has answered {} checks", counted.checks));

        Ok(ctx.records::<Observed>()?.iter().map(|record| record.index).collect::<Vec<_>>() == [999, 0, 777])
    }

    #[dokime_runtime::observer(interval_ms = 5, records = ObserveRecords)]
    async fn observe(ctx: &mut ObserverContext<'_>) -> dokime_runtime::Result<()> {
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

#[dokime_runtime::template(
    id = "observer.checks.faults",
    label = "Observer Faults",
    requires = ProbeRequired
)]
impl Faults {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> dokime_runtime::Result<(Self, Arc<Probe>)> {
        let template = Self(probe(&ctx.call(GetProbe {}).await?.token));
        let shared = Arc::clone(&template.0);
        Ok((template, shared))
    }

    #[dokime_runtime::step(label = "Wait For Independent Observers")]
    async fn proceed(&mut self, ctx: &mut StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.listen::<SuiteObserved>().next().await.expect("the healthy observer continues");
        *self.0.released.lock().unwrap() = true;
        self.0.release.notify_all();
        self.0.until(|probe| probe.failed.load(Ordering::SeqCst) >= 2).await;
        Ok(())
    }

    #[dokime_runtime::observer(interval_ms = 5)]
    async fn fails(ctx: &mut ObserverContext<'_>) -> Result<(), std::io::Error> {
        ctx.resources().bump(&ctx.resources().failed);
        Err(std::io::Error::other("observer check refused"))
    }

    #[dokime_runtime::observer(interval_ms = 5)]
    async fn panics(ctx: &mut ObserverContext<'_>) -> dokime_runtime::Result<()> {
        ctx.resources().bump(&ctx.resources().failed);
        panic!("observer check panicked");
    }

    #[dokime_runtime::observer(interval_ms = 5)]
    async fn healthy(ctx: &mut ObserverContext<'_>) -> dokime_runtime::Result<()> {
        if !ctx.resources().blocking.load(Ordering::SeqCst) {
            return Ok(());
        }
        let index = ctx.tick().index();
        ctx.publish(SuiteObserved { index }).await;
        Ok(())
    }

    #[dokime_runtime::observer(interval_ms = 5)]
    async fn blocking(ctx: &mut ObserverContext<'_>) -> dokime_runtime::Result<()> {
        let probe = ctx.resources();
        probe.blocking.store(true, Ordering::SeqCst);
        // Holds its thread until the step releases it; a starved sibling or
        // step never does, and the lane deadline ends the story.
        let released = probe.released.lock().unwrap();
        drop(probe.release.wait_while(released, |released| !*released).unwrap());
        Ok(())
    }
}

#[dokime_runtime::template(
    id = "observer.checks.pending",
    label = "Pending Observer",
    requires = ProbeRequired
)]
impl Pending {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> dokime_runtime::Result<(Self, Arc<Probe>)> {
        let template = Self(probe(&ctx.call(GetProbe {}).await?.token));
        let shared = Arc::clone(&template.0);
        Ok((template, shared))
    }

    #[dokime_runtime::step(label = "Wait For Cancellation")]
    async fn wait(&mut self, _: &mut StepContext<'_>) -> dokime_runtime::Result<()> {
        std::future::pending().await
    }

    #[dokime_runtime::observer(interval_ms = 5)]
    async fn blocked(ctx: &mut ObserverContext<'_>) -> dokime_runtime::Result<()> {
        let resources = ctx.resources();
        let _guard = PendingGuard(Arc::clone(resources));
        resources.entered.notify_one();
        std::future::pending().await
    }

    async fn cleanup(&mut self, _: &mut dokime_runtime::execution::CleanupContext<'_>) -> dokime_runtime::Result<()> {
        assert!(self.0.dropped.load(Ordering::SeqCst), "pending check dropped before cleanup");
        self.0.attempt_cleaned.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn registered(probe: &Arc<Probe>) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let token = format!("probe-{}", NEXT.fetch_add(1, Ordering::SeqCst));
    PROBES.lock().expect("the probe registry is never poisoned").insert(token.clone(), Arc::clone(probe));

    token
}

fn probe(token: &str) -> Arc<Probe> {
    Arc::clone(PROBES.lock().expect("the probe registry is never poisoned").get(token).expect("a run names a probe this file registered"))
}
fn answered(token: String) -> impl Fn(GetProbe) -> GetProbeResponse + Send + Sync + 'static {
    move |_| GetProbeResponse { token: token.clone(), secrets: true }
}

/* === Tests === */

#[tokio::test]
async fn activation_starts_execution() {
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
    // An observer publishes without any demand.
    native.host.first_filed().await;
    let (owner, filed) = native.host.records().remove(0);
    assert_eq!(owner, "observer_fixture");
    assert_eq!(filed.record_type.as_str(), "observer.observed");

    let Err(refused) = <MissingSecrets as dokime_runtime::Plugin>::prepare(Configuration::declared::<MissingSecretSettings>()) else {
        panic!("the required secret is absent");
    };
    assert!(refused.to_string().contains("token"), "{refused}");
    let previous = probe.secret_ticks.load(Ordering::Acquire);
    // The original observer ticks after another preparation fails.
    probe.until(|probe| probe.secret_ticks.load(Ordering::Acquire) > previous).await;
    assert!(probe.secrets.load(Ordering::Acquire));
    assert_eq!(probe.starts.load(Ordering::SeqCst), 1);
    native.shutdown().await;
    assert_eq!(probe.stops.load(Ordering::SeqCst), 1);
    assert!(probe.cleaned.load(Ordering::SeqCst));
    let settled = probe.checks.load(Ordering::SeqCst);
    // Shutdown Joins Every Observer Before Returning
    assert_eq!(probe.checks.load(Ordering::SeqCst), settled);
}

#[tokio::test]
async fn template_events_criterion() {
    let probe = Arc::new(Probe::default());
    let checks = Arc::clone(&probe);
    let report = Harness::template("observer.checks.await")
        .answering::<GetCount, _>(move |_| GetCountResponse {
            checks: checks.checks.load(Ordering::SeqCst) as u64,
        })
        .run::<OwnerSuites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.criteria().len(), 1);
    let records = report.records::<Observed>();
    assert_eq!(records.iter().map(|record| record.index).collect::<Vec<_>>(), [999, 0, 777]);
    assert_eq!(probe.starts.load(Ordering::SeqCst), 0, "a template run starts no plugin observer");
}

#[tokio::test]
async fn observer_failure_event() {
    let probe = Arc::new(Probe::default());
    let report = Harness::template("observer.checks.faults")
        .answering::<GetProbe, _>(answered(registered(&probe)))
        .run::<OwnerSuites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    let diagnostics = report.diagnostics();
    assert!(diagnostics.contains("observer check refused"), "{diagnostics}");
    assert!(diagnostics.contains("observer check panicked"), "{diagnostics}");
    assert_eq!(probe.failed.load(Ordering::SeqCst), 2);
    assert!(!report.records::<SuiteObserved>().is_empty());
}

#[tokio::test]
async fn cancellation_drops_cleanup() {
    let probe = Arc::new(Probe::default());
    let workflow = Workflow::<Pending>::new(SUITE, "observer_fixture");
    workflow
        .start_on(
            super::native::run_request("pending", SUITE, "observer.checks.pending", "{}".into()),
            None,
            dokime_runtime::execution::Caller::local(BTreeMap::from([super::native::answering::<GetProbe, _>(answered(registered(
                &probe,
            )))])),
        )
        .unwrap();
    let mut cursor = 0;
    // Drains between waits admit the step that enters the pending observer.
    loop {
        tokio::select! {
            () = probe.entered.notified() => break,
            batch = workflow.next("pending", cursor) => {
                let batch = batch.unwrap();
                cursor = batch.cursor;
                for encoded in &batch.step_begins {
                    let start: dokime_domain::model::execution::result::StepAttemptStart =
                        serde_json::from_str(encoded).unwrap();
                    workflow.admit("pending", &dokime_runtime::transport::StepAdmission {
                        attempt_id: start.attempt_id.to_string(),
                        accepted: true,
                        message: String::new(),
                    });
                }
            }
        }
    }
    workflow.cancel("pending");
    let outcome = super::native::settled(&workflow, "pending", cursor).await;
    assert!(!outcome.passed);
    assert!(probe.dropped.load(Ordering::SeqCst));
    assert!(probe.attempt_cleaned.load(Ordering::SeqCst));
    workflow.shutdown().await;
}
