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
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::{
    Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::value::{self, canonical};
use dokime_app::{
    RuntimeHandle,
    capabilities::evidence::{ScopeSelection, TelemetrySelection},
    core::{Identifier, Timestamp},
};
use dokime_contract::operations::{common, telemetry as wire};

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
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListTelemetrySamplesRequest,
        _: &mut (),
    ) -> Result<Response<wire::TelemetrySamplesReply>, Status> {
        let limit = bounded_limit(request.limit)?;
        let after_stream_cursors =
            bounded_cursors(std::mem::take(&mut request.after_stream_cursors))?;
        if request.latest && !after_stream_cursors.is_empty() {
            return Err(Status::invalid_argument(
                "latest telemetry windows cannot be continued with after_stream_cursors",
            ));
        }
        let selection = TelemetrySelection {
            after_stream_cursors: after_stream_cursors.clone(),
            limit: Some(limit),
            streams: std::mem::take(&mut request.streams)
                .into_iter()
                .map(stream)
                .collect::<Result<_, _>>()?,
            schema_versions: bounded_schema_versions(std::mem::take(&mut request.schema_versions))?,
            sources: bounded_texts(std::mem::take(&mut request.sources), "telemetry source")?,
            signals: bounded_texts(std::mem::take(&mut request.signals), "telemetry signal")?,
            scope: scope_selection(request.scope.take())?,
            start: timestamp(request.start.take(), "telemetry start")?,
            end: timestamp(request.end.take(), "telemetry end")?,
            latest: request.latest,
        };
        let samples = self
            .resources
            .runtime
            .try_selected_telemetry(selection)
            .await
            .map_err(|error| {
                Status::unavailable(format!("telemetry query unavailable: {error}"))
            })?;
        let sample_count = samples.len();
        let mut through_stream_cursors: BTreeMap<_, _> = if request.latest {
            BTreeMap::new()
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
        let required = samples
            .iter()
            .try_fold(
                std::mem::size_of::<wire::TelemetrySamplesReply>(),
                |bytes, item| {
                    Ok::<_, Status>(
                        bytes
                            .saturating_add(projection::memory(item)?)
                            .saturating_add(3 * std::mem::size_of::<wire::TelemetrySample>()),
                    )
                },
            )?
            .saturating_add(
                through_stream_cursors
                    .iter()
                    .fold(0usize, |bytes, (key, _)| {
                        bytes.saturating_add(key.len()).saturating_add(
                            4 * std::mem::size_of::<
                                wire::TelemetrySamplesReplyThroughStreamCursorsEntry,
                            >(),
                        )
                    }),
            );
        if required > self.resources.max_message_bytes {
            return Err(Status::resource_exhausted(
                "telemetry response exceeds original budget",
            ));
        }
        let preparation = self
            .resources
            .runtime
            .storage()
            .prepare_production(
                NonZeroUsize::new(required)
                    .ok_or_else(|| Status::resource_exhausted("invalid telemetry allowance"))?,
            )
            .map_err(failure)?;
        let samples = samples
            .iter()
            .map(projection::sample)
            .collect::<Result<Vec<_>, _>>()?;
        let reply = wire::TelemetrySamplesReply {
            samples,
            through_stream_cursors: through_stream_cursors
                .into_iter()
                .map(
                    |(key, value)| wire::TelemetrySamplesReplyThroughStreamCursorsEntry {
                        key,
                        value,
                    },
                )
                .collect(),
            complete: request_is_complete(request.latest, sample_count, limit),
        };
        if reply.encoded_len() > self.resources.max_message_bytes {
            return Err(Status::resource_exhausted(
                "telemetry sample response exceeds transport limit",
            ));
        }
        preparation
            .finish(reply)
            .map_err(failure)?
            .try_into_value()
            .map_err(failure)?
            .try_into_response()
            .map_err(failure)
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            max_message_bytes: maximum,
        }),
    )?;
    Ok(())
}

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
    values: Vec<wire::ListTelemetrySamplesRequestAfterStreamCursorsEntry>,
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

fn stream(value: String) -> Result<Identifier, Status> {
    value::text(&value)?;
    Identifier::new(value)
        .map_err(|error| Status::invalid_argument(format!("invalid telemetry stream: {error}")))
}

fn scope_selection(scope: Option<common::EvidenceScope>) -> Result<ScopeSelection, Status> {
    let Some(scope) = scope else {
        return Ok(ScopeSelection::default());
    };
    let scope = canonical::decoded_scope(&scope)?;
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

fn timestamp(value: Option<common::Timestamp>, _: &str) -> Result<Option<Timestamp>, Status> {
    value.map(canonical::decoded_timestamp).transpose()
}
fn failure(error: harmos::Error) -> Status {
    let status = if matches!(error, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("telemetry admission failed")
    } else {
        Status::internal("telemetry admission failed")
    };
    super::error::detailed(status, &error.into())
}

fn request_is_complete(latest: bool, sample_count: usize, limit: usize) -> bool {
    latest || sample_count < limit
}

/* === Tests === */
