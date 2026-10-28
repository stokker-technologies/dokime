/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::{BTreeMap, BTreeSet};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use super::RuntimeHandle;
use crate::domains::telemetry::Payloads;
use crate::telemetry::{TelemetryBatch, TelemetrySample};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

pub(super) type Subscriptions =
    Vec<Pin<Box<dyn futures_util::Stream<Item = harmos::streams::Item<harmos::streams::PublicationRow>> + Send>>>;

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

// A Run Of Rows Sharing One Declaration And Context Is Projected Once With The Saved Codec.
// Refused Rows Record A Gap; Admission Failure Never Replays An Already Decoded Run.
pub(super) async fn ingest_batch(
    handle: &RuntimeHandle, definition: &harmos::streams::Definition, rows: &[(harmos::storage::Cursor, &harmos::streams::PublicationRow)],
) {
    let Some(owner) = definition.owner() else {
        return;
    };
    let Ok(stream) = Identifier::new(definition.id()) else {
        return;
    };
    let Some((_, first)) = rows.first() else {
        return;
    };
    let scope = match publication_scope(handle, owner, &stream, first) {
        Ok(scope) => scope,
        Err(Some(error)) => {
            tracing::warn!(owner, %stream, %error, "native execution publication was refused");
            return;
        }
        Err(None) => return,
    };
    let codec = match saved_codec(handle, &stream) {
        Ok(codec) => codec,
        Err(error) => {
            tracing::warn!(owner, %stream, %error, "native telemetry could not be represented without loss");
            refuse(handle, owner, &stream, scope.as_ref(), &error).await;
            return;
        }
    };
    let mut samples = match crate::domains::telemetry::telemetry_slots(rows.len()) {
        Ok(samples) => samples,
        Err(error) => {
            refuse(handle, owner, &stream, scope.as_ref(), &error.to_string()).await;
            return;
        }
    };
    for (cursor, row) in rows {
        match decoded_sample(&codec, owner, &stream, scope.as_ref(), cursor.0, row) {
            Ok(sample) => samples.push(sample),
            Err(error) => {
                tracing::warn!(owner, %stream, %error, "native telemetry could not be represented without loss");
                refuse(handle, owner, &stream, scope.as_ref(), &error).await;
            }
        }
    }
    if samples.is_empty() {
        return;
    }
    // The shared capture reader cannot wait for one paused cycle.
    // Domain admission still filters paused/closed scopes; expose any
    // dropped row as a gap instead of holding unrelated streams behind it.
    let batch = TelemetryBatch::new(crate::model::identity::new_id(), stream.clone(), samples);
    match handle.publish_telemetry_to_streams(batch, crate::model::time::now_utc(), Payloads::Decoded(codec)).await {
        Ok(receipt) if receipt.dropped > 0 => {
            let reason = receipt.message.as_deref().unwrap_or("native telemetry was dropped at admission");
            refuse(handle, owner, &stream, scope.as_ref(), reason).await;
        }
        Ok(_) => {}
        Err(error) => {
            tracing::warn!(owner, %stream, ?error, "native telemetry was refused at domain admission");
            refuse(handle, owner, &stream, scope.as_ref(), &error.to_string()).await;
        }
    }
}

