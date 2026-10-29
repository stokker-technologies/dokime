/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;
use std::sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use dokime_runtime::harmos::operations::Host;
use dokime_runtime::harmos::plugin::Declared;
use dokime_runtime::harmos::sidecar::{ActivationId, ObserverAttachment, ObserverContext};

use dokime_runtime::execution::{CriterionContext, InitializationContext, StepContext};
use dokime_runtime::plugin::Configuration;
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::{Refusal, Supervisor};

const SUITE: &str = "calls.checks";
const TEMPLATE: &str = "calls.checks.probe";
const LATE: dokime_runtime::harmos::operations::OperationPolicy = dokime_runtime::harmos::operations::OperationPolicy {
    timeout: Duration::from_millis(20),
    attempt_timeout: Duration::from_millis(20),
    retries: dokime_runtime::harmos::operations::RetryPolicy::Never,
};

static OBSERVER_PROBES: LazyLock<Mutex<BTreeMap<String, ObserverTicks>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

static WIRE_CANCELLATIONS: AtomicU64 = AtomicU64::new(0);

// Woken whenever a fixture records a tick or a cancellation.
static CHANGED: tokio::sync::Notify = tokio::sync::Notify::const_new();

type ObserverTicks = Arc<Mutex<Vec<(u64, ActivationId)>>>;

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct GetReadingResponse {
    reads: u64,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct SharedReading {
    value: u64,
}

#[dokime_runtime::section(label = "Observer Probe")]
struct ObserverProbe {
    #[setting(label = "Probe", default = "unregistered")]
    observer_probe: String,
}

#[dokime_runtime::settings]
struct BenchSettings {
    observer: ObserverProbe,
}

struct Shared {
    reads: AtomicU64,
    observer_ticks: Option<ObserverTicks>,
}

struct Bench {
    shared: Arc<Shared>,
    observers: Option<Arc<ObserverAttachment>>,
}
#[allow(dead_code)]
struct Watch(Option<ObserverTicks>);

struct Checks;

struct Probe;

struct WireCancellation {
    completed: bool,
}

#[dokime_runtime::harmos::catalog(harmos_path = dokime_runtime::harmos)]
enum Implemented {
    GetReading(GetReading),
    GetLateReading(GetLateReading),
    GetStatus(GetStatus),
    Echo(Echo),
    Double(Double),
}

#[dokime_runtime::catalog]
enum BenchObservers {
    Watch(Watch),
}

#[dokime_runtime::catalog]
enum BenchSuites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[dokime_runtime::catalog]
enum ProbeRequired {
    GetReading(GetReading),
    Echo(Echo),
}

#[dokime_runtime::operation(id = "call_fixture.get_reading", policy = super::native::POLICY, response = GetReadingResponse, resources = [Shared])]
#[derive(Clone, PartialEq)]
struct GetReading {}
impl dokime_runtime::Operation for GetReading {
    type Error = Refusal;

    async fn run(
        self, context: dokime_runtime::harmos::Context<'_, Self>,
    ) -> Result<dokime_runtime::harmos::Response<GetReadingResponse>, Refusal> {
        let resource = context.resource::<Shared>();
        let answered: Result<GetReadingResponse, Refusal> = async {
            Ok(GetReadingResponse {
                reads: resource.reads.fetch_add(1, Ordering::SeqCst) + 1,
            })
        }
        .await;
        answered.map(Into::into)
    }
}
#[dokime_runtime::operation(id = "call_fixture.get_late_reading", policy = LATE, response = GetReadingResponse)]
#[derive(Clone, PartialEq)]
struct GetLateReading {}
impl dokime_runtime::Operation for GetLateReading {
    type Error = Refusal;

    async fn run(
        self, _: dokime_runtime::harmos::Context<'_, Self>,
    ) -> Result<dokime_runtime::harmos::Response<GetReadingResponse>, Refusal> {
        let answered: Result<GetReadingResponse, Refusal> = async {
            let mut cancellation = WireCancellation { completed: false };
            std::future::pending::<()>().await;
            cancellation.completed = true;

            Ok(GetReadingResponse { reads: 0 })
        }
        .await;
        answered.map(Into::into)
    }
}
#[dokime_runtime::operation(id = "call_fixture.get_status",
    policy = super::native::POLICY,
    response = dokime_runtime::domains::StatusResponse)]
