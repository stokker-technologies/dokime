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
use dokime_runtime::harmos::operations::{Host, ServingError, WireTable};
use dokime_runtime::harmos::plugin::Declared;
use dokime_runtime::harmos::sidecar::{ActivationId, ObserverAttachment, ObserverContext};
// Local Imports
use dokime_runtime::execution::{CriterionContext, InitializationContext, StepContext};
use dokime_runtime::plugin::Configuration;
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */
const SUITE: &str = "calls.checks";
const TEMPLATE: &str = "calls.checks.probe";

const SLOW: Duration = Duration::from_millis(300);

const LATE: dokime_runtime::harmos::operations::OperationPolicy =
    dokime_runtime::harmos::operations::OperationPolicy {
        timeout: Duration::from_millis(20),
        attempt_timeout: Duration::from_millis(20),
        retries: dokime_runtime::harmos::operations::RetryPolicy::Never,
        restart: dokime_runtime::harmos::operations::RestartPolicy::Never,
    };

static OBSERVER_PROBES: LazyLock<Mutex<BTreeMap<String, ObserverTicks>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

static WIRE_CANCELLATIONS: AtomicU64 = AtomicU64::new(0);

// Woken whenever a fixture records a tick or a cancellation.
static CHANGED: tokio::sync::Notify = tokio::sync::Notify::const_new();

/* === Macros === */
super::native::observer_control!(BenchObserveControl, "call-fixture");

/* === Type Aliases === */
type ObserverTicks = Arc<Mutex<Vec<(u64, ActivationId)>>>;

/* === Structs === */
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
    observers: Option<Arc<ObserverAttachment<Shared>>>,
}

struct GetReading(Arc<Shared>);

struct GetLateReading;

struct GetStatus;

struct Echo;

struct Double;

#[allow(dead_code)]
struct Hidden;

struct Watch(Option<ObserverTicks>);

struct Checks;

struct Probe;

struct WireCancellation {
    completed: bool,
}

/* === Enums === */
#[dokime_runtime::catalog]
enum Implemented {
    GetReading(GetReading),
    GetLateReading(GetLateReading),
    GetStatus(GetStatus),
    Echo(Echo),
    Double(Double),
}

