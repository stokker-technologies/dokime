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

// Third Party Imports
use dokime_runtime::harmos::catalog::Node;
use dokime_runtime::harmos::operations::{Host, OperationDeclaration, WireDeclaration};
use dokime_runtime::harmos::plugin::Declared;
use dokime_runtime::harmos::sidecar::{Attach, PluginNative};

// Local Imports
use dokime_runtime::catalog::required_by;
use dokime_runtime::execution::{InitializationContext, Run, SidecarTemplate, Signal, StepContext};
use dokime_runtime::plugin::Configuration;
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */

const OWNER: &str = "public_owner";
const CONSUMER: &str = "public_consumer";
const SUITE: &str = "public_calls.checks";
const TEMPLATE: &str = "public_calls.checks.probe";
const CONSUMER_SUITE: &str = "public_consumer.checks";
const CONSUMER_TEMPLATE: &str = "public_consumer.checks.probe";

static PROBES: LazyLock<Mutex<BTreeMap<String, Arc<Probed>>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

// Woken whenever a probed operation starts, answers or is cancelled.
static CHANGED: tokio::sync::Notify = tokio::sync::Notify::const_new();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct Probed {
    started: AtomicU64,
    answered: AtomicU64,
    cancelled: AtomicU64,
}

#[dokime_runtime::message]
#[derive(Clone, PartialEq)]
struct Reading {
    value: u64,
}

#[dokime_runtime::parameters]
struct ProbeParameters {
    #[parameter(label = "Probe", default = "unregistered")]
    probe: String,
    #[parameter(label = "Call The Public Operation", default = true)]
    public: bool,
    #[parameter(label = "Wait For Cancellation", default = false)]
    wait: bool,
}

struct Owner;

struct Consumer;

struct Checks;

struct Probe;

struct ConsumerChecks;

struct ConsumerProbe;

// Records the cancellation of an operation that never answered.
struct InFlight {
    probed: Arc<Probed>,
    completed: bool,
}

/* === Enums === */

#[dokime_runtime::harmos::catalog(harmos_path = dokime_runtime::harmos)]
enum Provided {
    ReadPublic(ReadPublic),
    ReadPrivate(ReadPrivate),
}

#[dokime_runtime::catalog]
enum OwnerSuites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[dokime_runtime::catalog]
enum ProbeRequired {
    ReadPublic(ReadPublic),
    ReadPrivate(ReadPrivate),
}

#[dokime_runtime::catalog]
enum ConsumerSuites {
    Checks(ConsumerChecks),
}

#[dokime_runtime::catalog]
enum ConsumerTemplates {
    Probe(ConsumerProbe),
}

#[dokime_runtime::catalog]
enum ConsumerRequired {
    ReadPublic(ReadPublic),
}

/* === Traits === */

/* === Implementations === */

#[dokime_runtime::operation(id = "public_owner.read_public", policy = super::native::POLICY, response = Reading, visibility = Public)]
#[derive(Clone, PartialEq)]
struct ReadPublic {
    probe: String,
    value: u64,
    wait: bool,
}
impl dokime_runtime::Operation for ReadPublic {
    type Error = Refusal;

    async fn run(self, _: dokime_runtime::harmos::Context<'_, Self>) -> Result<dokime_runtime::harmos::Response<Reading>, Refusal> {
        answer(&self.probe, self.value, self.wait).await.map(Into::into)
    }
}

#[dokime_runtime::operation(id = "public_owner.read_private", policy = super::native::POLICY, response = Reading)]
#[derive(Clone, PartialEq)]
struct ReadPrivate {
    probe: String,
    value: u64,
    wait: bool,
}
impl dokime_runtime::Operation for ReadPrivate {
    type Error = Refusal;

    async fn run(self, _: dokime_runtime::harmos::Context<'_, Self>) -> Result<dokime_runtime::harmos::Response<Reading>, Refusal> {
        answer(&self.probe, self.value, self.wait).await.map(Into::into)
    }
}

#[dokime_runtime::operation(id = "public_owner.read_host", policy = super::native::POLICY, response = Reading, visibility = Host)]
#[derive(Clone, PartialEq)]
struct ReadHost {}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for Owner {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "public_owner",
    requires(),
    contributions(dokime_runtime::Suites<OwnerSuites>),
    restart = "never", description = "Provides a public operation its own template calls.")]
impl dokime_runtime::Plugin for Owner {
    const CATALOGS: &'static [dokime_runtime::harmos::Catalog] = &[Provided::CATALOG];
}

impl dokime_runtime::harmos::Lifecycle for Owner {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    async fn prepare(_: Configuration, setup: &mut dokime_runtime::harmos::Setup<Self>) -> Result<Self, Refusal> {
        dokime_runtime::execution::prepare::<Self, OwnerSuites>(setup)?;
        Ok(Self)
    }
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for Consumer {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "public_consumer",
    requires(ReadPublic),
    contributions(dokime_runtime::Suites<ConsumerSuites>),
    restart = "never", description = "Requires another plugin's public operation.")]