#[derive(Clone, PartialEq)]
struct GetStatus {}
impl dokime_runtime::Operation for GetStatus {
    type Error = Refusal;

    async fn run(
        self, _: dokime_runtime::harmos::Context<'_, Self>,
    ) -> Result<dokime_runtime::harmos::Response<dokime_runtime::domains::StatusResponse>, Refusal> {
        let answered: Result<dokime_runtime::domains::StatusResponse, Refusal> =
            async { Ok(dokime_runtime::domains::StatusResponse::ready()) }.await;
        answered.map(Into::into)
    }
}
#[dokime_runtime::operation(id = "call_fixture.echo", policy = super::native::POLICY, response = SharedReading)]
#[derive(Clone, PartialEq)]
struct Echo {
    value: u64,
}
impl dokime_runtime::Operation for Echo {
    type Error = Refusal;

    async fn run(self, _: dokime_runtime::harmos::Context<'_, Self>) -> Result<dokime_runtime::harmos::Response<SharedReading>, Refusal> {
        let request = self;
        let answered: Result<SharedReading, Refusal> = async { Ok(SharedReading { value: request.value }) }.await;
        answered.map(Into::into)
    }
}
#[dokime_runtime::operation(id = "call_fixture.double", policy = super::native::POLICY, response = SharedReading)]
#[derive(Clone, PartialEq)]
struct Double {
    value: u64,
}
impl dokime_runtime::Operation for Double {
    type Error = Refusal;

    async fn run(self, _: dokime_runtime::harmos::Context<'_, Self>) -> Result<dokime_runtime::harmos::Response<SharedReading>, Refusal> {
        let request = self;
        let answered: Result<SharedReading, Refusal> = async { Ok(SharedReading { value: request.value * 2 }) }.await;
        answered.map(Into::into)
    }
}

#[allow(dead_code)]
#[dokime_runtime::operation(id = "call_fixture.hidden", policy = super::native::POLICY, response = ())]
#[derive(Clone, PartialEq)]
struct Hidden {}
// Never dispatched: the story proves no catalog lists it.
#[allow(dead_code)]
impl dokime_runtime::Operation for Hidden {
    type Error = Refusal;

    async fn run(self, _: dokime_runtime::harmos::Context<'_, Self>) -> Result<dokime_runtime::harmos::Response<()>, Refusal> {
        let answered: Result<(), Refusal> = async { panic!("an operation no catalog lists must never be dispatched") }.await;
        answered.map(Into::into)
    }
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for Bench {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "call_fixture",
    requires(),
    contributions(dokime_runtime::Suites<BenchSuites>, dokime_runtime::Settings<BenchSettings>),
    observers = BenchObservers,
    restart = "never", description = "The call verb on every context.")]
impl dokime_runtime::Plugin for Bench {
    const CATALOGS: &'static [dokime_runtime::harmos::Catalog] = &[Implemented::CATALOG];
    fn attachment(&self) -> Option<Arc<dyn dokime_runtime::harmos::plugin::Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn dokime_runtime::harmos::plugin::Attachment<Supervisor>>)
    }
}

impl dokime_runtime::harmos::Lifecycle for Bench {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    async fn prepare(config: Configuration, setup: &mut dokime_runtime::harmos::Setup<Self>) -> Result<Self, Refusal> {
        let mut owner = {
            let settings = dokime_runtime::plugin::refusal(config.settings::<BenchSettings>())?;
            Self {
                shared: Arc::new(Shared {
                    reads: AtomicU64::new(0),
                    observer_ticks: OBSERVER_PROBES.lock().unwrap().get(&settings.observer.observer_probe).cloned(),
                }),
                observers: None,
            }
        };
        setup.resource::<Shared>(owner.resources());
        owner.observers = Some(ObserverAttachment::from_catalog::<BenchObservers>(<Self as Declared>::STREAMS)?);
        dokime_runtime::execution::prepare::<Self, BenchSuites>(setup)?;
        Ok(owner)
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        Ok(())
    }
}

