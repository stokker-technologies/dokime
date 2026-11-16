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
use harmos_runtime::operations::OperationOutcome;
use harmos_runtime::operations::middleware::{AttemptObservation, Enter, Invocation, Middleware, Observation};
use harmos_runtime::streams::{Definition, PublicationRow};
use harmos_runtime::{BackendFailure, Error as RuntimeError};
use harmos_runtime::{
    clock::{Clock as RuntimeClock, Controller},
    storage::{Batch, Cursor, StreamConsumer},
};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, Kind, MapKey, MessageDescriptor, Value as FieldValue};
use serde_json::Value;
use tokio::sync::watch::{self, Sender};

// Local Imports
use crate::services::composition::Application;
use crate::{Error, Result as HostResult, SchedulerClock};
use dokime_operations::{Clock, Error as OperationError, Resources, Result};
use dokime_storage::{Sample, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct ExecutionTime {
    pub clock: RuntimeClock,
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
                    .map_err(|source| RuntimeError::backend("Record Invocation Attempt", BackendFailure::Permanent, source))?;
            }
            if invocation.operation().id.as_str() == "harmos.host.v1.Time/Sleep"
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
                    .map_err(|source| RuntimeError::backend("Record Invocation Attempt", BackendFailure::Permanent, source))?;
                let kind = if observation.outcome == OperationOutcome::Succeeded { "attempt:succeeded" } else { "attempt:failed" };
                resources
                    .evidence(kind, Vec::new(), None)
                    .map_err(|source| RuntimeError::backend("Record Invocation Attempt", BackendFailure::Permanent, source))?;
                *attempts.start_ns.lock().expect("attempt clock lock is healthy") = resources.clock.now_ns();
                resources.attempt.store(observation.attempt as u32 + 2, Ordering::SeqCst);
                attempts.started.store(false, Ordering::SeqCst);
            }
            if invocation.operation().id.as_str() == "harmos.host.v1.Time/Sleep"
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
        if invocation.operation().id.as_str() == "harmos.host.v1.Time/Sleep"
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
        let duration = u64::try_from(duration_ns).map_err(|source| OperationError::Invalid(format!("Sleep: {source}")))?;
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
    pub fn new(catalog: &Declarations, streams: &[String]) -> HostResult<Self> {
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
            let interval = declaration.interval.is_some() || !declaration.interval_setting.is_empty();
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
            let result = self.ingest(definition, publication.value);
            if let Err(source) = result {
                let scope = publication
                    .value
                    .context
                    .as_ref()
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
    fn ingest(&self, definition: &Definition, publication: &PublicationRow) -> HostResult<()> {
        let Some(context) = &publication.context else { return Ok(()) };
        let context: Value = serde_json::from_str(context).map_err(|source| Error::Invalid(format!("Ingest Sample Scope: {source}")))?;
        let Some(execution) = context["scope"].as_str() else { return Ok(()) };
        let mut scopes = self.scopes.lock().map_err(|source| Error::Invalid(format!("Ingest Sample Scope: {source}")))?;
        let scope = scopes.get_mut(execution).ok_or_else(|| Error::Invalid("Ingest Sample Scope: Retain Its Execution Binding".into()))?;
        let Some(stream) = scope.streams.get_mut(definition.id()) else { return Ok(()) };
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

fn message(descriptor: MessageDescriptor, value: &Value) -> HostResult<DynamicMessage> {
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
                    FieldValue::Bool(value) => MapKey::Bool(value),
                    FieldValue::I32(value) => MapKey::I32(value),
                    FieldValue::I64(value) => MapKey::I64(value),
                    FieldValue::U32(value) => MapKey::U32(value),
                    FieldValue::U64(value) => MapKey::U64(value),
                    FieldValue::String(value) => MapKey::String(value),
                    _ => return Err(Error::Invalid("Decode Sample: Invalid Map Key".into())),
                };
                let value =
                    if entry["value"].is_null() { FieldValue::default_value(&item.kind()) } else { scalar(&item.kind(), &entry["value"])? };
                values.insert(key, value);
            }
            FieldValue::Map(values)
        } else if field.is_list() {
            let values = value.as_array().ok_or_else(|| Error::Invalid("Decode Sample: Expected a Repeated Field".into()))?;
            FieldValue::List(values.iter().map(|value| scalar(&field.kind(), value)).collect::<HostResult<_>>()?)
        } else {
            scalar(&field.kind(), value)?
        };
        message
            .try_set_field(&field, converted)
            .map_err(|source| Error::Invalid(format!("Decode Sample Field {}: {source}", field.name())))?;
    }
    Ok(message)
}

fn scalar(kind: &Kind, value: &Value) -> HostResult<FieldValue> {
    let invalid = || Error::Invalid("Decode Sample: Value Does Not Match Its Declared Field".into());
    Ok(match kind {
        Kind::Bool => FieldValue::Bool(value.as_bool().ok_or_else(invalid)?),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => {
            FieldValue::I32(i32::try_from(value.as_i64().ok_or_else(invalid)?).map_err(|_| invalid())?)
        }
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => FieldValue::I64(value.as_i64().ok_or_else(invalid)?),
        Kind::Uint32 | Kind::Fixed32 => FieldValue::U32(u32::try_from(value.as_u64().ok_or_else(invalid)?).map_err(|_| invalid())?),
        Kind::Uint64 | Kind::Fixed64 => FieldValue::U64(value.as_u64().ok_or_else(invalid)?),
        Kind::Float => FieldValue::F32(value.as_f64().ok_or_else(invalid)? as f32),
        Kind::Double => FieldValue::F64(value.as_f64().ok_or_else(invalid)?),
        Kind::String => FieldValue::String(value.as_str().ok_or_else(invalid)?.into()),
        Kind::Bytes => FieldValue::Bytes(
            value
                .as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(|value| u8::try_from(value.as_u64().ok_or_else(invalid)?).map_err(|_| invalid()))
                .collect::<HostResult<Vec<_>>>()?
                .into(),
        ),
        Kind::Enum(_) => FieldValue::EnumNumber(i32::try_from(value.as_i64().ok_or_else(invalid)?).map_err(|_| invalid())?),
        Kind::Message(descriptor) => FieldValue::Message(message(descriptor.clone(), value)?),
    })
}

/* === Tests === */
