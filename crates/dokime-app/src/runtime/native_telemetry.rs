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
use std::collections::{BTreeMap, BTreeSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
// Third Party Imports

// Local Imports
use super::RuntimeHandle;
use dokime::core::{Identifier, RecordScope, Timestamp, Value};
use dokime::telemetry::{TelemetryBatch, TelemetrySample};
use harmos::schema::{Event, PathElement, Scalar, Validator};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type Subscriptions =
    Vec<Pin<Box<dyn futures_util::Stream<Item = harmos::Item<harmos::PublicationRow>> + Send>>>;

/* === Structs === */
#[derive(Default)]
pub(super) struct Decoder {
    descriptors: BTreeMap<(String, u32), Validator>,
}

#[derive(Clone, Default)]
pub(crate) struct Executions(Arc<Mutex<BTreeMap<String, Execution>>>);

struct Execution {
    owner: String,
    scope: RecordScope,
    streams: BTreeSet<Identifier>,
}

pub(crate) struct Admission {
    executions: Executions,
    token: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Executions {
    pub(crate) fn admit(
        &self,
        owner: &str,
        token: &str,
        scope: RecordScope,
        streams: impl IntoIterator<Item = Identifier>,
    ) -> Result<Admission, String> {
        if scope
            .test_execution_id
            .as_ref()
            .map(ToString::to_string)
            .as_deref()
            != Some(token)
        {
            return Err("publication context must name its host execution".into());
        }
        let mut executions = self
            .0
            .lock()
            .expect("execution publication admissions are healthy");
        if executions.contains_key(token) {
            return Err("execution publication context is already admitted".into());
        }
        executions.insert(
            token.to_owned(),
            Execution {
                owner: owner.to_owned(),
                scope,
                streams: streams.into_iter().collect(),
            },
        );
        Ok(Admission {
            executions: self.clone(),
            token: token.to_owned(),
        })
    }

    pub(super) fn scope(
        &self,
        owner: &str,
        token: &str,
        stream: &Identifier,
    ) -> Result<RecordScope, String> {
        let executions = self
            .0
            .lock()
            .expect("execution publication admissions are healthy");
        let execution = executions
            .get(token)
            .ok_or("execution publication context is not active")?;
        if execution.owner != owner {
            return Err("publication context belongs to another plugin".into());
        }
        if !execution.streams.contains(stream) {
            return Err("stream is not declared by this execution's template".into());
        }
        Ok(execution.scope.clone())
    }

    fn scopes(&self, owner: &str, stream: &Identifier) -> Vec<RecordScope> {
        self.0
            .lock()
            .expect("execution publication admissions are healthy")
            .values()
            .filter(|execution| execution.owner == owner && execution.streams.contains(stream))
            .map(|execution| execution.scope.clone())
            .collect()
    }
}

impl Drop for Admission {
    fn drop(&mut self) {
        self.executions
            .0
            .lock()
            .expect("execution publication admissions are healthy")
            .remove(&self.token);
    }
}

impl Decoder {
    pub(super) async fn ingest_batch(
        &mut self,
        handle: &RuntimeHandle,
        definition: &harmos::Definition,
        rows: &[(harmos::Cursor, &harmos::PublicationRow)],
    ) {
        if rows.len() > 1
            && let Some(owner) = definition.owner()
            && let Ok(stream) = Identifier::new(definition.id())
        {
            let scope = match rows[0].1.context.as_deref() {
                Some(token) => handle
                    .native_executions
                    .scope(owner, token, &stream)
                    .map(Some),
                None if handle.streams_owned_by(owner).contains(&stream) => Ok(None),
                None => Err("stream owner is unavailable".into()),
            };
            if let Ok(scope) = scope {
                let samples = rows
                    .iter()
                    .map(|(cursor, row)| {
                        let mut values = self.decode(definition, &row.payload)?;
                        let now = publication_time(row.published_unix_micros)?;
                        values.insert("timestamp".into(), Value::from(now.clone()));
                        let mut sample = TelemetrySample::new(stream.clone(), cursor.0, now, owner);
                        sample.values = values;
                        sample.scope = scope.clone();
                        Ok(sample)
                    })
                    .collect::<Result<Vec<_>, String>>();
                if let Ok(samples) = samples
                    && let Ok(receipt) = handle
                        .publish_telemetry_to_streams(
                            TelemetryBatch::new(stream.clone(), samples),
                            Timestamp::now_utc(),
                        )
                        .await
                {
                    if receipt.dropped > 0 {
                        refuse(
                            handle,
                            owner,
                            &stream,
                            scope.as_ref(),
                            receipt
                                .message
                                .as_deref()
                                .unwrap_or("native telemetry was dropped at admission"),
                        )
                        .await;
                    }
                    return;
                }
            }
        }
        for (cursor, row) in rows {
            self.ingest(handle, definition, cursor.0, row).await;
        }
    }

    pub(super) async fn ingest(
        &mut self,
        handle: &RuntimeHandle,
        definition: &harmos::Definition,
        sequence: u64,
        row: &harmos::PublicationRow,
    ) {
        let Some(owner) = definition.owner() else {
            return;
        };
        let Ok(stream) = Identifier::new(definition.id()) else {
            return;
        };
        let scope = match row.context.as_deref() {
            Some(token) => match handle.native_executions.scope(owner, token, &stream) {
                Ok(scope) => Some(scope),
                Err(error) => {
                    tracing::warn!(owner, %stream, %error, "native execution publication was refused");
                    return;
                }
            },
            None if handle.streams_owned_by(owner).contains(&stream) => None,
            None => return,
        };
        let result = self.decode(definition, &row.payload).and_then(|values| {
            publication_time(row.published_unix_micros).map(|timestamp| (values, timestamp))
        });
        match result {
            Ok((mut values, now)) => {
                values.insert("timestamp".into(), Value::from(now.clone()));
                let mut sample = TelemetrySample::new(stream.clone(), sequence, now, owner);
                sample.values = values;
                sample.scope = scope.clone();
                // The shared capture reader cannot wait for one paused cycle.
                // Domain admission still filters paused/closed scopes; expose any
                // dropped row as a gap instead of holding unrelated streams behind it.
                match handle
                    .publish_telemetry_to_streams(
                        TelemetryBatch::new(stream.clone(), vec![sample]),
                        Timestamp::now_utc(),
                    )
                    .await
                {
                    Ok(receipt) if receipt.dropped > 0 => {
                        refuse(
                            handle,
                            owner,
                            &stream,
                            scope.as_ref(),
                            receipt
                                .message
                                .as_deref()
                                .unwrap_or("native telemetry was dropped at admission"),
                        )
                        .await;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(owner, %stream, ?error, "native telemetry was refused at domain admission");
                        refuse(handle, owner, &stream, scope.as_ref(), &error.to_string()).await;
                    }
                }
            }
            Err(error) => {
                tracing::warn!(owner, %stream, %error, "native telemetry could not be represented without loss");
                refuse(handle, owner, &stream, scope.as_ref(), &error).await;
            }
        }
    }

    fn decode(
        &mut self,
        definition: &harmos::Definition,
        bytes: &[u8],
    ) -> Result<BTreeMap<String, Value>, String> {
        let key = (definition.id().to_owned(), definition.version());
        if !self.descriptors.contains_key(&key) {
            let fields = definition
                .fields()
                .ok_or("native stream has no inspected fields")?;
            let descriptor = Validator::new(fields).map_err(|error| error.to_string())?;
            self.descriptors.insert(key.clone(), descriptor);
        }
        values(&self.descriptors[&key], bytes)
    }
}

/* === Public Functions === */
pub(super) fn validate(
    artifact: &crate::InstalledPlugin,
    streams: &[harmos::sidecar::StreamDeclaration],
) -> Result<(), String> {
    let contracts = artifact.telemetry_contracts.iter().chain(
        artifact
            .suites
            .suites
            .iter()
            .flat_map(|suite| &suite.telemetry_contracts),
    );
    for contract in contracts.filter(|contract| !contract.foreign) {
        let stream = streams
            .iter()
            .find(|stream| stream.id == contract.stream.as_str())
            .ok_or_else(|| {
                format!(
                    "telemetry `{}` has no native stream owned by `{}`",
                    contract.stream, artifact.source
                )
            })?;
        if stream.schema != contract.schema_version {
            return Err(format!(
                "telemetry `{}` disagrees with its native stream schema version",
                contract.stream
            ));
        }
        if !contract.allow_extra_fields
            && stream.fields.iter().any(|field| {
                !contract
                    .columns
                    .iter()
                    .any(|column| column.metadata.identifier.as_str() == field.name)
            })
        {
            return Err(format!(
                "telemetry `{}` omits fields from its native stream",
                contract.stream
            ));
        }
        for column in &contract.columns {
            let name = column.metadata.identifier.as_str();
            if name == "timestamp" {
                continue;
            }
            let field = stream
                .fields
                .iter()
                .find(|field| field.name == name)
                .ok_or_else(|| {
                    format!(
                        "telemetry `{}` declares missing native field `{name}`",
                        contract.stream
                    )
                })?;
            use dokime::core::ValueType;
            use dokime::schema::{Cardinality, Kind};
            let expected = if field.cardinality == Cardinality::Repeated {
                ValueType::Array
            } else {
                match field.kind {
                    Kind::Bool => ValueType::Boolean,
                    Kind::I32 | Kind::I64 | Kind::U32 | Kind::U64 => ValueType::Integer,
                    Kind::F32 | Kind::F64 => ValueType::Number,
                    Kind::String => ValueType::Text,
                    Kind::Bytes => ValueType::Array,
                    Kind::Message { .. } => ValueType::Object,
                }
            };
            if column.value_type != expected {
                return Err(format!(
                    "telemetry `{}` disagrees with native field `{name}`",
                    contract.stream
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn observe(handle: &RuntimeHandle, owner: &str) -> Subscriptions {
    let contracts = handle
        .installed_plugins()
        .get(owner)
        .map(|artifact| artifact.telemetry_contracts.clone())
        .unwrap_or_default();
    handle
        .streams()
        .definitions()
        .into_iter()
        .filter(|definition| {
            definition.owner() == Some(owner)
                && contracts.iter().any(|contract| {
                    contract.stream.as_str() == definition.id()
                        && contract.schema_version == definition.version()
                })
        })
        .filter_map(|definition| {
            let stream = handle
                .streams()
                .publication(definition.id(), definition.version())?;
            Some(Box::pin(stream.watch())
                as Pin<
                    Box<
                        dyn futures_util::Stream<Item = harmos::Item<harmos::PublicationRow>>
                            + Send,
                    >,
                >)
        })
        .collect()
}

pub(super) async fn incomplete(
    handle: &RuntimeHandle,
    owner: &str,
    stream: &Identifier,
    reason: &str,
) {
    if handle.streams_owned_by(owner).contains(stream) {
        refuse(handle, owner, stream, None, reason).await;
    }
    for scope in handle.native_executions.scopes(owner, stream) {
        refuse(handle, owner, stream, Some(&scope), reason).await;
    }
}

/* === Private Functions === */
async fn refuse(
    handle: &RuntimeHandle,
    owner: &str,
    stream: &Identifier,
    scope: Option<&RecordScope>,
    reason: &str,
) {
    if let Some(scope) = scope {
        handle
            .refuse_execution_stream(stream, scope, owner, reason)
            .await;
    } else {
        handle.stream_leases().mark_gap(stream);
        handle.forward_lease_gap(stream, reason).await;
    }
}

fn publication_time(published_unix_micros: Option<i64>) -> Result<Timestamp, String> {
    let Some(micros) = published_unix_micros else {
        return Ok(Timestamp::now_utc());
    };
    let instant = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000)
        .map_err(|error| format!("producer timestamp is outside the supported range: {error}"))?;
    let encoded = instant
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| format!("producer timestamp cannot be represented: {error}"))?;
    Ok(Timestamp::new(encoded))
}

fn values(validator: &Validator, bytes: &[u8]) -> Result<BTreeMap<String, Value>, String> {
    let mut containers = Vec::new();
    let mut root = None;
    validator
        .visit(bytes, |path, event| {
            match event {
                Event::BeginMessage => containers.push(Value::Object(BTreeMap::new())),
                Event::BeginRepeated { .. } => containers.push(Value::Array(Vec::new())),
                Event::EndMessage | Event::EndRepeated => {
                    let value = containers.pop().ok_or("missing telemetry container")?;
                    if containers.is_empty() {
                        root = Some(value);
                    } else {
                        attach(&mut containers, path, value)?;
                    }
                }
                Event::Optional { present: false } => {
                    attach(&mut containers, path, Value::Null)?;
                }
                Event::Scalar(scalar) => attach(&mut containers, path, value(scalar))?,
                Event::Optional { present: true } | Event::Variant { .. } => {}
            }
            Ok::<(), String>(())
        })
        .map_err(|error| error.to_string())?;
    match root {
        Some(Value::Object(values)) if containers.is_empty() => Ok(values),
        _ => Err("telemetry root is not a complete message".into()),
    }
}

fn attach(containers: &mut [Value], path: &[PathElement<'_>], value: Value) -> Result<(), String> {
    match (containers.last_mut(), path.last()) {
        (Some(Value::Object(fields)), Some(PathElement::Field { name, .. })) => {
            fields.insert((*name).to_owned(), value);
        }
        (Some(Value::Array(items)), Some(PathElement::Index(index))) if *index == items.len() => {
            items.push(value);
        }
        _ => return Err("telemetry value does not match its container".into()),
    }
    Ok(())
}

fn value(value: Scalar<'_>) -> Value {
    match value {
        Scalar::Bool(value) => Value::Bool(value),
        Scalar::I32(value) => Value::Integer(i64::from(value)),
        Scalar::I64(value) => Value::Integer(value),
        Scalar::U32(value) => Value::Unsigned(u64::from(value)),
        Scalar::U64(value) => Value::Unsigned(value),
        Scalar::F32(value) => Value::Number(f64::from(value)),
        Scalar::F64(value) => Value::Number(value),
        Scalar::String(value) => Value::Text(value.to_owned()),
        Scalar::Bytes(value) => Value::Array(
            value
                .iter()
                .map(|byte| Value::Integer(i64::from(*byte)))
                .collect(),
        ),
    }
}

/* === Tests === */