impl Bench {
    fn resources(&self) -> Arc<Shared> {
        Arc::clone(&self.shared)
    }
}

#[dokime_runtime::observer(interval_ms = 5, description = "A plugin observer reaches its own plugin's operations the way a step does.")]
impl Watch {
    async fn initialize(context: &dokime_runtime::harmos::ExecutionContext) -> Result<Self, dokime_runtime::harmos::sidecar::CallError> {
        let shared = context.resource::<Shared>().expect("plugin prepares its observer resources");
        Ok(Self(shared.observer_ticks.clone()))
    }

    async fn observe(&mut self, ctx: &ObserverContext<'_>) -> Result<(), dokime_runtime::harmos::sidecar::CallError> {
        let reading = ctx.call::<GetReading>(GetReading {}).await?;
        assert!(reading.reads >= 1, "the operation answered the observer");
        if ctx.tick().ordinal() <= 2
            && let Some(ticks) = &self.0
        {
            ticks.lock().unwrap().push((ctx.tick().ordinal(), ctx.activation_id()));
            CHANGED.notify_waiters();
        }

        Ok(())
    }
}

#[dokime_runtime::suite(id = SUITE, label = "Calls", templates = ChecksTemplates)]
impl Checks {}

#[dokime_runtime::template(id = TEMPLATE, label = "Probe", requires = ProbeRequired)]
impl Probe {
    async fn initialize(_: &(), _ctx: &mut InitializationContext<'_>) -> Result<Self, dokime_runtime::Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Call the operation", description = "The step asks the operation rather than the method beside it.")]
    async fn call(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime_runtime::Error> {
        let reading = ctx.call::<GetReading>(GetReading {}).await?;
        dokime_runtime::info!(ctx, "the operation answered {} reads", reading.reads);
        let echoed = ctx.call(Echo { value: 7 }).await?;
        assert_eq!(echoed.value, 7, "the request reaches the operation it names");
        let hosted = ctx.host().call::<Echo>(Echo { value: 9 }).await.expect("the host surface routes the same call");
        assert_eq!(hosted.into_owned().value, 9, "the host surface reaches the operation it names");

        Ok(())
    }

    #[dokime_runtime::observer(interval_ms = 10)]
    async fn watch(ctx: &mut dokime_runtime::execution::ObserverContext<'_>) -> Result<(), dokime_runtime::Error> {
        let reading = ctx.call::<GetReading>(GetReading {}).await?;
        dokime_runtime::debug!(ctx, "the observer read {}", reading.reads);

        Ok(())
    }

    #[dokime_runtime::criterion(
        id = "calls.read",
        label = "The operation answered",
        description = "A criterion may call too, which is why it is written `async fn` here."
    )]
    async fn read(ctx: &CriterionContext<'_>) -> Result<bool, dokime_runtime::Error> {
        let reading = ctx.call::<GetReading>(GetReading {}).await?;
        Ok(reading.reads >= 1)
    }
}

impl Drop for WireCancellation {
    fn drop(&mut self) {
        if !self.completed {
            WIRE_CANCELLATIONS.fetch_add(1, Ordering::SeqCst);
            CHANGED.notify_waiters();
        }
    }
}

// Resolves once `done` holds, re-checking after every recorded change.
async fn until(done: impl Fn() -> bool) {
    loop {
        let changed = CHANGED.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        if done() {
            return;
        }
        changed.await;
    }
}

fn configured(settings: BTreeMap<String, dokime_runtime::Value>) -> Vec<u8> {
    serde_json::to_vec(&Configuration { settings, ..Configuration::default() }).unwrap()
}