#[dokime_runtime::catalog]
enum Operations {
    #[nested]
    Implemented(Implemented),
    Observe(BenchObserveControl),
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

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::operation(id = "call-fixture.get_reading", version = 1, policy = super::native::POLICY)]
impl GetReading {
    async fn initialize(shared: Arc<Shared>, _: &Supervisor) -> Self {
        Self(shared)
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<GetReadingResponse, Refusal> {
        Ok(GetReadingResponse {
            reads: self.0.reads.fetch_add(1, Ordering::SeqCst) + 1,
        })
    }
}

#[dokime_runtime::operation(id = "call-fixture.get_late_reading", version = 1, policy = LATE)]
impl GetLateReading {
    async fn initialize(_: Arc<Shared>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<GetReadingResponse, Refusal> {
        let mut cancellation = WireCancellation { completed: false };
        tokio::time::sleep(SLOW).await;
        cancellation.completed = true;

        Ok(GetReadingResponse { reads: 0 })
    }
}

#[dokime_runtime::operation(id = "call-fixture.get_status", version = 1, policy = super::native::POLICY)]
impl GetStatus {
    async fn initialize(_: Arc<Shared>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &Supervisor,
        _: &(),
    ) -> Result<dokime_runtime::domains::StatusResponse, Refusal> {
        Ok(dokime_runtime::domains::StatusResponse::ready())
    }
}

#[dokime_runtime::operation(id = "call-fixture.echo", version = 1, policy = super::native::POLICY)]
impl Echo {
    async fn initialize(_: Arc<Shared>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &Supervisor,
        request: &SharedReading,
    ) -> Result<SharedReading, Refusal> {
        Ok(request.clone())
    }
}

#[dokime_runtime::operation(id = "call-fixture.double", version = 1, policy = super::native::POLICY)]
impl Double {
    async fn initialize(_: Arc<Shared>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &Supervisor,
        request: &SharedReading,
    ) -> Result<SharedReading, Refusal> {
        Ok(SharedReading {
            value: request.value * 2,
        })
    }
}

#[allow(dead_code)]
#[dokime_runtime::operation(id = "call-fixture.hidden", version = 1, policy = super::native::POLICY)]
impl Hidden {
    async fn initialize(_: Arc<Shared>, _: &Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<(), Refusal> {
        panic!("an operation no catalog lists must never be dispatched")
    }
}

impl dokime_runtime::harmos::operations::lifecycle::Policy for Bench {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "call-fixture",
    provides = Operations,
    requires(),
    contributions(dokime_runtime::Suites<BenchSuites>, dokime_runtime::Settings<BenchSettings>),
    observers = BenchObservers,
    observer_control = BenchObserveControl,
    restart = "never", description = "The call verb on every context.")]
impl dokime_runtime::Plugin for Bench {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Shared;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime_runtime::plugin::refusal(config.settings::<BenchSettings>())?;
        Ok(Self {
            shared: Arc::new(Shared {
                reads: AtomicU64::new(0),
                observer_ticks: OBSERVER_PROBES
                    .lock()
                    .unwrap()
                    .get(&settings.observer.observer_probe)
                    .cloned(),
            }),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<BenchObservers>(
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
        dokime_runtime::execution::serve::<Self, BenchSuites>(table)?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<BenchObserveControl>(table)
    }

    fn attachment(&self) -> Option<Arc<dyn dokime_runtime::harmos::plugin::Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| {
            Arc::clone(observers) as Arc<dyn dokime_runtime::harmos::plugin::Attachment<Supervisor>>
        })
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        Ok(())
    }
}

#[dokime_runtime::observer(
    interval_ms = 5,
    description = "A plugin observer reaches its own plugin's operations the way a step does."
)]
impl Watch {
    async fn initialize(shared: &Shared) -> Result<Self, dokime_runtime::harmos::sidecar::CallError> {
        Ok(Self(shared.observer_ticks.clone()))
    }

    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Shared>,
    ) -> Result<(), dokime_runtime::harmos::sidecar::CallError> {
        let reading = ctx.call_as::<GetReadingDeclaration>(()).await?;
        assert!(reading.reads >= 1, "the operation answered the observer");
        if ctx.tick().ordinal() <= 2
            && let Some(ticks) = &self.0
        {
            ticks
                .lock()
                .unwrap()
                .push((ctx.tick().ordinal(), ctx.activation_id()));
            CHANGED.notify_waiters();
        }

        Ok(())
    }
}

#[dokime_runtime::suite(id = SUITE, label = "Calls", templates = ChecksTemplates)]
impl Checks {}

#[dokime_runtime::template(id = TEMPLATE, label = "Probe", requires = ProbeRequired)]
impl Probe {
    async fn initialize(
        _: &(),
        _ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, dokime_runtime::Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(
        label = "Call the operation",
        description = "The step asks the operation rather than the method beside it."
    )]
    async fn call(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime_runtime::Error> {
        let reading = ctx.call::<GetReading>(()).await?;
        dokime_runtime::info!(ctx, "the operation answered {} reads", reading.reads);
        let echoed = ctx.call::<Echo>(SharedReading { value: 7 }).await?;
        assert_eq!(
            echoed.value, 7,
            "the request reaches the operation it names"
        );
        let hosted = ctx
            .host()
            .call_as::<EchoDeclaration>(SharedReading { value: 9 })
            .await
            .expect("the host surface routes the same call");
        assert_eq!(
            hosted.into_owned().value,
            9,
            "the host surface reaches the operation it names"
        );

        Ok(())
    }

    #[dokime_runtime::observer(interval_ms = 10)]
    async fn watch(ctx: &mut dokime_runtime::execution::ObserverContext<'_>) -> Result<(), dokime_runtime::Error> {
        let reading = ctx.call::<GetReading>(()).await?;
        dokime_runtime::debug!(ctx, "the observer read {}", reading.reads);

        Ok(())
    }

    #[dokime_runtime::criterion(
        id = "calls.read",
        label = "The operation answered",
        description = "A criterion may call too, which is why it is written `async fn` here."
    )]
    async fn read(ctx: &CriterionContext<'_>) -> Result<bool, dokime_runtime::Error> {
        let reading = ctx.call::<GetReading>(()).await?;

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

/* === Public Functions === */

/* === Private Functions === */
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
    serde_json::to_vec(&Configuration {
        settings,
        ..Configuration::default()
    })
    .unwrap()
}

