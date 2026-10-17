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
use crate::telemetry::{TelemetryBatch, TelemetrySample};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use harmos::schema::Validator;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type Subscriptions =
    Vec<Pin<Box<dyn futures_util::Stream<Item = harmos::streams::Item<harmos::streams::PublicationRow>> + Send>>>;

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
        &self, owner: &str, token: &str, scope: RecordScope, streams: impl IntoIterator<Item = Identifier>,
    ) -> Result<Admission, String> {
        if scope.test_execution_id.as_ref().map(ToString::to_string).as_deref() != Some(token) {
            return Err("publication context must name its host execution".into());
        }
        let mut executions = self.0.lock().expect("execution publication admissions are healthy");
        if executions.contains_key(token) {
            return Err("execution publication context is already admitted".into());
        }
        executions.insert(token.to_owned(), Execution { owner: owner.to_owned(), scope, streams: streams.into_iter().collect() });
        Ok(Admission { executions: self.clone(), token: token.to_owned() })
    }

    pub(super) fn scope(&self, owner: &str, token: &str, stream: &Identifier) -> Result<RecordScope, String> {
        let executions = self.0.lock().expect("execution publication admissions are healthy");
        let execution = executions.get(token).ok_or("execution publication context is not active")?;
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
        self.executions.0.lock().expect("execution publication admissions are healthy").remove(&self.token);
    }
}