#[tokio::test]
async fn step_criterion_operation() {
    let reads = Arc::new(AtomicU64::new(0));
    let counted = Arc::clone(&reads);
    let report = Harness::template(TEMPLATE)
        .answering::<GetReading, _>(move |_| GetReadingResponse {
            reads: counted.fetch_add(1, Ordering::SeqCst) + 1,
        })
        .answering::<Echo, _>(|request: Echo| SharedReading { value: request.value })
        .run::<BenchSuites>()
        .await
        .expect("the probe runs");

    assert!(report.passed(), "{}", report.diagnostics());
    assert!(reads.load(Ordering::SeqCst) >= 2, "the step and the criterion each reached the operation");
}

#[tokio::test]
async fn unanswered_operation_name() {
    let report = Harness::template(TEMPLATE)
        .answering::<Echo, _>(|request: Echo| SharedReading { value: request.value })
        .run::<BenchSuites>()
        .await
        .expect("the probe reaches an outcome");

    assert!(!report.passed());
    assert!(report.diagnostics().contains("call_fixture.get_reading"), "{}", report.diagnostics());
}

#[tokio::test]
async fn plugin_observer_operation() {
    let ticks = Arc::new(Mutex::new(Vec::new()));
    let token = "observer-call-ticks";
    OBSERVER_PROBES.lock().unwrap().insert(token.to_owned(), Arc::clone(&ticks));
    for activation in 0..2 {
        let native = super::native::Native::initialize(
            super::native::attach::<Bench>(),
            <Bench as Declared>::VERSION,
            configured(BTreeMap::from([("observer_probe".to_owned(), dokime_runtime::Value::Text(token.to_owned()))])),
            |_, _| {},
            vec![],
        )
        .await;

        // The observer calls its own plugin's operation.
        until(|| ticks.lock().unwrap().len() >= (activation + 1) * 2).await;
        // The original observer physically joins.
        native.shutdown().await;
    }

    OBSERVER_PROBES.lock().unwrap().remove(token);
    let ticks = ticks.lock().unwrap();
    assert_eq!(ticks.len(), 4);
    for activation in ticks.as_chunks::<2>().0 {
        assert_eq!((activation[0].0, activation[1].0), (1, 2));
        assert_eq!(activation[0].1, activation[1].1, "ticks retain the same original activation");
    }
    assert_ne!(ticks[0].1, ticks[2].1, "restarted observers have distinct activations");
}

// The late operation outlives its policy deadline on the paused clock.
#[tokio::test(start_paused = true)]
async fn catalog_operations_policy() {
    let native = super::native::Native::initialize(
        super::native::attach::<Bench>(),
        <Bench as Declared>::VERSION,
        configured(BTreeMap::new()),
        |_, _| {},
        vec![],
    )
    .await;
    let operations = &native.runtime.operations;
    assert!(super::native::call_wire::<GetReading>(operations, "call_fixture", GetReading {}).await.unwrap().reads >= 1);
    assert_eq!(
        super::native::call_wire::<GetStatus>(operations, "call_fixture", GetStatus {}).await.unwrap(),
        dokime_runtime::domains::StatusResponse::ready()
    );
    assert_eq!(super::native::call_wire::<Echo>(operations, "call_fixture", Echo { value: 7 }).await.unwrap().value, 7);
    assert_eq!(super::native::call_wire::<Double>(operations, "call_fixture", Double { value: 7 }).await.unwrap().value, 14);
    let hidden = super::native::call_wire::<Hidden>(operations, "call_fixture", Hidden {}).await;
    assert!(matches!(hidden, Err(harmos::Error::NotRegistered { .. })), "{hidden:?}");

    let before = WIRE_CANCELLATIONS.load(Ordering::SeqCst);
    let late = super::native::call_wire::<GetLateReading>(operations, "call_fixture", GetLateReading {}).await;
    assert!(matches!(late, Err(harmos::Error::OperationRefused(harmos::Refusal::TimedOut))), "{late:?}");
    // The deadline cancels the operation it timed out.
    until(|| WIRE_CANCELLATIONS.load(Ordering::SeqCst) != before).await;

    native.shutdown().await;
}