/* === Tests === */

#[tokio::test]
async fn step_criterion_operation() {
    let reads = Arc::new(AtomicU64::new(0));
    let counted = Arc::clone(&reads);
    let report = Harness::template(TEMPLATE)
        .answering::<GetReading, _>(move |()| GetReadingResponse {
            reads: counted.fetch_add(1, Ordering::SeqCst) + 1,
        })
        .answering::<Echo, _>(|request: SharedReading| request)
        .run::<BenchSuites>()
        .await
        .expect("the probe runs");

    assert!(report.passed(), "{}", report.diagnostics());
    assert!(
        reads.load(Ordering::SeqCst) >= 2,
        "the step and the criterion each reached the operation"
    );
}

#[tokio::test]
async fn unanswered_operation_name() {
    let report = Harness::template(TEMPLATE)
        .answering::<Echo, _>(|request: SharedReading| request)
        .run::<BenchSuites>()
        .await
        .expect("the probe reaches an outcome");

    assert!(!report.passed());
    assert!(
        report.diagnostics().contains("call-fixture.get_reading"),
        "{}",
        report.diagnostics()
    );
}

#[tokio::test]
async fn plugin_observer_operation() {
    let ticks = Arc::new(Mutex::new(Vec::new()));
    let token = "observer-call-ticks";
    OBSERVER_PROBES
        .lock()
        .unwrap()
        .insert(token.to_owned(), Arc::clone(&ticks));
    for activation in 0..2 {
        let native = super::native::Native::initialize(
            super::native::attach::<Bench>(),
            <Bench as Declared>::VERSION,
            configured(BTreeMap::from([(
                "observer_probe".to_owned(),
                dokime_runtime::Value::Text(token.to_owned()),
            )])),
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
        assert_eq!(
            activation[0].1, activation[1].1,
            "ticks retain the same original activation"
        );
    }
    assert_ne!(
        ticks[0].1, ticks[2].1,
        "restarted observers have distinct activations"
    );
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

    assert!(
        super::native::call_wire::<GetReadingDeclaration>(operations, ())
            .await
            .unwrap()
            .reads
            >= 1
    );
    assert_eq!(
        super::native::call_wire::<GetStatusDeclaration>(operations, ())
            .await
            .unwrap(),
        dokime_runtime::domains::StatusResponse::ready()
    );
    assert_eq!(
        super::native::call_wire::<EchoDeclaration>(operations, SharedReading { value: 7 })
            .await
            .unwrap()
            .value,
        7
    );
    assert_eq!(
        super::native::call_wire::<DoubleDeclaration>(operations, SharedReading { value: 7 })
            .await
            .unwrap()
            .value,
        14
    );

    let hidden = super::native::call_wire::<HiddenDeclaration>(operations, ()).await;
    assert!(
        matches!(hidden, Err(harmos::Error::NotRegistered { .. })),
        "{hidden:?}"
    );

    let before = WIRE_CANCELLATIONS.load(Ordering::SeqCst);
    let late = super::native::call_wire::<GetLateReadingDeclaration>(operations, ()).await;
    assert!(
        matches!(
            late,
            Err(harmos::Error::OperationRefused(harmos::Refusal::TimedOut))
        ),
        "{late:?}"
    );
    // The deadline cancels the operation it timed out.
    until(|| WIRE_CANCELLATIONS.load(Ordering::SeqCst) != before).await;

    native.shutdown().await;
}
