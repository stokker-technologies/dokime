/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod projection;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};
// Third Party Imports
use crate::grpc::transcode::Transcode;
use harmos::operations::{Context, Operation, Response};
use tonic::Status;
// Local Imports
use super::value::{self, canonical};
use dokime_domain::model::identity::IdentifierPattern;
use dokime_domain::model::identity::wire as common;
use dokime_domain::model::time::Timestamp;
use dokime_domain::operations::telemetry as wire;
use dokime_runtime::RuntimeHandle;
use dokime_runtime::domains::evidence::ScopeSelection;
use dokime_runtime::domains::evidence::TelemetrySelection;

/* === Constants === */
const DEFAULT_LIMIT: usize = 500;
const MAX_LIMIT: usize = 5_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct List {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for List {
    type Declaration = wire::ListSamples;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListSamples, _: &mut (),
    ) -> Result<Response<dokime_domain::model::telemetry::wire::TelemetrySamplesReply>, Status> {
        let limit = bounded_limit(request.limit)?;
        let after_stream_cursors = bounded_cursors(std::mem::take(&mut request.after_stream_cursors))?;
        if request.latest && !after_stream_cursors.is_empty() {
            return Err(Status::invalid_argument("latest telemetry windows cannot be continued with after_stream_cursors"));
        }
        if request.streams.len() > 64 {
            return Err(Status::invalid_argument("stream selection exceeds 64 alternatives"));
        }
        let selection = TelemetrySelection {
            after_stream_cursors: after_stream_cursors.clone(),
            limit: Some(limit),
            streams: std::mem::take(&mut request.streams).into_iter().map(stream).collect::<Result<_, _>>()?,
            schema_versions: bounded_schema_versions(std::mem::take(&mut request.schema_versions))?,
            sources: bounded_texts(std::mem::take(&mut request.sources), "telemetry source")?,
            signals: bounded_texts(std::mem::take(&mut request.signals), "telemetry signal")?,
            scope: scope_selection(request.scope.take())?,
            start: timestamp(request.start.take(), "telemetry start")?,
            end: timestamp(request.end.take(), "telemetry end")?,
            latest: request.latest,
        };
        let mut samples = self
            .resources
            .runtime
            .try_selected_telemetry(selection)
            .await
            .map_err(|error| Status::unavailable(format!("telemetry query unavailable: {error}")))?;
        let selected = samples.len();
        let (through_stream_cursors, _) = fit_page(&mut samples, request.latest, after_stream_cursors, self.resources.max_message_bytes)?;
        // A clamped page is never complete: forward pages continue from the
        // returned cursors, a latest window asks again with a smaller limit.
        let clamped = samples.len() < selected;
        let samples = samples.iter().map(projection::sample).collect::<Result<Vec<_>, _>>()?;
        let reply = dokime_domain::model::telemetry::wire::TelemetrySamplesReply {
            samples,
            through_stream_cursors: through_stream_cursors
                .into_iter()
                .map(|(key, value)| dokime_domain::model::telemetry::wire::TelemetrySamplesReplyThroughStreamCursorsEntry { key, value })
                .collect(),
            complete: !clamped && request_is_complete(request.latest, selected, limit),
        };
        if reply.wire_len()? > self.resources.max_message_bytes {
            return Err(Status::resource_exhausted("telemetry sample response exceeds transport limit"));
        }
        Ok(reply.into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), max_message_bytes: maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn bounded_limit(value: u32) -> Result<usize, Status> {
    let value = if value == 0 {
        DEFAULT_LIMIT
    } else {
        usize::try_from(value).map_err(|_| Status::out_of_range("telemetry limit is unsupported"))?
    };
    // Asking for more than one page holds is never refused; the page clamps.
    Ok(value.min(MAX_LIMIT))
}

fn fit_page(
    samples: &mut Vec<dokime_runtime::telemetry::TelemetrySample>, latest: bool, after_stream_cursors: BTreeMap<String, u64>, budget: usize,
) -> Result<(BTreeMap<String, u64>, usize), Status> {
    let cursor = |key: &str| {
        key.len().saturating_add(
            4 * std::mem::size_of::<dokime_domain::model::telemetry::wire::TelemetrySamplesReplyThroughStreamCursorsEntry>(),
        )
    };
    let mut through = if latest { BTreeMap::new() } else { after_stream_cursors };
    let mut required =
        through.keys().fold(std::mem::size_of::<dokime_domain::model::telemetry::wire::TelemetrySamplesReply>(), |bytes, key| {
            bytes.saturating_add(cursor(key))
        });
    let order: Box<dyn Iterator<Item = usize>> = if latest { Box::new((0..samples.len()).rev()) } else { Box::new(0..samples.len()) };
    let mut kept = 0;
    for index in order {
        let sample = &samples[index];
        let mut bytes =
            projection::memory(sample)?.saturating_add(3 * std::mem::size_of::<dokime_domain::model::telemetry::wire::TelemetrySample>());
        let stream = (!latest).then(|| dokime_runtime::domains::telemetry::telemetry_stream_id(sample.scope.as_ref(), &sample.stream));
        if let Some(stream) = &stream
            && !through.contains_key(stream)
        {
            bytes = bytes.saturating_add(cursor(stream));
        }
        if required.saturating_add(bytes) > budget {
            break;
        }
        required = required.saturating_add(bytes);
        if let Some(stream) = stream {
            through.entry(stream).and_modify(|through| *through = (*through).max(sample.sequence)).or_insert(sample.sequence);
        }
        kept += 1;
    }
    if kept == 0 && !samples.is_empty() {
        return Err(Status::resource_exhausted("one telemetry sample exceeds the message budget"));
    }
    if latest {
        samples.drain(..samples.len() - kept);
    } else {
        samples.truncate(kept);
    }
    Ok((through, required))
}

fn bounded_cursors(
    values: Vec<dokime_domain::model::telemetry::wire::ListTelemetrySamplesRequestAfterStreamCursorsEntry>,
) -> Result<std::collections::BTreeMap<String, u64>, Status> {
    value::collection(values.len())?;
    values
        .into_iter()
        .map(|entry| {
            value::text(&entry.key)?;
            Ok((entry.key, entry.value))
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

fn stream(value: String) -> Result<IdentifierPattern, Status> {
    value::text(&value)?;
    IdentifierPattern::new(value).map_err(|error| Status::invalid_argument(format!("invalid telemetry stream pattern: {error}")))
}

fn scope_selection(scope: Option<common::EvidenceScope>) -> Result<ScopeSelection, Status> {
    let Some(scope) = scope else {
        return Ok(ScopeSelection::default());
    };
    let scope = canonical::decoded_scope(&scope)?;
    if scope.session_id.is_some() || scope.step_id.is_some() || scope.criterion_id.is_some() || scope.suite_id.is_some() {
        return Err(Status::invalid_argument("telemetry scope supports cycle, case, execution, and template identifiers"));
    }
    Ok(ScopeSelection {
        test_cycle_id: scope.test_cycle_id,
        test_execution_id: scope.test_execution_id,
        test_case_id: scope.test_case_id,
        template_id: scope.template_id,
    })
}

fn timestamp(value: Option<common::Timestamp>, _: &str) -> Result<Option<Timestamp>, Status> {
    value.map(canonical::decoded_timestamp).transpose()
}

fn request_is_complete(latest: bool, sample_count: usize, limit: usize) -> bool {
    latest || sample_count < limit
}

/* === Tests === */