impl Decoder {
    pub(super) async fn ingest_batch(
        &mut self, handle: &RuntimeHandle, definition: &harmos::streams::Definition,
        rows: &[(harmos::storage::Cursor, &harmos::streams::PublicationRow)],
    ) {
        if rows.len() > 1
            && let Some(owner) = definition.owner()
            && let Ok(stream) = Identifier::new(definition.id())
        {
            let scope = match rows[0].1.context.as_deref() {
                Some(token) => handle.native_executions.scope(owner, token, &stream).map(Some),
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
                        let mut sample = TelemetrySample::new(crate::model::identity::new_id(), stream.clone(), cursor.0, now, owner);
                        sample.values = values;
                        sample.storage_payload = Some(row.payload.clone());
                        sample.scope = scope.clone();
                        Ok(sample)
                    })
                    .collect::<Result<Vec<_>, String>>();
                if let Ok(samples) = samples
                    && let Ok(receipt) = handle
                        .publish_telemetry_to_streams(
                            TelemetryBatch::new(crate::model::identity::new_id(), stream.clone(), samples),
                            crate::model::time::now_utc(),
                        )
                        .await
                {
                    if receipt.dropped > 0 {
                        refuse(
                            handle,
                            owner,
                            &stream,
                            scope.as_ref(),
                            receipt.message.as_deref().unwrap_or("native telemetry was dropped at admission"),
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
        &mut self, handle: &RuntimeHandle, definition: &harmos::streams::Definition, sequence: u64, row: &harmos::streams::PublicationRow,
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
        let result = self
            .decode(definition, &row.payload)
            .and_then(|values| publication_time(row.published_unix_micros).map(|timestamp| (values, timestamp)));
        match result {
            Ok((mut values, now)) => {
                values.insert("timestamp".into(), Value::from(now.clone()));
                let mut sample = TelemetrySample::new(crate::model::identity::new_id(), stream.clone(), sequence, now, owner);
                sample.values = values;
                sample.storage_payload = Some(row.payload.clone());
                sample.scope = scope.clone();
                // The shared capture reader cannot wait for one paused cycle.
                // Domain admission still filters paused/closed scopes; expose any
                // dropped row as a gap instead of holding unrelated streams behind it.
                match handle
                    .publish_telemetry_to_streams(
                        TelemetryBatch::new(crate::model::identity::new_id(), stream.clone(), vec![sample]),
                        crate::model::time::now_utc(),
                    )
                    .await
                {
                    Ok(receipt) if receipt.dropped > 0 => {
                        refuse(
                            handle,
                            owner,
                            &stream,
                            scope.as_ref(),
                            receipt.message.as_deref().unwrap_or("native telemetry was dropped at admission"),
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

    fn decode(&mut self, definition: &harmos::streams::Definition, bytes: &[u8]) -> Result<BTreeMap<String, Value>, String> {
        let key = (definition.id().to_owned(), definition.version());
        if !self.descriptors.contains_key(&key) {
            let schema = definition.schema().ok_or("native stream has no inspected schema")?;
            let descriptor =
                Validator::from_schema(schema, &harmos::config::SchemaSettings::default()).map_err(|error| error.to_string())?;
            self.descriptors.insert(key.clone(), descriptor);
        }
        crate::storage::telemetry::decode_stream_values(definition.id(), &self.descriptors[&key], bytes).map_err(|error| error.to_string())
    }
}

/* === Public Functions === */
pub(super) fn validate(artifact: &crate::InstalledPlugin, streams: &[harmos::sidecar::StreamDeclaration]) -> Result<(), String> {
    let contracts = artifact.telemetry_contracts.iter().chain(artifact.suites.suites.iter().flat_map(|suite| &suite.telemetry_contracts));
    for contract in contracts.filter(|contract| !contract.foreign) {
        let stream = streams.iter().find(|stream| stream.id == contract.stream.as_str()).ok_or_else(|| {
            format!(
                "telemetry `{}` is declared by `{}` but the plugin does not register it \
                     as a native stream: rebuild the plugin so `#[crate::plugin]` derives its \
                     streams from its suites' templates, its observers and its evidence",
                contract.stream, artifact.source
            )
        })?;
        let declared = contract
            .storage_schema
            .as_ref()
            .ok_or_else(|| format!("telemetry `{}` has no authoritative storage row schema", contract.stream))?;
        if !stream
            .row
            .equivalent(stream.row.root, declared, declared.root, &harmos::config::SchemaSettings::default())
            .map_err(|error| error.to_string())?
        {
            return Err(format!("telemetry `{}` storage graph differs from its native stream", contract.stream));
        }
        if stream.schema != contract.schema_version {
            return Err(format!("telemetry `{}` disagrees with its native stream schema version", contract.stream));
        }
        stream.row.prepare(&harmos::config::SchemaSettings::default()).map_err(|error| error.to_string())?;
        let fields = stream.row.root().map_err(|error| error.to_string())?.fields();
        if !contract.allow_extra_fields
            && fields.iter().any(|field| !contract.columns.iter().any(|column| column.metadata.identifier.as_str() == field.name))
        {
            return Err(format!("telemetry `{}` omits fields from its native stream", contract.stream));
        }
        for column in &contract.columns {
            let name = column.metadata.identifier.as_str();
            if name == "timestamp" {
                continue;
            }
            let field = fields
                .iter()
                .find(|field| field.name == name)
                .ok_or_else(|| format!("telemetry `{}` declares missing native field `{name}`", contract.stream))?;
            use crate::schema::{Cardinality, graph::Kind};
            use dokime_domain::model::values::ValueType;
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
                return Err(format!("telemetry `{}` disagrees with native field `{name}`", contract.stream));
            }
        }
    }
    Ok(())
}

pub(super) fn observe(handle: &RuntimeHandle, owner: &str) -> Subscriptions {
    let contracts = handle.installed_plugins().get(owner).map(|artifact| artifact.telemetry_contracts.clone()).unwrap_or_default();
    handle
        .streams()
        .definitions()
        .into_iter()
        .filter(|definition| {
            definition.owner() == Some(owner)
                && contracts
                    .iter()
                    .any(|contract| contract.stream.as_str() == definition.id() && contract.schema_version == definition.version())
        })
        .filter_map(|definition| {
            let stream = handle.streams().publication(definition.id(), definition.version())?;
            Some(Box::pin(stream.watch())
                as Pin<Box<dyn futures_util::Stream<Item = harmos::streams::Item<harmos::streams::PublicationRow>> + Send>>)
        })
        .collect()
}

pub(super) async fn incomplete(handle: &RuntimeHandle, owner: &str, stream: &Identifier, reason: &str) {
    if handle.streams_owned_by(owner).contains(stream) {
        refuse(handle, owner, stream, None, reason).await;
    }
    for scope in handle.native_executions.scopes(owner, stream) {
        refuse(handle, owner, stream, Some(&scope), reason).await;
    }
}

/* === Private Functions === */
async fn refuse(handle: &RuntimeHandle, owner: &str, stream: &Identifier, scope: Option<&RecordScope>, reason: &str) {
    if let Some(scope) = scope {
        handle.refuse_execution_stream(stream, scope, owner, reason).await;
    } else {
        handle.stream_leases().mark_gap(stream);
        handle.forward_lease_gap(stream, reason).await;
    }
}

fn publication_time(published_unix_micros: Option<i64>) -> Result<Timestamp, String> {
    let Some(micros) = published_unix_micros else {
        return Ok(crate::model::time::now_utc());
    };
    let instant = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(micros) * 1_000)
        .map_err(|error| format!("producer timestamp is outside the supported range: {error}"))?;
    let encoded = instant
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| format!("producer timestamp cannot be represented: {error}"))?;
    Ok(Timestamp::new(encoded))
}

/* === Tests === */