impl dokime_runtime::Plugin for Consumer {
    const CATALOGS: &'static [dokime_runtime::harmos::Catalog] = &[];
}

impl dokime_runtime::harmos::Lifecycle for Consumer {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    async fn prepare(_: Configuration, _: &mut dokime_runtime::harmos::Setup<Self>) -> Result<Self, Refusal> {
        panic!("provider resolution must not prepare a plugin")
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        panic!("provider resolution must not initialize a plugin")
    }
}

#[dokime_runtime::suite(id = SUITE, label = "Public Calls", templates = ChecksTemplates)]
impl Checks {}

#[dokime_runtime::template(id = TEMPLATE, label = "Probe", requires = ProbeRequired, parameters = ProbeParameters)]
impl Probe {
    async fn initialize(_: &ProbeParameters, _: &mut InitializationContext<'_>) -> Result<Self, dokime_runtime::Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Call The Plugin's Own Operation")]
    async fn call(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime_runtime::Error> {
        let ProbeParameters { probe, public, wait } = ctx.parameters();
        let (probe, public, wait) = (probe.clone(), *public, *wait);
        let (called, hosted) = if public {
            let called = ctx.call(ReadPublic { probe: probe.clone(), value: 7, wait }).await?.value;
            let hosted = ctx.host().call::<ReadPublic>(ReadPublic { probe, value: 9, wait }).await;
            (called, hosted.expect("the host surface routes the same call").into_owned().value)
        } else {
            let called = ctx.call(ReadPrivate { probe: probe.clone(), value: 7, wait }).await?.value;
            let hosted = ctx.host().call::<ReadPrivate>(ReadPrivate { probe, value: 9, wait }).await;
            (called, hosted.expect("the host surface routes the same call").into_owned().value)
        };
        assert_eq!((called, hosted), (14, 18), "the plugin's own handler answered each call");

        Ok(())
    }
}

#[dokime_runtime::suite(id = CONSUMER_SUITE, label = "Public Consumer", templates = ConsumerTemplates)]
impl ConsumerChecks {}

#[dokime_runtime::template(id = CONSUMER_TEMPLATE, label = "Consumer Probe", requires = ConsumerRequired)]
impl ConsumerProbe {
    async fn initialize(_: &(), _: &mut InitializationContext<'_>) -> Result<Self, dokime_runtime::Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Call The Provider's Public Operation")]
    async fn call(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime_runtime::Error> {
        ctx.call(ReadPublic {
            probe: String::new(),
            value: 1,
            wait: false,
        })
        .await?;

        Ok(())
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        if !self.completed {
            self.probed.cancelled.fetch_add(1, Ordering::SeqCst);
            CHANGED.notify_waiters();
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

// Doubles `value`, or waits for cancellation when asked to.
async fn answer(probe: &str, value: u64, wait: bool) -> Result<Reading, Refusal> {
    let probed = PROBES.lock().unwrap().get(probe).cloned();
    let probed = probed.ok_or_else(|| Refusal::of("unregistered probe"))?;
    probed.started.fetch_add(1, Ordering::SeqCst);
    CHANGED.notify_waiters();
    let mut in_flight = InFlight {
        probed: Arc::clone(&probed),
        completed: false,
    };
    if wait {
        std::future::pending::<()>().await;
    }
    in_flight.completed = true;
    probed.answered.fetch_add(1, Ordering::SeqCst);
    CHANGED.notify_waiters();

    Ok(Reading { value: value * 2 })
}

fn registered(token: &str) -> Arc<Probed> {
    let probed = Arc::new(Probed::default());
    PROBES.lock().unwrap().insert(token.to_owned(), Arc::clone(&probed));

    probed
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

fn start(token: &str, probe: &str, public: bool, wait: bool) -> dokime_runtime::transport::RunExecution {
    let parameters = serde_json::json!({
        "probe": dokime_runtime::Value::Text(probe.to_owned()),
        "public": dokime_runtime::Value::Bool(public),
        "wait": dokime_runtime::Value::Bool(wait),
    });
    super::native::run_request(token, SUITE, TEMPLATE, parameters.to_string())
}

async fn opened() -> super::native::Native<PluginNative<Owner>> {
    super::native::Native::initialize(
        super::native::attach::<Owner>(),
        <Owner as Declared>::VERSION,
        serde_json::to_vec(&Configuration::default()).unwrap(),
        |_, _| {},
        vec![],
    )
    .await
}

fn definition<A: Attach>(attach: &A, version: &str) -> dokime_runtime::harmos::plugin::PluginDefinition {
    dokime_runtime::harmos::plugin::PluginDefinition {
        id: attach.owner().into(),
        version: version.parse().unwrap(),
        artifact: dokime_runtime::harmos::plugin::PluginArtifact::LocalSidecar { revision: "fixture".into() },
        configuration: serde_json::to_vec(&Configuration::default()).unwrap(),
        contributions: dokime_runtime::harmos::plugin::PluginContributions {
            operations: attach.operations().unwrap(),
            wire_contracts: attach.operation_contracts().unwrap().unwrap_or_default(),
            required_contracts: attach.required_contracts().unwrap(),
            ..Default::default()
        },
        requires: vec![],
    }
}

/* === Tests === */

#[tokio::test]
async fn template_calls_own_public() {
    let public = [Node::Leaf(WireDeclaration::of::<ReadPublic>())];
    assert!(!required_by(&public, <Owner as Declared>::REQUIRES, &[]), "the owner declares no requirement on itself");
    let native = opened().await;
    for public in [false, true] {
        let token = format!("own-{public}");
        let probed = registered(&token);
        let request = Run::new(start(&token, &token, public, false));
        let outcome = super::native::call_wire::<Run<Probe>>(&native.runtime.operations, OWNER, request).await.unwrap();
        assert!(outcome.passed, "{outcome:?}");
        assert_eq!(probed.answered.load(Ordering::SeqCst), 2, "the step and its host surface each reached the provider");
        assert_eq!(probed.cancelled.load(Ordering::SeqCst), 0);
    }
    native.shutdown().await;
}

#[tokio::test]
async fn own_public_call_cancels() {
    let native = opened().await;
    let operations = &native.runtime.operations;
    for abort in [false, true] {
        let mut settled = Vec::new();
        for public in [false, true] {
            let token = format!("cancelled-{abort}-{public}");
            let probed = registered(&token);
            let run = format!("{OWNER}.{}", <Run<Probe> as OperationDeclaration>::ID);
            let request = dokime_runtime::harmos::Message::encode(&start(&token, &token, public, true));
            let invocation = operations.submit_encoded(&run, request).unwrap();
            until(|| probed.started.load(Ordering::SeqCst) == 1).await;
            if abort {
                invocation.request_cancel();
                let cancelled = invocation.await;
                assert!(matches!(
                    cancelled,
                    Err(dokime_runtime::harmos::Error::OperationRefused(dokime_runtime::harmos::Refusal::Cancelled))
                ));
            } else {
                super::native::call_wire::<Signal<Probe>>(
                    operations,
                    OWNER,
                    Signal::new(dokime_runtime::transport::ExecutionSignal {
                        execution_token: token,
                        cancel: true,
                        ..Default::default()
                    }),
                )
                .await
                .unwrap();
                let response = invocation.await.expect("cooperative stop returns the run's final response");
                let response =
                    <dokime_runtime::transport::SidecarExecutionOutcome as dokime_runtime::harmos::Message>::decode(response.as_slice())
                        .expect("the final run response decodes");
                assert!(!response.passed);
                assert!(response.failed_in.is_none());
            }
            until(|| probed.cancelled.load(Ordering::SeqCst) == 1).await;
            settled.push((
                probed.started.load(Ordering::SeqCst),
                probed.answered.load(Ordering::SeqCst),
                probed.cancelled.load(Ordering::SeqCst),
            ));
        }
        assert_eq!(settled, [(1, 0, 1), (1, 0, 1)]);
    }
    native.shutdown().await;
}

#[test]
fn template_host_operations_refused() {
    let host = [Node::Leaf(WireDeclaration::of::<ReadHost>())];
    let public = [Node::Leaf(WireDeclaration::of::<ReadPublic>())];
    let private = [Node::Leaf(WireDeclaration::of::<ReadPrivate>())];
    assert!(!required_by(&host, &[], &host), "a plugin's own host operation never satisfies its template");
    assert!(required_by(&public, &[], &public));
    assert!(required_by(&private, &[], &private));
    assert!(required_by(<Probe as SidecarTemplate>::REQUIRED, <Owner as Declared>::REQUIRES, <Owner as Declared>::PROVIDES));
}

#[test]
fn consumer_requires_public() {
    assert!(required_by(<ConsumerProbe as SidecarTemplate>::REQUIRED, <Consumer as Declared>::REQUIRES, <Consumer as Declared>::PROVIDES));
    assert!(!required_by(<ConsumerProbe as SidecarTemplate>::REQUIRED, &[], <Consumer as Declared>::PROVIDES));
    let owner = super::native::attach::<Owner>();
    let consumer = super::native::attach::<Consumer>();
    let definitions = dokime_runtime::harmos::plugin::PluginDefinitions::new(
        [definition(&owner, <Owner as Declared>::VERSION), definition(&consumer, <Consumer as Declared>::VERSION)],
        [],
    )
    .expect("another plugin resolves the public operation it requires");
    let plugins = dokime_runtime::harmos::plugin::PluginManager::<(), (), ()>::new(definitions).unwrap();
    assert_eq!(plugins.dependencies(CONSUMER).unwrap(), [OWNER]);
}