pub(super) fn validate(
    artifact: &crate::InstalledPlugin, streams: &[harmos::sidecar::StreamDeclaration], settings: &harmos::config::SchemaSettings,
) -> Result<(), String> {
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
        if !stream.row.equivalent(stream.row.root, declared, declared.root, settings).map_err(|error| error.to_string())? {
            return Err(format!("telemetry `{}` storage graph differs from its native stream", contract.stream));
        }
        if stream.schema != contract.schema_version {
            return Err(format!("telemetry `{}` disagrees with its native stream schema version", contract.stream));
        }
        stream.row.prepare(settings).map_err(|error| error.to_string())?;
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
    let state = handle.state_read();
    let Some(artifact) = state.installed_plugins.get(owner) else { return Vec::new() };
    let contracts = &artifact.telemetry_contracts;
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

// The Execution A Publication Context Names, None For The Owner's Own Stream; An Unknown Context Is
// Refused With Its Reason And An Unowned Stream Without Context Is Ignored.
fn publication_scope(
    handle: &RuntimeHandle, owner: &str, stream: &Identifier, row: &harmos::streams::PublicationRow,
) -> Result<Option<RecordScope>, Option<String>> {
    match row.context.as_deref() {
        Some(token) => handle.native_executions.scope(owner, token, stream).map(Some).map_err(Some),
        None if handle.streams_owned_by(owner).contains(stream) => Ok(None),
        None => Err(None),
    }
}

fn saved_codec(handle: &RuntimeHandle, stream: &Identifier) -> Result<Arc<crate::storage::telemetry::TypedCodec>, String> {
    let state = handle.state_read();
    let contract = state.telemetry_definitions.get(stream).ok_or("native stream has no registered telemetry contract")?;
    handle.execution_storage.registered_codec(contract).map_err(|error| error.to_string())
}

// The Row's One Payload Pass: Values Are Projected From The Retained Bytes, Then The Bytes Are Owned Once.
fn decoded_sample(
    codec: &crate::storage::telemetry::TypedCodec, owner: &str, stream: &Identifier, scope: Option<&RecordScope>, sequence: u64,
    row: &harmos::streams::PublicationRow,
) -> Result<TelemetrySample, String> {
    let mut values = codec.decode_payload(&row.payload).map_err(|error| error.to_string())?;
    let now = publication_time(row.published_unix_micros)?;
    values.insert("timestamp".into(), Value::from(now.clone()));
    let mut sample = TelemetrySample::new(crate::model::identity::new_id(), stream.clone(), sequence, now, owner);
    sample.values = values;
    sample.storage_payload = Some(row.payload.clone());
    sample.scope = scope.cloned();
    Ok(sample)
}

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

#[cfg(test)]
mod tests {
    use super::decoded_sample;
    use crate::storage::telemetry::{PAYLOAD_PASSES, codec};
    use crate::telemetry::TelemetryContract;
    use dokime_domain::model::values::Value;
    use harmos::Message;

    #[harmos::message]
    struct Reading {
        value: String,
    }

    #[test]
    fn native_rows_are_projected_in_one_pass() {
        let contract = TelemetryContract::declared("fixture.native", "Native").typed::<Reading>();
        let codec = codec(&contract, &harmos::config::SchemaSettings::default()).unwrap();
        let payload = Reading { value: "7".into() }.encode();
        let row = harmos::streams::PublicationRow {
            payload: payload.clone(),
            published_unix_micros: Some(1_000_000),
            context: None,
        };
        PAYLOAD_PASSES.with(std::cell::Cell::take);
        let sample = decoded_sample(&codec, "fixture", &contract.stream, None, 3, &row).unwrap();
        assert_eq!(PAYLOAD_PASSES.with(std::cell::Cell::take), 1);
        assert_eq!(sample.values.get("value"), Some(&Value::from("7")));
        assert_eq!(sample.timestamp.as_str(), "1970-01-01T00:00:01Z");
        assert_eq!(sample.storage_payload.as_deref(), Some(&payload[..]));
    }

    #[test]
    fn native_projection_is_reused_by_lease_staging() {
        use crate::domains::telemetry::{Payloads, share_scopes};
        use dokime_domain::model::{execution::RecordScope, identity::SessionId};
        use std::{collections::BTreeMap, sync::Arc};

        let contract = TelemetryContract::declared("fixture.native", "Native").typed::<Reading>();
        let codec = Arc::new(codec(&contract, &harmos::config::SchemaSettings::default()).unwrap());
        let payload = Reading { value: "7".repeat(8192) }.encode();
        let row = harmos::streams::PublicationRow {
            payload,
            published_unix_micros: Some(1_000_000),
            context: None,
        };
        let epoch = SessionId::from_seed("native.once");

        for count in [0, 8, 64] {
            let scopes = (0..count)
                .map(|_| RecordScope {
                    test_execution_id: Some(crate::model::identity::new_id()),
                    ..Default::default()
                })
                .collect();
            let leased = BTreeMap::from([(contract.stream.clone(), scopes)]);
            PAYLOAD_PASSES.with(std::cell::Cell::take);
            let sample = decoded_sample(&codec, "fixture", &contract.stream, None, 3, &row).unwrap();
            let shared = share_scopes(vec![sample], &leased, |scope| Ok(scope.clone())).unwrap();

            let codecs = [Arc::clone(&codec)];
            let payloads = Payloads::Decoded(Arc::clone(&codec));
            let staged = crate::runtime::storage::telemetry::stage_rows(&epoch, &shared, &codecs, &payloads).unwrap();
            assert_eq!(PAYLOAD_PASSES.with(std::cell::Cell::take), 1);
            assert_eq!(staged.len(), count + 1);
            assert!(shared.iter().all(|sample| Arc::ptr_eq(&sample.sample, &shared[0].sample)));
        }
    }
}
