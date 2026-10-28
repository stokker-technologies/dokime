/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod observers {
    use super::*;

    #[dokime_runtime::row]
    #[derive(Clone)]
    pub(super) struct Row {
        #[row(label = "Measured Value", unit = "V", format = fixed(2), description = "A shared measurement")]
        pub(super) value: i64,
        pub(super) index: i64,
    }
    #[dokime_runtime::stream(id = "owned-stream.auxiliary",
        window = 64,
        row = Row,
        label = "Auxiliary",
        description = "One shape, two streams: each of them is a type, and neither of them is the shape, so each observer says which \
        of them its rows are.")]
    pub(super) struct Auxiliary;

    #[dokime_runtime::stream(id = "owned-stream.measurement", window = 64, row = Row, label = "Measurement")]
    pub(super) struct Measurement;

    #[dokime_runtime::stream(
        id = "owned-stream.revisioned",
        label = "Revisioned",
        tags("instrument"),
        attributes(source = "test"),
        window = 64,
        row = Row, description = "Shared row metadata")]
    struct Revisioned;

    #[test]
    fn native_metadata_revision() {
        use dokime_runtime::telemetry::TelemetryStream;
        let contract = Revisioned::contract();
        assert_eq!(contract.stream.as_str(), "owned-stream.revisioned");
        assert_eq!(contract.schema_version, <Revisioned as dokime_runtime::StreamSpec>::VERSION);
        assert_eq!(contract.metadata.label, "Revisioned");
        assert_eq!(contract.metadata.description.as_deref(), Some("Shared row metadata"));
        assert_eq!(contract.metadata.tags, vec!["instrument"]);
        assert_eq!(contract.metadata.attributes["source"], dokime_runtime::Value::Text("test".into()));
        let value = contract.columns.iter().find(|column| column.metadata.identifier.as_str() == "value").unwrap();
        assert_eq!(value.metadata.label, "Measured Value");
        assert_eq!(value.metadata.description.as_deref(), Some("A shared measurement"));
        assert!(value.format.is_some());
    }

    pub(super) struct Auxiliaries {
        probe: Arc<Probe>,
    }

    impl dokime_runtime::harmos::plugin::inventory::observers::Declared for Auxiliaries {
        const ENTRY: dokime_runtime::harmos::plugin::inventory::observers::Entry =
            dokime_runtime::harmos::plugin::inventory::observers::Entry {
                id: <Self as dokime_runtime::harmos::sidecar::Observer<Publishing>>::ID,
                description: "",
                interval: <Self as dokime_runtime::harmos::sidecar::Observer<Publishing>>::INTERVAL,
                activation: <Self as dokime_runtime::harmos::sidecar::Observer<Publishing>>::ACTIVATION,
                streams: <Self as dokime_runtime::harmos::sidecar::Observer<Publishing>>::STREAMS,
                demand_keys: &[],
            };
    }

    impl dokime_runtime::harmos::sidecar::Observer<Publishing> for Auxiliaries {
        type Resources = Resource;
        type Error = Error;
        const ID: &'static str = "fixture.auxiliaries";
        const STREAMS: &'static [dokime_runtime::harmos::plugin::inventory::publications::Entry] =
            &[dokime_runtime::harmos::plugin::inventory::publications::Entry::of::<Auxiliary>()];
        const ACTIVATION: dokime_runtime::harmos::sidecar::Activation = dokime_runtime::harmos::sidecar::Activation::Demand;
        const INTERVAL: Duration = Duration::from_millis(5);

        async fn initialize(resource: &Resource) -> Result<Self, Error> {
            Ok(Self { probe: Arc::clone(&resource.0) })
        }

        async fn observe(&mut self, ctx: &ObserverContext<'_, Resource, Publishing>) -> Result<(), Error> {
            self.probe.bump(&self.probe.auxiliary_samples);
            ctx.publish_on::<Auxiliary>(&Row { value: 2, index: (ctx.tick().ordinal() - 1) as i64 }).await?;
            Ok(())
        }
    }

    pub(super) struct Measurements {
        probe: Arc<Probe>,
    }

    #[dokime_runtime::observer(streams(Measurement), activation = "demand", interval_ms = 5)]
    impl Measurements {
        async fn initialize(resource: &Resource) -> Result<Self, Error> {
            resource.0.bump(&resource.0.observer_starts);
            Ok(Self { probe: Arc::clone(&resource.0) })
        }

        async fn observe(&mut self, ctx: &ObserverContext<'_, Resource, Publishing>) -> Result<(), Error> {
            self.probe.bump(&self.probe.samples);
            let index = (ctx.tick().ordinal() - 1) as i64;
            ctx.publish_on::<Measurement>(&Row { value: 1, index }).await?;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), Error> {
            self.probe.bump(&self.probe.observer_stops);
            Ok(())
        }
    }
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{
    Arc, LazyLock,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use std::time::Duration;
// Third Party Imports
use dokime_runtime::harmos::operations::{ServingError, WireTable};
use dokime_runtime::harmos::plugin::Declared;
use dokime_runtime::harmos::sidecar::{DemandKey, ObserverAttachment, ObserverContext, Publisher};
use harmos::Message;
// Local Imports
use dokime_domain::model;
use dokime_runtime::plugin::Configuration;
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */
static PROBES: LazyLock<std::sync::Mutex<BTreeMap<String, Arc<Probe>>>> = LazyLock::new(|| std::sync::Mutex::new(BTreeMap::new()));

const UNIT_NOTED: DemandKey = DemandKey::new("dokime.record", "owned.unit.noted");

/* === Macros === */
/* === Type Aliases === */
type Publishing = Publisher<Measured>;

/* === Structs === */
#[derive(Default)]
struct Probe {
    opens: AtomicUsize,
    samples: AtomicUsize,
    observer_starts: AtomicUsize,
    observer_stops: AtomicUsize,
    record_starts: AtomicUsize,
    record_stops: AtomicUsize,
    auxiliary_samples: AtomicUsize,
    drops: AtomicUsize,
    finalized: std::sync::atomic::AtomicBool,
    published: AtomicUsize,
    active_listeners: AtomicUsize,
    refused: std::sync::Mutex<Option<String>>,
    echoed: AtomicUsize,
    changed: tokio::sync::Notify,
}

struct Listening(Arc<Probe>);

struct Resource(Arc<Probe>);

#[dokime_runtime::section(
    label = "Probe",
    description = "The token a run names its registered [`Probe`] by, and whether this attempt refuses to open — a setting travels \
        as a serialized value, so the live handle crosses as a name and the flag crosses as itself."
)]
pub struct Probes {
    #[setting(
        label = "Probe Token",
        default = "unregistered",
        description = "Never read as its declared default: every run in this file names a real registered token, but a required text \
        setting still has to declare a nonempty one to pass catalog validation."
    )]
    pub token: String,

    #[setting(label = "Refuse")]
    pub refuse: bool,
}

