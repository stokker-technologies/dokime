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
use dokime::harmos::operations::{ServingError, WireTable};
use dokime::harmos::plugin::contract::Declared;
use dokime::harmos::plugin::sidecar::{ActivationId, ObserverAttachment, ObserverContext};
// Local Imports
use dokime::execution::{CriterionContext, InitializationContext, StepContext};
use dokime::plugin::Configuration;
use dokime::plugin::testing::Harness;
use dokime::{Refusal, Supervisor};

/* === Constants === */
const SUITE: &str = "calls.checks";
const TEMPLATE: &str = "calls.checks.probe";

const SLOW: Duration = Duration::from_millis(300);

const LATE: dokime::harmos::operations::OperationPolicy =
    dokime::harmos::operations::OperationPolicy {
        timeout: Duration::from_millis(20),
        attempt_timeout: Duration::from_millis(20),
        retries: dokime::harmos::operations::RetryPolicy::Never,
    };

static OBSERVER_PROBES: LazyLock<Mutex<BTreeMap<String, ObserverTicks>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

static WIRE_CANCELLATIONS: AtomicU64 = AtomicU64::new(0);

/* === Macros === */
super::native::observer_controls!(BenchObserverStart, BenchObserverStop, "call-fixture");

/* === Type Aliases === */
type ObserverTicks = Arc<Mutex<Vec<(u64, ActivationId)>>>;

/* === Structs === */
#[dokime::message]
#[derive(Clone, PartialEq)]
struct GetReadingResponse {
    reads: u64,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
struct SharedReading {
    value: u64,
}

#[dokime::section(label = "Observer Probe")]
struct ObserverProbe {
    #[setting(label = "Probe", default = "unregistered")]
    observer_probe: String,
}

#[dokime::settings]
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
#[dokime::catalog]
enum Implemented {
    GetReading(GetReading),
    GetLateReading(GetLateReading),
    GetStatus(GetStatus),
    Echo(Echo),
    Double(Double),
}

#[dokime::catalog]
enum Operations {
    #[nested]
    Implemented(Implemented),
    Start(BenchObserverStart),
    Stop(BenchObserverStop),
}

#[dokime::catalog]
enum BenchObservers {
    Watch(Watch),
}

#[dokime::catalog]
enum BenchSuites {
    Checks(Checks),
}

#[dokime::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[dokime::catalog]
enum ProbeRequired {
    GetReading(GetReading),
    Echo(Echo),
}

/* === Traits === */

/* === Implementations === */
#[dokime::operation(id = "call-fixture.get_reading", version = 1, policy = super::native::POLICY)]
impl GetReading {
    fn new(shared: Arc<Shared>) -> Self {
        Self(shared)
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<GetReadingResponse, Refusal> {
        Ok(GetReadingResponse {
            reads: self.0.reads.fetch_add(1, Ordering::SeqCst) + 1,
        })
    }
}

#[dokime::operation(id = "call-fixture.get_late_reading", version = 1, policy = LATE)]
impl GetLateReading {
    fn new(_: Arc<Shared>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<GetReadingResponse, Refusal> {
        let mut cancellation = WireCancellation { completed: false };
        tokio::time::sleep(SLOW).await;
        cancellation.completed = true;

        Ok(GetReadingResponse { reads: 0 })
    }
}

#[dokime::operation(id = "call-fixture.get_status", version = 1, policy = super::native::POLICY)]
impl GetStatus {
    fn new(_: Arc<Shared>) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &Supervisor,
        _: &(),
    ) -> Result<dokime::domains::StatusResponse, Refusal> {
        Ok(dokime::domains::StatusResponse::ready())
    }
}

#[dokime::operation(id = "call-fixture.echo", version = 1, policy = super::native::POLICY)]
impl Echo {
    fn new(_: Arc<Shared>) -> Self {
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

#[dokime::operation(id = "call-fixture.double", version = 1, policy = super::native::POLICY)]
impl Double {
    fn new(_: Arc<Shared>) -> Self {
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
#[dokime::operation(id = "call-fixture.hidden", version = 1, policy = super::native::POLICY)]
impl Hidden {
    fn new(_: Arc<Shared>) -> Self {
        Self
    }

    async fn run(&mut self, _: &Supervisor, _: &()) -> Result<(), Refusal> {
        panic!("an operation no catalog lists must never be dispatched")
    }
}

impl dokime::harmos::operations::lifecycle::Policy for Bench {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime::plugin(
    id = "call-fixture",
    provides = Operations,
    requires(),
    contributions(dokime::Suites<BenchSuites>, dokime::Settings<BenchSettings>),
    observers = BenchObservers,
    observer_controls(BenchObserverStart, BenchObserverStop),
    restart = "never", description = "The call verb on every context.")]
impl dokime::Plugin for Bench {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Shared;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<BenchSettings>())?;
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
        dokime::execution::serve::<Self, BenchSuites>(table)?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<BenchObserverStart, BenchObserverStop>(table)
    }

    fn attachment(
        &self,
    ) -> Option<Arc<dyn dokime::harmos::plugin::contract::Attachment<Supervisor>>> {
        self.observers.as_ref().map(|observers| {
            Arc::clone(observers)
                as Arc<dyn dokime::harmos::plugin::contract::Attachment<Supervisor>>
        })
    }

    async fn shutdown(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        self.observers.take();
        Ok(())
    }
}

#[dokime::observer(
    interval_ms = 5,
    description = "A plugin observer reaches its own plugin's operations the way a step does."
)]
impl Watch {
    fn new(shared: &Shared) -> Result<Self, dokime::harmos::plugin::sidecar::CallError> {
        Ok(Self(shared.observer_ticks.clone()))
    }

    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Shared>,
    ) -> Result<(), dokime::harmos::plugin::sidecar::CallError> {
        let reading = ctx.call::<GetReadingDeclaration>(()).await?;
        assert!(reading.reads >= 1, "the operation answered the observer");
        if ctx.tick().ordinal() <= 2
            && let Some(ticks) = &self.0
        {
            ticks
                .lock()
                .unwrap()
                .push((ctx.tick().ordinal(), ctx.activation_id()));
        }

        Ok(())
    }
}

#[dokime::suite(id = SUITE, label = "Calls", templates = ChecksTemplates)]
impl Checks {}

#[dokime::template(id = TEMPLATE, label = "Probe", requires = ProbeRequired)]
impl Probe {
    async fn initialize(
        _: &(),
        _ctx: &mut InitializationContext<'_>,
    ) -> Result<Self, dokime::Error> {
        Ok(Self)
    }

