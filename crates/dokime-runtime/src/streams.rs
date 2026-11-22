/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

#[cfg(test)]
mod burst;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeSet, HashMap},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime},
};

// Third Party Imports
use async_trait::async_trait;
use harmos_domain::schema::json::JsonCodec;
use harmos_protocol::model::Declarations;
use harmos_runtime::BackendFailure;
use harmos_runtime::operations::OperationOutcome;
use harmos_runtime::operations::middleware::{AttemptObservation, Enter, Invocation, Middleware, Observation};
use harmos_runtime::streams::{Definition, Gap, PublicationRow};
use harmos_runtime::{
    clock::{self, Controller},
    storage::{Batch, Cursor, StreamConsumer},
};
use prost::Message;
use prost_reflect::{self, DescriptorPool, DynamicMessage, Kind, MapKey, MessageDescriptor};
use prost_types::Timestamp;
use serde_json::Value;
use tokio::sync::watch::{self, Sender};

// Local Imports
use crate::services::composition::Application;
use crate::{Error, SchedulerClock};
use dokime_operations::{Clock, Resources, Result};
use dokime_storage::{Evidence, Sample, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct ExecutionTime {
    pub clock: clock::Clock,
    pub sleeping: Sender<BTreeSet<u64>>,
}

pub(crate) struct Samples {
    pub store: Store,
    pub cursor: Cursor,
    pub failures: Arc<Mutex<HashMap<String, String>>>,
    pub scopes: Arc<Mutex<HashMap<String, SampleScope>>>,
}

/// The Storage Side of One Delivery, Moved Off the Async Delivery Task.
struct Intake {
    store: Store,
    failures: Arc<Mutex<HashMap<String, String>>>,
    scopes: Arc<Mutex<HashMap<String, SampleScope>>>,
}

pub(crate) struct SampleScope {
    streams: HashMap<String, SampleRow>,
}
struct SampleRow {
    descriptor: MessageDescriptor,
    codec: Option<JsonCodec>,
    /// Interval Streams Store `harmos.batch` Rows of Columns; Device Streams Store One Row per Sample.
    interval: bool,
    sequence: u64,
}

pub(crate) struct Timing;

pub(crate) struct Attempts {
    started: AtomicBool,
    start_ns: Mutex<i64>,
}

impl Attempts {
    pub fn new(start_ns: i64) -> Self {
        Self {
            started: AtomicBool::new(false),
            start_ns: Mutex::new(start_ns),
        }
    }
    fn start(&self, resources: &Resources) -> Result<()> {
        if !self.started.swap(true, Ordering::SeqCst) {
            let at = *self.start_ns.lock().expect("attempt clock lock is healthy");
            resources.evidence("attempt:start", Vec::new(), Some(at))?;
        }
        Ok(())
    }
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Middleware for Timing {
    fn enter<'a>(&'a self, invocation: &'a mut Invocation) -> Enter<'a> {
        Box::pin(async move {
            if let Some(attempts) = invocation.context().get::<Arc<Attempts>>().cloned()
                && let Some(resources) = invocation.context().get::<Resources>().cloned()
            {
                attempts
                    .start(&resources)
                    .map_err(|source| harmos_runtime::Error::backend("Record Invocation Attempt", BackendFailure::Permanent, source))?;
            }
            if invocation.operation().id.as_str() == "harmos.plugin.v1.Time/Sleep"
                && let Some(time) = invocation.context().get::<Arc<ExecutionTime>>()
            {
                time.sleeping.send_modify(|active| {
                    active.insert(invocation.id().value());
                });
            }
            Ok(())
        })
    }
    fn attempt<'a>(&'a self, invocation: &'a Invocation, observation: AttemptObservation) -> Enter<'a> {
        Box::pin(async move {
            if let Some(attempts) = invocation.context().get::<Arc<Attempts>>()
                && let Some(resources) = invocation.context().get::<Resources>().cloned()
                && invocation.operation().id.as_str() == resources.operation
            {
                resources.attempt.store(observation.attempt as u32 + 1, Ordering::SeqCst);
                attempts
                    .start(&resources)
                    .map_err(|source| harmos_runtime::Error::backend("Record Invocation Attempt", BackendFailure::Permanent, source))?;
                let kind = if observation.outcome == OperationOutcome::Succeeded { "attempt:succeeded" } else { "attempt:failed" };
                resources
                    .evidence(kind, Vec::new(), None)
                    .map_err(|source| harmos_runtime::Error::backend("Record Invocation Attempt", BackendFailure::Permanent, source))?;
                *attempts.start_ns.lock().expect("attempt clock lock is healthy") = resources.clock.now_ns();
                resources.attempt.store(observation.attempt as u32 + 2, Ordering::SeqCst);
                attempts.started.store(false, Ordering::SeqCst);
            }
            if invocation.operation().id.as_str() == "harmos.plugin.v1.Time/Sleep"
                && let Some(time) = invocation.context().get::<Arc<ExecutionTime>>()
            {
                time.sleeping.send_modify(|active| {
                    active.remove(&invocation.id().value());
                });
            }
            Ok(())
        })
    }
    fn observe(&self, invocation: &Invocation, _: Observation) {
        if invocation.operation().id.as_str() == "harmos.plugin.v1.Time/Sleep"
            && let Some(time) = invocation.context().get::<Arc<ExecutionTime>>()
        {
            time.sleeping.send_modify(|active| {
                active.remove(&invocation.id().value());
            });
        }
    }
}

impl ExecutionTime {
    pub fn open(source: &dyn SchedulerClock) -> (Arc<Self>, Option<Controller>) {
        let (clock, controller) = source.execution_clock();
        (
            Arc::new(Self {
                clock,
                sleeping: watch::channel(BTreeSet::new()).0,
            }),
            controller,
        )
    }
}

#[async_trait]
impl Clock for ExecutionTime {
    fn now_ns(&self) -> i64 {
        self.clock
            .date()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|value| value.as_nanos() as i64)
            .unwrap_or_else(|value| -(value.duration().as_nanos() as i64))
    }
    async fn sleep(&self, duration_ns: i64) -> Result<()> {
        let duration = u64::try_from(duration_ns).map_err(|source| dokime_operations::Error::Invalid(format!("Sleep: {source}")))?;
        self.clock.sleep(Duration::from_nanos(duration)).await;
        Ok(())
    }
}

impl StreamConsumer<Application> for Samples {
    fn resume(&self) -> Cursor {
        self.cursor
    }
    async fn drain(&mut self, batch: Batch<Application>) -> Cursor {
        let reaches = batch.reaches();
        let intake = Intake {
            store: self.store.clone(),
            failures: self.failures.clone(),
            scopes: self.scopes.clone(),
        };
        // Storage Admission Takes Locks and Can Flush Under Byte Pressure, So It Runs Off the Async Delivery Task.
        if let Err(error) = tokio::task::spawn_blocking(move || intake.drain(&batch)).await {
            // A Delivery Whose Admission Failed Lost Its Rows, So Every Sampling Execution Fails.
            let scopes = self.scopes.lock().expect("sample scope lock is healthy");
            let mut failures = self.failures.lock().expect("sample failure lock is healthy");
            for execution in scopes.keys() {
                let action = "Ingest Execution Samples: Storage Admission Failed; See the Host Log, Then Rerun the Execution".to_owned();
                tracing::error!(%execution, %error, "Ingest Execution Samples");
                failures.insert(execution.clone(), action);
            }
        }
        self.cursor = reaches;
        self.cursor
    }
}

impl SampleScope {
    pub fn new(catalog: &Declarations, streams: &[String]) -> crate::Result<Self> {
        let types = catalog.types.clone().ok_or_else(|| Error::Invalid("Open Sample Scope: Declare Its Row Types".into()))?;
        let pool =
            DescriptorPool::from_file_descriptor_set(types).map_err(|source| Error::Invalid(format!("Open Sample Scope: {source}")))?;
        let mut selected = HashMap::new();
        for stream in streams {
            let declaration = catalog
                .streams
                .iter()
                .find(|declaration| declaration.id == *stream)
                .ok_or_else(|| Error::Invalid(format!("Open Sample Scope: Declare Stream {stream}")))?;
            let descriptor = pool
                .get_message_by_name(&declaration.row)
                .ok_or_else(|| Error::Invalid(format!("Open Sample Scope: Declare Row {}", declaration.row)))?;
            let interval =
                declaration.collection.is_empty() && (declaration.interval.is_some() || !declaration.interval_setting.is_empty());
            selected.insert(
                stream.clone(),
                SampleRow {
                    descriptor,
                    codec: None,
                    interval,
                    sequence: 0,
                },
            );
        }
        Ok(Self { streams: selected })
    }
}

impl Intake {
    fn drain(&self, batch: &Batch<Application>) {
        // A Delivery That Lost Rows Before Storage Read Them Fails Every Execution Sampling That Stream.
        for definition in batch.incomplete() {
            let scopes = self.scopes.lock().expect("sample scope lock is healthy");
            let mut failures = self.failures.lock().expect("sample failure lock is healthy");
            for (execution, _) in scopes.iter().filter(|(_, scope)| scope.streams.contains_key(definition.id())) {
                let action = format!(
                    "Ingest Execution Samples: Stream {} Lost Rows Before Storage Read Them; Shorten Its Window or Lower Its Rate, Then \
                     Rerun the Execution",
                    definition.id()
                );
                tracing::error!(%execution, %action, "Ingest Execution Samples");
                failures.insert(execution.clone(), action);
            }
        }
        for (definition, _, publication) in batch.publications() {
            let result = self.ingest(definition, publication.value.row(), publication.value.gaps());
            if let Err(source) = result {
                let scope = publication
                    .value
                    .context()
                    .and_then(|context| serde_json::from_str::<Value>(context).ok())
                    .and_then(|context| context["scope"].as_str().map(str::to_owned));
                if let Some(scope) = scope {
                    let action = format!("Ingest Execution Samples: Stream {}: {source}; Rerun the Execution", definition.id());
                    self.failures.lock().expect("sample failure lock is healthy").insert(scope, action);
                }
                tracing::error!(%source, "Ingest Execution Samples");
            }
        }
    }
    fn ingest(&self, definition: &Definition, publication: &PublicationRow, gaps: &[Gap]) -> crate::Result<()> {
        let Some(context) = &publication.context else { return Ok(()) };
        let context: Value = serde_json::from_str(context).map_err(|source| Error::Invalid(format!("Ingest Sample Scope: {source}")))?;
        let Some(execution) = context["scope"].as_str() else { return Ok(()) };
        let mut scopes = self.scopes.lock().map_err(|source| Error::Invalid(format!("Ingest Sample Scope: {source}")))?;
        let scope = scopes.get_mut(execution).ok_or_else(|| Error::Invalid("Ingest Sample Scope: Retain Its Execution Binding".into()))?;
        let Some(stream) = scope.streams.get_mut(definition.id()) else { return Ok(()) };
        for gap in gaps {
            let [start_ns, end_ns] = [gap.time.start, gap.time.end].map(|time| {
                let time = Timestamp::from(time);
                time.seconds.saturating_mul(1_000_000_000).saturating_add(i64::from(time.nanos))
            });
            let payload = serde_json::json!({"stream": definition.id(), "missed": gap.missed, "start_ns": start_ns, "end_ns": end_ns});
            self.store.append(
                execution,
                Evidence {
                    kind: "streams:gap".into(),
                    source_ns: start_ns,
                    receipt_ns: publication.published_unix_micros.unwrap_or_default().saturating_mul(1000),
                    payload: payload.to_string().into_bytes(),
                    ..Default::default()
                },
            )?;
        }
        if stream.interval {
            // One Stored Batch Row Holds Consecutive Samples; Storage Expands It Column by Column.
            stream.sequence += self.store.sample_interval(execution, definition.id(), &publication.payload, stream.sequence + 1)?;
            return Ok(());
        }
        if stream.codec.is_none() {
            let schema = definition.schema();
            let view = schema.view(schema.root).map_err(|source| Error::Invalid(format!("Decode Sample Schema: {source}")))?;
            let codec = JsonCodec::new(view, &Default::default()).map_err(|source| Error::Invalid(format!("Decode Sample: {source}")))?;
            stream.codec = Some(codec);
        }
        let value = stream
            .codec
            .as_ref()
            .unwrap()
            .decode(&publication.payload)
            .map_err(|source| Error::Invalid(format!("Decode Sample: {source}")))?;
        let row = message(stream.descriptor.clone(), &value)?.encode_to_vec();
        let receipt_ns = publication.published_unix_micros.unwrap_or_default().saturating_mul(1000);
        let source_ns = context
            .get("source_time")
            .map(|at| at["seconds"].as_i64().unwrap_or_default().saturating_mul(1_000_000_000) + at["nanos"].as_i64().unwrap_or_default())
            .unwrap_or(receipt_ns);
        stream.sequence += 1;
        self.store.sample(
            execution,
            Sample {
                stream: definition.id().into(),
                sequence: stream.sequence,
                source_ns,
                receipt_ns,
                row,
                ..Default::default()
            },
        )?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn message(descriptor: MessageDescriptor, value: &Value) -> crate::Result<DynamicMessage> {
    let mut message = DynamicMessage::new(descriptor.clone());
    for field in descriptor.fields() {
        let Some(value) = value.get(field.name()).filter(|value| !value.is_null()) else { continue };
        let converted = if field.is_map() {
            let Kind::Message(entry) = field.kind() else { unreachable!() };
            let key = entry.get_field_by_name("key").unwrap();
            let item = entry.get_field_by_name("value").unwrap();
            let entries = value.as_array().ok_or_else(|| Error::Invalid("Decode Sample: Expected a Map".into()))?;
            let mut values = HashMap::new();
            for entry in entries {
                let key = match scalar(&key.kind(), &entry["key"])? {
                    prost_reflect::Value::Bool(value) => MapKey::Bool(value),
                    prost_reflect::Value::I32(value) => MapKey::I32(value),
                    prost_reflect::Value::I64(value) => MapKey::I64(value),
                    prost_reflect::Value::U32(value) => MapKey::U32(value),
                    prost_reflect::Value::U64(value) => MapKey::U64(value),
                    prost_reflect::Value::String(value) => MapKey::String(value),
                    _ => return Err(Error::Invalid("Decode Sample: Invalid Map Key".into())),
                };
                let value = if entry["value"].is_null() {
                    prost_reflect::Value::default_value(&item.kind())
                } else {
                    scalar(&item.kind(), &entry["value"])?
                };
                values.insert(key, value);
            }
            prost_reflect::Value::Map(values)
        } else if field.is_list() {
            let values = value.as_array().ok_or_else(|| Error::Invalid("Decode Sample: Expected a Repeated Field".into()))?;
            prost_reflect::Value::List(values.iter().map(|value| scalar(&field.kind(), value)).collect::<crate::Result<_>>()?)
        } else {
            scalar(&field.kind(), value)?
        };
        message
            .try_set_field(&field, converted)
            .map_err(|source| Error::Invalid(format!("Decode Sample Field {}: {source}", field.name())))?;
    }
    Ok(message)
}

fn scalar(kind: &Kind, value: &Value) -> crate::Result<prost_reflect::Value> {
    let invalid = || Error::Invalid("Decode Sample: Value Does Not Match Its Declared Field".into());
    Ok(match kind {
        Kind::Bool => prost_reflect::Value::Bool(value.as_bool().ok_or_else(invalid)?),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => {
            prost_reflect::Value::I32(i32::try_from(value.as_i64().ok_or_else(invalid)?).map_err(|_| invalid())?)
        }
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => prost_reflect::Value::I64(value.as_i64().ok_or_else(invalid)?),
        Kind::Uint32 | Kind::Fixed32 => {
            prost_reflect::Value::U32(u32::try_from(value.as_u64().ok_or_else(invalid)?).map_err(|_| invalid())?)
        }
        Kind::Uint64 | Kind::Fixed64 => prost_reflect::Value::U64(value.as_u64().ok_or_else(invalid)?),
        Kind::Float => prost_reflect::Value::F32(value.as_f64().ok_or_else(invalid)? as f32),
        Kind::Double => prost_reflect::Value::F64(value.as_f64().ok_or_else(invalid)?),
        Kind::String => prost_reflect::Value::String(value.as_str().ok_or_else(invalid)?.into()),
        Kind::Bytes => prost_reflect::Value::Bytes(
            value
                .as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(|value| u8::try_from(value.as_u64().ok_or_else(invalid)?).map_err(|_| invalid()))
                .collect::<crate::Result<Vec<_>>>()?
                .into(),
        ),
        Kind::Enum(_) => prost_reflect::Value::EnumNumber(i32::try_from(value.as_i64().ok_or_else(invalid)?).map_err(|_| invalid())?),
        Kind::Message(descriptor) => prost_reflect::Value::Message(message(descriptor.clone(), value)?),
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Declarations, Definition, ExecutionTime, Intake, JsonCodec, PublicationRow, SampleScope, Store, Timing};
    use crate::ControlledClock;
    use dokime_protocol::model::{Import, Origin, origin};
    use dokime_storage::Execution;
    use harmos_domain::schema::Cardinality;
    use harmos_domain::schema::graph::{Field, Kind, Schema};
    use harmos_operations::facilities::{TimeGet, TimeSleep};
    use harmos_protocol::host::{TimeGetRequest, TimeSleepRequest};
    use harmos_protocol::model::StreamDeclaration;
    use harmos_runtime::operations::{self, CallContext};
    use harmos_runtime::state::Never;
    use harmos_runtime::{Application, Lifecycle, Runtime, Setup, Startup};
    use prost::Message;
    use std::collections::HashMap;
    use std::convert::Infallible;
    use std::error::Error;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::time::Duration;
    use tokio::sync::watch::error::RecvError;

    #[derive(dokime::Row)]
    #[dokime(package = "fixture")]
    struct Collected {
        value: u64,
    }

    struct Model;

    impl Lifecycle for Model {
        type Config = ();
        type Error = Infallible;
        type Context = Startup<Self>;

        async fn prepare(_: (), _: &mut Setup<Self>) -> Result<Self, Self::Error> {
            Ok(Self)
        }
    }

    impl Application for Model {
        type State = ();
        type Transaction = Never;
        type Record = ();
    }

    #[test]
    fn collection_rows_retain_values_and_tick_times_in_execution_storage() -> Result<(), Box<dyn Error>> {
        // Open a Real Execution With an Interval Collection's Individual Row Contract.
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let catalog = Declarations {
            types: Some(<Collected as dokime::encoding::contract::Message>::types()),
            streams: vec![StreamDeclaration {
                id: "readings".into(),
                row: <Collected as dokime::encoding::contract::Message>::name(),
                interval: Some(prost_types::Duration { seconds: 1, nanos: 0 }),
                collection: "readings".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let execution = Execution {
            identifier: "execution".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::Import(Import::default())),
            }),
            catalog: catalog.encode_to_vec(),
            ..Default::default()
        };
        store.create(&execution)?;
        store.open_stream("execution", "readings")?;
        let scope = SampleScope::new(&catalog, &["readings".into()])?;
        let intake = Intake {
            store: store.clone(),
            scopes: Arc::new(Mutex::new(HashMap::from([("execution".into(), scope)]))),
            failures: Arc::new(Mutex::new(HashMap::new())),
        };

        // Deliver Native Rows From Two Ticks, With Deterministic Source and Receipt Times.
        let schema = Schema {
            root: 0,
            definitions: vec![vec![Field {
                name: "value".into(),
                tag: 1,
                kind: Kind::U64,
                cardinality: Cardinality::Required,
            }]],
            representations: Default::default(),
        };
        let definition = Definition::declared::<PublicationRow>("readings", schema, 8)?.publication("fixture");
        let schema = definition.schema();
        let codec = JsonCodec::new(schema.view(schema.root)?, &Default::default())?;
        for (index, value) in [13_u64, 21].into_iter().enumerate() {
            let publication = PublicationRow {
                payload: codec.encode(&serde_json::json!({"value": value}))?,
                published_unix_micros: Some(10 + index as i64 * 1_000_000),
                context: Some(serde_json::json!({"scope": "execution", "source_time": {"seconds": index, "nanos": 100}}).to_string()),
            };
            intake.ingest(&definition, &publication, &[])?;
        }

        // Read the Stored Protobuf Rows and Their Exact Tick Times, Without Batch Expansion.
        store.flush_samples()?;
        let through = store.cut("execution")?.frontier;
        let saved = store.sample_page("execution", "readings", 0, through, NonZeroUsize::new(4).unwrap())?;
        let mut observed = Vec::new();
        for row in saved {
            observed.push((row.sequence, row.source_ns, row.receipt_ns, Collected::decode(row.row.as_slice())?.value));
        }
        assert_eq!(observed, [(1, 100, 10_000, 13), (2, 1_000_000_100, 1_000_010_000, 21)]);
        Ok(())
    }

    #[tokio::test]
    async fn plugin_clock_facilities_advance_virtual_execution_time() -> Result<(), Box<dyn Error>> {
        // Assemble a Virtual Execution Clock and Its Actual Timing Middleware.
        let (time, controller) = ExecutionTime::open(&ControlledClock::new(7_000_000_000));
        let controller = controller.unwrap();
        let runtime = Runtime::<Model>::start_with((), (), |setup| {
            setup.settings().clock(time.clock.clone()).operation_middleware(Arc::new(Timing));
            setup.settings().operation_settings(operations::Settings {
                preparation_bytes: Some(NonZeroUsize::new(4 * 1024 * 1024).unwrap()),
                ..Default::default()
            });
        })
        .await?;
        let mut context = CallContext::default();
        context.insert(time.clone())?;
        let operations = runtime.operations.with_context(context);
        let before = operations.call_protobuf::<TimeGet>(TimeGetRequest {}).await?.now.unwrap();
        let mut sleeping = time.sleeping.subscribe();

        // Advance Only When the Plugin Sleep Has Entered the Execution's Sleeping Set.
        let duration = Duration::from_secs(2);
        let sleep = operations.call_protobuf::<TimeSleep>(TimeSleepRequest { duration: Some(duration.try_into()?) });
        let advance = async {
            while sleeping.borrow_and_update().is_empty() {
                sleeping.changed().await?;
            }
            controller.advance_by(duration).await;
            Ok::<(), RecvError>(())
        };
        let (slept, advanced) = tokio::join!(sleep, advance);
        slept?;
        advanced?;

        // Read the Same Plugin Clock and Prove Settlement Removed the Sleeping Invocation.
        let after = operations.call_protobuf::<TimeGet>(TimeGetRequest {}).await?.now.unwrap();
        assert_eq!(after.seconds - before.seconds, duration.as_secs() as i64);
        assert_eq!(after.nanos, before.nanos);
        assert!(time.sleeping.borrow().is_empty());
        runtime.stop().await?;
        Ok(())
    }
}