#[dokime_runtime::settings]
pub struct MeasuredSettings {
    pub probes: Probes,
}

struct Measured {
    resource: Arc<Resource>,
    observers: Option<Arc<ObserverAttachment<Resource, Publishing>>>,
}

#[dokime_runtime::record(
    id = "owned.unit.connected",
    label = "Unit connected",
    description = "One fact the host files, which this fixture is told about."
)]
struct UnitConnected {
    #[record(description = "The unit the host says is on the bench.")]
    serial: String,
}

#[dokime_runtime::record(
    id = "owned.unit.noted",
    label = "Unit noted",
    description = "One fact this fixture narrates back, with no run behind it."
)]
struct UnitNoted {
    #[record(description = "The unit the bench took note of.")]
    serial: String,
}

#[dokime_runtime::stream(id = "owned-stream.pushed", window = 64, label = "Pushed", description = "A stream this plugin never declares.")]
struct PushedRow {
    value: i64,
}

struct RecordObserver(Arc<Probe>);

struct Noted;

struct Echoed;

#[dokime_runtime::stream(id = "owned-stream.raw-identifiers", label = "Raw Identifiers", window = 8)]
struct RawIdentifiers {
    #[row(label = "Type", role = state)]
    r#type: String,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Publication(#[from] dokime_runtime::harmos::sidecar::PublishError),
    #[error(transparent)]
    Undeclared(#[from] dokime_runtime::harmos::sidecar::PublicationError),
    #[error(transparent)]
    Published(#[from] dokime_runtime::Error),
    #[error("resource is temporarily unavailable")]
    Unavailable,
}

#[dokime_runtime::catalog]
enum MeasuredObservers {
    Auxiliaries(observers::Auxiliaries),
    Measurements(observers::Measurements),
    Records(RecordObserver),
}

#[dokime_runtime::catalog]
enum MeasuredListeners {
    Noted(Noted),
    Echoed(Echoed),
}

#[dokime_runtime::catalog]
enum MeasuredRecords {
    UnitNoted(UnitNoted),
}

/* === Traits === */

/* === Implementations === */
impl Probe {
    // Counts one event and wakes every story waiting on the probe.
    fn bump(&self, counter: &AtomicUsize) -> usize {
        let previous = counter.fetch_add(1, Ordering::SeqCst);
        self.changed.notify_waiters();
        previous
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

impl Drop for Listening {
    fn drop(&mut self) {
        self.0.active_listeners.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        self.0.bump(&self.0.drops);
    }
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for Measured {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "owned-stream",
    provides = (),
    requires(dokime_runtime::harmos::builtins::streams::Publish),
    contributions(
        dokime_runtime::Records<MeasuredRecords, MeasuredListeners>,
        dokime_runtime::Settings<MeasuredSettings>,
    ),
    observers = MeasuredObservers,
    restart = "never", description = "Owned stream fixture.")]
impl dokime_runtime::Plugin for Measured {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resource;

    fn prepare(config: Configuration) -> Result<Self, Refusal> {
        let settings = dokime_runtime::plugin::refusal(config.settings::<MeasuredSettings>())?;
        let probe = probe(&settings.probes.token);
        probe.bump(&probe.opens);
        if settings.probes.refuse {
            return dokime_runtime::plugin::refusal(Err(Error::Unavailable));
        }
        Ok(Self { resource: Arc::new(Resource(probe)), observers: None })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<MeasuredObservers>(self.resources(), <Self as Declared>::STREAMS)?);
        Ok(())
    }

    fn resources(&self) -> Arc<Resource> {
        Arc::clone(&self.resource)
    }

    fn bind_operations(&self, table: &mut WireTable) -> Result<(), ServingError> {
        let mut dependencies = dokime_runtime::harmos::Dependencies::new();
        dependencies.insert(self.resources()).map_err(|_| ServingError::Catalog)?;
        table.register(dokime_runtime::harmos::RegistrationBatch::new(dependencies).operation::<Noted>().operation::<Echoed>())?;
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
        let probe = &self.resource.0;
        assert_eq!(
            probe.observer_starts.load(Ordering::SeqCst),
            probe.observer_stops.load(Ordering::SeqCst),
            "every observer finishes its stop before plugin cleanup"
        );
        assert_eq!(
            probe.record_starts.load(Ordering::SeqCst),
            probe.record_stops.load(Ordering::SeqCst),
            "every record observer finishes its stop before plugin cleanup"
        );
        self.resource.0.finalized.store(true, Ordering::SeqCst);
        self.resource.0.changed.notify_waiters();
        Ok(())
    }
}

#[dokime_runtime::observer(demand_keys(UNIT_NOTED), activation = "demand", interval_ms = 5)]
impl RecordObserver {
    async fn initialize(resource: &Resource) -> Result<Self, Error> {
        resource.0.bump(&resource.0.record_starts);
        Ok(Self(Arc::clone(&resource.0)))
    }

    async fn observe(&mut self, ctx: &ObserverContext<'_, Resource, Publishing>) -> Result<(), Error> {
        dokime_runtime::plugin::Publisher::new(ctx.host().supervisor()).publish(UnitNoted { serial: "demand".into() }).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Error> {
        self.0.bump(&self.0.record_stops);
        Ok(())
    }
}

#[dokime_runtime::listener(id = "owned_stream.noted", record = UnitConnected, resources = [Resource])]
impl Noted {
    async fn hear(ctx: &dokime_runtime::plugin::ListenerContext<'_, Self>, connected: UnitConnected) -> Result<(), Error> {
        let resource = ctx.resource::<Resource>();
        let probe = &resource.0;
        let host = &*ctx.host();
        probe.bump(&probe.active_listeners);
        let _listening = Listening(Arc::clone(probe));
        if connected.serial == "STALL" {
            std::future::pending::<()>().await;
        }
        // A row on a stream this plugin never declared is refused before it
        // reaches the connection, through either publisher.
        assert!(matches!(
            host.publisher::<Measured>().publish_on::<PushedRow>(&PushedRow { value: 1 }).await,
            Err(dokime_runtime::harmos::sidecar::PublishError::Publication(
                dokime_runtime::harmos::sidecar::PublicationError::Undeclared { .. }
            ))
        ));
        assert!(matches!(
            dokime_runtime::plugin::Publisher::new(host).publish(PushedRow { value: 1 }).await,
            Err(dokime_runtime::Error::Publish(dokime_runtime::error::PublishFailure::Undeclared { .. }))
        ));
        host.publisher::<Measured>()
            .publish_on::<observers::Measurement>(&observers::Row { value: 73, index: 9 })
            .await
            .expect("a listener publishes a typed row without an observer lease");
        match dokime_runtime::plugin::Publisher::new(host).publish(UnitNoted { serial: connected.serial }).await {
            Ok(()) => probe.bump(&probe.published),
            Err(refusal) => {
                *probe.refused.lock().expect("the probe lock is healthy") = Some(refusal.to_string());
                0
            }
        };

        Ok(())
    }
}

#[dokime_runtime::listener(id = "owned_stream.echoed", record = UnitConnected, resources = [Resource])]
impl Echoed {
    async fn hear(ctx: &dokime_runtime::plugin::ListenerContext<'_, Self>, connected: UnitConnected) -> Result<(), Error> {
        assert_eq!(connected.serial, "SN-7", "the record decodes as its type");
        let probe = &ctx.resource::<Resource>().0;
        probe.bump(&probe.echoed);

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

fn configuration(probe: &Arc<Probe>, refuse: bool) -> Configuration {
    Configuration {
        settings: BTreeMap::from([
            ("token".to_owned(), dokime_runtime::Value::Text(registered(probe))),
            ("refuse".to_owned(), dokime_runtime::Value::Bool(refuse)),
        ]),
        ..Configuration::default()
    }
}

async fn opened(
    probe: &Arc<Probe>, demand_keys: Vec<String>,
) -> super::native::Native<dokime_runtime::harmos::sidecar::PluginNative<Measured>> {
    super::native::Native::initialize(
        super::native::attach::<Measured>(),
        <Measured as Declared>::VERSION,
        serde_json::to_vec(&configuration(probe, false)).unwrap(),
        |_, _| {},
        demand_keys,
    )
    .await
}

fn connected(serial: &str) -> dokime_runtime::plugin::Delivery {
    let mut filed = model::records::evidence::Record::new(
        dokime_runtime::model::identity::new_id(),
        dokime_runtime::model::time::now_utc(),
        <UnitConnected as dokime_runtime::records::DeclaredRecord>::record_type(),
        1,
        "gateway",
        dokime_runtime::records::RecordSeverity::Info,
    );
    filed.details = [("serial".to_owned(), dokime_runtime::Value::from(serial))].into();
    dokime_runtime::plugin::Delivery::of(&filed)
}

/* === Tests === */

#[tokio::test]
async fn instance_opens_resources() {
    let probe = Arc::new(Probe::default());
    let native = opened(&probe, vec![]).await;
    let runtime = &native.runtime;
    assert_eq!(probe.opens.load(Ordering::SeqCst), 1);
    assert_eq!(probe.samples.load(Ordering::SeqCst), 0, "activation does not start observations");

    let measurement = runtime.streams.publication("owned-stream.measurement", 1).unwrap();
    let auxiliary = runtime.streams.publication("owned-stream.auxiliary", 1).unwrap();
    let first = measurement.watch();
    let second = measurement.watch();
    probe.until(|probe| probe.samples.load(Ordering::SeqCst) != 0).await;
    assert_eq!(probe.auxiliary_samples.load(Ordering::SeqCst), 0);
    assert_eq!(probe.observer_starts.load(Ordering::SeqCst), 1);
    drop(first);
    assert_eq!(probe.observer_stops.load(Ordering::SeqCst), 0);
    let auxiliary_hold = auxiliary.watch();
    probe.until(|probe| probe.auxiliary_samples.load(Ordering::SeqCst) != 0).await;

    drop(second);
    probe.until(|probe| probe.observer_stops.load(Ordering::SeqCst) != 0).await;
    let restarted = measurement.watch();
    probe.until(|probe| probe.observer_starts.load(Ordering::SeqCst) == 2).await;

    native.shutdown().await;
    drop((restarted, auxiliary_hold));
    assert_eq!(probe.observer_stops.load(Ordering::SeqCst), 2);
    assert!(probe.finalized.load(Ordering::SeqCst));
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    let settled = (probe.samples.load(Ordering::SeqCst), probe.auxiliary_samples.load(Ordering::SeqCst));
    // Shutdown Joins Every Observer Before Returning
    assert_eq!(
        (probe.samples.load(Ordering::SeqCst), probe.auxiliary_samples.load(Ordering::SeqCst),),
        settled,
        "a joined stream measures nothing further"
    );
}

#[test]
fn plugin_refuses_line() {
    let probe = Arc::new(Probe::default());
    let Err(refused) = <Measured as dokime_runtime::Plugin>::prepare(configuration(&probe, true)) else {
        panic!("a plugin that cannot open its resource refuses to start");
    };

    // A refusal is a message on a wire, and the host copies it into the failure
    // record an operator reads in a list. The author's own words belong there;
    // the frames behind them belong in the log.
    let reported = refused.to_string();
    assert!(reported.contains("resource is temporarily unavailable"), "{reported}");
    assert!(!reported.contains('\n'), "{reported}");
    assert!(!reported.contains("backtrace"), "{reported}");
    assert_eq!(probe.drops.load(Ordering::SeqCst), 0, "a refused preparation never opened the resource it would have to close");
}

#[tokio::test(start_paused = true)]
async fn listener_publishes_asked() {
    let probe = Arc::new(Probe::default());
    let native = opened(&probe, vec![]).await;
    let runtime = &native.runtime;

    let mut invocation = runtime
        .operations
        .submit_encoded(
            <Noted as dokime_runtime::harmos::operations::OperationDeclaration>::ID,
            connected("STALL").encode(),
        )
        .unwrap();
    // A Paused Clock Advances Without Turning Pending Delivery Into A Success
    assert!(futures_util::FutureExt::now_or_never(&mut invocation).is_none());
    tokio::time::advance(Duration::from_millis(100)).await;
    assert!(futures_util::FutureExt::now_or_never(&mut invocation).is_none());
    invocation.cancel();
    let cancelled = invocation.await;
    assert!(matches!(cancelled, Err(harmos::Error::OperationRefused(harmos::Refusal::Cancelled))), "{cancelled:?}");
    super::native::call_wire::<Noted>(&runtime.operations, connected("SN-7")).await.unwrap();
    super::native::call_wire::<Echoed>(&runtime.operations, connected("SN-7")).await.unwrap();
    if let Some(refusal) = probe.refused.lock().unwrap().clone() {
        panic!("the listener could not publish: {refusal}");
    }
    assert_eq!(probe.published.load(Ordering::SeqCst), 1);
    assert_eq!(probe.echoed.load(Ordering::SeqCst), 1);
    assert_eq!(probe.active_listeners.load(Ordering::SeqCst), 0, "the cancelled reaction cannot survive delivery cancellation");
    let filed = native.host.records();
    assert_eq!(filed.len(), 1, "the record reaches the host's filing operation");
    assert_eq!(filed[0].0, "owned-stream");
    assert_eq!(filed[0].1.record_type.as_str(), "owned.unit.noted");

    // TODO(source-hook): a wait for a published row to reach its stream window without engaging demand.
    tokio::time::timeout(Duration::from_secs(120), async {
        loop {
            let stream = runtime.streams.publication("owned-stream.measurement", 1);
            if let Some(stream) = stream {
                let rows = stream.query(..).await.unwrap().collect::<Vec<_>>();
                if let Some(row) = rows.first() {
                    assert_eq!(rows.len(), 1, "no observer was leased");
                    assert!(row.value.context.is_none(), "the row is process scoped");
                    assert!(row.value.published_unix_micros.is_some());
                    let decoded = observers::Row::decode(row.value.payload.as_slice()).unwrap();
                    assert_eq!((decoded.value, decoded.index), (73, 9));
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("typed protobuf telemetry reaches Harmos capture");

    native.shutdown().await;
}

#[test]
fn raw_field_keys() {
    use dokime_runtime::telemetry::TelemetryRow;

    let columns = RawIdentifiers::columns();
    assert_eq!(columns[0].metadata.identifier.as_str(), "type");
    assert_eq!(RawIdentifiers::column_type().field(), "type");

    let values = RawIdentifiers { r#type: "probe".to_owned() }.into_row();
    assert_eq!(values["type"], dokime_runtime::Value::Text("probe".into()));
    assert!(!values.contains_key("r#type"));
    assert_eq!(RawIdentifiers::from_row(&values).unwrap().r#type, "probe");
}

#[tokio::test]
async fn record_demand_cancel() {
    let probe = Arc::new(Probe::default());
    let native = opened(&probe, vec![UNIT_NOTED.name()]).await;
    let demands = native.demands.clone();
    assert_eq!(probe.record_starts.load(Ordering::SeqCst), 0);
    assert!(demands.hold("dokime.record:unknown").is_err());
    let first = demands.hold(&UNIT_NOTED.name()).unwrap();
    let second = demands.hold(&UNIT_NOTED.name()).unwrap();
    native.host.first_filed().await;
    let (owner, filed) = native.host.records().remove(0);
    assert_eq!(owner, "owned-stream");
    assert_eq!(filed.record_type.as_str(), "owned.unit.noted");
    assert_eq!(probe.record_starts.load(Ordering::SeqCst), 1);
    assert_eq!(probe.observer_starts.load(Ordering::SeqCst), 0);
    assert_eq!(probe.auxiliary_samples.load(Ordering::SeqCst), 0);
    drop(first);
    // TODO(source-hook): a demand event that reports a released hold left the record source running.
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(probe.record_stops.load(Ordering::SeqCst), 0);
    drop(second);
    probe.until(|probe| probe.record_stops.load(Ordering::SeqCst) == 1).await;

    let (started, opened) = tokio::sync::oneshot::channel();
    let waiting = tokio::spawn(async move {
        let _hold = demands.hold(&UNIT_NOTED.name()).unwrap();
        started.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    opened.await.unwrap();
    probe.until(|probe| probe.record_starts.load(Ordering::SeqCst) == 2).await;
    waiting.abort();
    assert!(waiting.await.unwrap_err().is_cancelled());
    probe.until(|probe| probe.record_stops.load(Ordering::SeqCst) == 2).await;
    native.shutdown().await;
}