    #[dokime::step(
        label = "Call the operation",
        description = "The step asks the operation rather than the method beside it."
    )]
    async fn call(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime::Error> {
        let reading = ctx.call::<GetReading>(()).await?;
        dokime::info!(ctx, "the operation answered {} reads", reading.reads);
        let echoed = ctx.call::<Echo>(SharedReading { value: 7 }).await?;
        assert_eq!(
            echoed.value, 7,
            "the request reaches the operation it names"
        );

        Ok(())
    }

    #[dokime::observer(interval_ms = 10)]
    async fn watch(ctx: &mut dokime::execution::ObserverContext<'_>) -> Result<(), dokime::Error> {
        let reading = ctx.call::<GetReading>(()).await?;
        dokime::debug!(ctx, "the observer read {}", reading.reads);

        Ok(())
    }

    #[dokime::criterion(
        id = "calls.read",
        label = "The operation answered",
        description = "A criterion may call too, which is why it is written `async fn` here."
    )]
    async fn read(ctx: &CriterionContext<'_>) -> Result<bool, dokime::Error> {
        let reading = ctx.call::<GetReading>(()).await?;

        Ok(reading.reads >= 1)
    }
}

impl Drop for WireCancellation {
    fn drop(&mut self) {
        if !self.completed {
            WIRE_CANCELLATIONS.fetch_add(1, Ordering::SeqCst);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn configured(settings: BTreeMap<String, dokime::Value>) -> Vec<u8> {
    serde_json::to_vec(&Configuration {
        settings,
        ..Configuration::default()
    })
    .unwrap()
}

/* === Tests === */

pub(crate) async fn step_criterion_operation() {
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

pub(crate) async fn unanswered_operation_name() {
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

pub(crate) async fn plugin_observer_operation() {
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
                dokime::Value::Text(token.to_owned()),
            )])),
            |_, _| {},
            vec![],
        )
        .await;

        tokio::time::timeout(Duration::from_secs(5), async {
            while ticks.lock().unwrap().len() < (activation + 1) * 2 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .expect("the observer calls its own plugin's operation");
        tokio::time::timeout(Duration::from_secs(5), native.shutdown())
            .await
            .expect("the original observer physically joins");
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

pub(crate) async fn catalog_operations_policy() {
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
        dokime::domains::StatusResponse::ready()
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
    tokio::time::timeout(Duration::from_secs(2), async {
        while WIRE_CANCELLATIONS.load(Ordering::SeqCst) == before {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("the deadline cancels the operation it timed out");

    native.shutdown().await;
}
