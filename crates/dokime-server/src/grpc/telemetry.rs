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
use std::collections::HashMap;
// Third Party Imports
use prost::Message;
use tonic::{Request, Response, Status};
// Local Imports
use super::{plots, value};
use dokime_app::{
    RuntimeHandle,
    capabilities::evidence::{ScopeSelection, TelemetrySelection},
    core::{Identifier, Timestamp},
    telemetry::TelemetrySample,
};
use dokime_protocol::v1::{self, telemetry_server::Telemetry};

/* === Constants === */
const DEFAULT_LIMIT: usize = 500;
const MAX_LIMIT: usize = 5_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Telemetry for Service {
    async fn list_samples(
        &self,
        request: Request<v1::ListTelemetrySamplesRequest>,
    ) -> Result<Response<v1::TelemetrySamplesReply>, Status> {
        let request = request.into_inner();
        let limit = bounded_limit(request.limit)?;
        let after_stream_cursors = bounded_cursors(request.after_stream_cursors)?;
        if request.latest && !after_stream_cursors.is_empty() {
            return Err(Status::invalid_argument(
                "latest telemetry windows cannot be continued with after_stream_cursors",
            ));
        }
        let selection = TelemetrySelection {
            after_stream_cursors: after_stream_cursors.clone(),
            limit: Some(limit),
            streams: request
                .streams
                .into_iter()
                .map(stream)
                .collect::<Result<_, _>>()?,
            schema_versions: bounded_schema_versions(request.schema_versions)?,
            sources: bounded_texts(request.sources, "telemetry source")?,
            signals: bounded_texts(request.signals, "telemetry signal")?,
            scope: scope_selection(request.scope)?,
            start: timestamp(request.start, "telemetry start")?,
            end: timestamp(request.end, "telemetry end")?,
            latest: request.latest,
        };
        let samples = self
            .runtime
            .try_selected_telemetry(selection)
            .await
            .map_err(|error| {
                Status::unavailable(format!("telemetry query unavailable: {error}"))
            })?;
        let sample_count = samples.len();
        let mut through_stream_cursors: HashMap<_, _> = if request.latest {
            HashMap::new()
        } else {
            after_stream_cursors.into_iter().collect()
        };
        if !request.latest {
            for sample in &samples {
                let stream = dokime_app::capabilities::telemetry::telemetry_stream_id(
                    sample.scope.as_ref(),
                    &sample.stream,
                );
                through_stream_cursors
                    .entry(stream)
                    .and_modify(|cursor| *cursor = (*cursor).max(sample.sequence))
                    .or_insert(sample.sequence);
            }
        }
        let samples = samples
            .into_iter()
            .map(sample)
            .collect::<Result<Vec<_>, _>>()?;
        let reply = v1::TelemetrySamplesReply {
            samples,
            through_stream_cursors,
            complete: request_is_complete(request.latest, sample_count, limit),
        };
        if reply.encoded_len() > self.max_message_bytes {
            return Err(Status::resource_exhausted(
                "telemetry sample response exceeds transport limit",
            ));
        }
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn bounded_limit(value: u32) -> Result<usize, Status> {
    let value = if value == 0 {
        DEFAULT_LIMIT
    } else {
        usize::try_from(value)
            .map_err(|_| Status::out_of_range("telemetry limit is unsupported"))?
    };
    if value > MAX_LIMIT {
        return Err(Status::resource_exhausted(
            "telemetry limit exceeds 5000 samples",
        ));
    }
    Ok(value)
}

fn bounded_cursors(
    values: std::collections::HashMap<String, u64>,
) -> Result<std::collections::BTreeMap<String, u64>, Status> {
    value::collection(values.len())?;
    values
        .into_iter()
        .map(|(key, value)| {
            value::text(&key)?;
            Ok((key, value))
        })
        .collect()
}

fn bounded_schema_versions(values: Vec<u32>) -> Result<Vec<u32>, Status> {
    value::collection(values.len())?;
    Ok(values)
}

fn bounded_texts(values: Vec<String>, label: &str) -> Result<Vec<String>, Status> {
    value::collection(values.len())?;
    values
        .into_iter()
        .map(|value| {
            value::text(&value)?;
            if value.is_empty() {
                return Err(Status::invalid_argument(format!("{label} is empty")));
            }
            Ok(value)
        })
        .collect()
}

fn stream(value: String) -> Result<Identifier, Status> {
    value::text(&value)?;
    Identifier::new(value)
        .map_err(|error| Status::invalid_argument(format!("invalid telemetry stream: {error}")))
}

fn scope_selection(scope: Option<v1::EvidenceScope>) -> Result<ScopeSelection, Status> {
    let Some(scope) = scope else {
        return Ok(ScopeSelection::default());
    };
    let scope = plots::record_scope(scope)?;
    if scope.session_id.is_some()
        || scope.step_id.is_some()
        || scope.criterion_id.is_some()
        || scope.suite_id.is_some()
    {
        return Err(Status::invalid_argument(
            "telemetry scope supports cycle, case, execution, and template identifiers",
        ));
    }
    Ok(ScopeSelection {
        test_cycle_id: scope.test_cycle_id,
        test_execution_id: scope.test_execution_id,
        test_case_id: scope.test_case_id,
        template_id: scope.template_id,
    })
}

fn timestamp(
    value: Option<prost_types::Timestamp>,
    label: &str,
) -> Result<Option<Timestamp>, Status> {
    value
        .map(|value| {
            if !(0..1_000_000_000).contains(&value.nanos) {
                return Err(Status::invalid_argument(format!(
                    "{label} has invalid nanoseconds",
                )));
            }
            let time = time::OffsetDateTime::from_unix_timestamp(value.seconds)
                .map_err(|_| Status::invalid_argument(format!("{label} is out of range")))?
                .replace_nanosecond(value.nanos as u32)
                .map_err(|_| Status::invalid_argument(format!("{label} is out of range")))?;
            let text = time
                .format(&time::format_description::well_known::Rfc3339)
                .map_err(|_| Status::invalid_argument(format!("{label} is invalid")))?;
            Ok(Timestamp::new(text))
        })
        .transpose()
}

fn sample(value: TelemetrySample) -> Result<v1::TelemetrySample, Status> {
    Ok(v1::TelemetrySample {
        sample_id: value.sample_id.to_string(),
        stream: value.stream.to_string(),
        schema_version: value.schema_version,
        sequence: value.sequence,
        timestamp: Some(value::timestamp(&value.timestamp)?),
        received_at: Some(value::timestamp(&value.received_at)?),
        source: value.source,
        scope: value.scope.as_ref().map(value::scope),
        values: Some(plots::values(&value.values)?),
        summary: Some(plots::values(&value.summary)?),
        metadata: Some(plots::values(&value.metadata)?),
    })
}

fn request_is_complete(latest: bool, sample_count: usize, limit: usize) -> bool {
    latest || sample_count < limit
}

/* === Tests === */
