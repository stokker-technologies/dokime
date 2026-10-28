/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(super) mod projection;

use crate::grpc::transcode::Transcode;

use std::{collections::BTreeMap, sync::Arc};

use dokime::harmos::{Context, RegistrationExt, Response};
use tonic::Status;

use super::value::{self, canonical};

use dokime::RuntimeHandle;
use dokime::domain::model::identity::IdentifierPattern;
use dokime::domain::model::identity::wire as common;
use dokime::domain::model::time::Timestamp;
use dokime::domains::evidence::ScopeSelection;
use dokime::domains::evidence::TelemetrySelection;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::telemetry as wire;

const SERVER_DEFAULT_TELEMETRY_ITEMS: u64 = 500;
const SERVER_EVIDENCE_SELECTION_ITEMS: u64 = 64;
const SERVER_MAXIMUM_TELEMETRY_ITEMS: u64 = 5_000;

struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct Selection {
    query: TelemetrySelection,
}

enum ProjectionError {
    Status(Status),
    Storage(dokime::harmos::storage::Error),
}

impl From<Status> for ProjectionError {
    fn from(error: Status) -> Self {
        Self::Status(error)
    }
}

impl From<dokime::harmos::storage::Error> for ProjectionError {
    fn from(error: dokime::harmos::storage::Error) -> Self {
        Self::Storage(error)
    }
}

impl ProjectionError {
    fn status(self) -> Status {
        match self {
            Self::Status(error) => error,
            Self::Storage(error @ dokime::harmos::storage::Error::ResourceExhausted { .. }) => {
                Status::resource_exhausted(error.to_string())
            }
            Self::Storage(error) => Status::unavailable(error.to_string()),
        }
    }
}

impl Serve<wire::ListSamples> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::ListSamples, _: Context<'a, wire::ListSamples>,
    ) -> Served<'a, dokime::domain::model::telemetry::wire::TelemetrySamplesReply> {
        Box::pin(async move {
            let session = self.runtime.storage().clone();
            let selection = self.selection(&mut request)?;
            let at = session.cursor_saved();
            self.page(&session, &selection, &at, 0, false).await.map(|(page, ..)| page)
        })
    }
}

impl Serve<wire::WatchSamples> for Resources {
    fn serve<'a>(&'a self, mut request: wire::WatchSamples, ctx: Context<'a, wire::WatchSamples>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let mut selection = request.selection.take().ok_or_else(|| Status::invalid_argument("telemetry selection is missing"))?;
                if selection.latest || request.follow && selection.scope.as_ref().and_then(|scope| scope.execution_id.as_ref()).is_none() {
                    return Err(Status::invalid_argument("telemetry watch requires forward paging and an execution scope when following"));
                }
                let runtime = &self.runtime;
                let session = runtime.storage().clone();
                let mut selection = self.selection(&mut selection)?;
                let mut at = session.cursor_saved();
                // Every Selected Row At Or Below `after` Was Delivered, So Each Page And Wake Scans Only Newer Rows.
                let mut after = 0;
                loop {
                    let (page, sealed, settled) = self.page(&session, &selection, &at, after, true).await?;
                    let complete = page.complete;
                    let through = page.through_stream_cursors.clone();
                    output.send_owned(page).await.map_err(|error| Status::unavailable(error.to_string()))?;
                    selection.advance_cursors(&through);
                    after = settled;
                    if !complete {
                        continue;
                    }
                    if !request.follow || sealed {
                        return Ok(().into());
                    }
                    // Native Wait Registers Before Inspecting The Saved Frontier; Cancellation Only Drops The Waiter.
                    at = loop {
                        match tokio::time::timeout(super::WATCH_HEARTBEAT, session.wait_saved_after(&at)).await {
                            Ok(saved) => break saved.map_err(storage_failure)?,
                            Err(_) => {
                                output.send_owned(heartbeat().into()).await.map_err(|error| Status::unavailable(error.to_string()))?
                            }
                        }
                    };
                }
            })
            .await
        })
    }
}

impl Resources {
    fn selection(&self, request: &mut wire::ListSamples) -> Result<Selection, Status> {
        let limit = bounded_limit(request.limit)?;
        if request.latest && !request.after_stream_cursors.is_empty() {
            return Err(Status::invalid_argument("latest telemetry windows cannot be continued with after_stream_cursors"));
        }
        if request.streams.len() as u64 > SERVER_EVIDENCE_SELECTION_ITEMS {
            return Err(Status::invalid_argument("stream selection exceeds the configured alternatives limit"));
        }
        value::collection(request.after_stream_cursors.len())?;
        let after_stream_cursors =
            std::mem::take(&mut request.after_stream_cursors).into_iter().map(|entry| (entry.key, entry.value)).collect();
        let mut streams = Vec::new();
        streams.try_reserve_exact(request.streams.len()).map_err(|_| selection_memory())?;

        for value in std::mem::take(&mut request.streams) {
            streams.push(stream(value)?);
        }
        let query = TelemetrySelection {
            after_stream_cursors,
            limit: Some(limit),
            streams,
            schema_versions: bounded_schema_versions(std::mem::take(&mut request.schema_versions))?,
            sources: bounded_texts(std::mem::take(&mut request.sources), "telemetry source")?,
            signals: bounded_texts(std::mem::take(&mut request.signals), "telemetry signal")?,
            scope: scope_selection(request.scope.take())?,
            start: timestamp(request.start.take(), "telemetry start")?,
            end: timestamp(request.end.take(), "telemetry end")?,
            latest: request.latest,
        };
        Ok(Selection { query })
    }

    // Answers One Page And The Storage Position Through Which The Selection Is Now Settled.
    async fn page(
        &self, session: &dokime::storage::Manager, selection: &Selection, at: &dokime::harmos::storage::StorageCursor, after: u64,
        streamed: bool,
    ) -> Result<(Response<dokime::domain::model::telemetry::wire::TelemetrySamplesReply>, bool, u64), Status> {
        let query = &selection.query;
        let after_stream_cursors = &query.after_stream_cursors;
        let limit = query.limit.expect("selection limit is bounded during admission");
        let (page, sealed) = self
            .runtime
            .telemetry_page_at(session, query, at, after)
            .await
            .map_err(|error| Status::unavailable(format!("telemetry query unavailable: {error}")))?;
        let mut delivered = 0;
        let reply = page
            .samples
            .try_map(|mut samples| -> Result<_, ProjectionError> {
                let selected = samples.len();
                let (through, _) = fit_page(&mut samples, query.latest, after_stream_cursors, self.max_message_bytes, streamed, limit)?;
                delivered = samples.len();
                let clamped = samples.len() < selected;
                let mut projected = Vec::new();
                projected
                    .try_reserve_exact(samples.len())
                    .map_err(|_| Status::resource_exhausted("telemetry response allocation refused"))?;
                for sample in &samples {
                    projected.push(projection::sample(sample)?);
                }
                let mut cursors = Vec::new();
                cursors.try_reserve_exact(through.len()).map_err(|_| Status::resource_exhausted("telemetry cursor allocation refused"))?;
                for (key, value) in through {
                    cursors.push(dokime::domain::model::telemetry::wire::TelemetrySamplesReplyThroughStreamCursorsEntry { key, value });
                }
                Ok(dokime::domain::model::telemetry::wire::TelemetrySamplesReply {
                    samples: projected,
                    through_stream_cursors: cursors,
                    complete: !clamped && request_is_complete(query.latest, selected, limit),
                })
            })
            .map_err(ProjectionError::status)?;
        // A Whole Page Settles The Frontier; A Clamped One Settles Its Last Delivered Storage Position.
        let settled = match delivered.checked_sub(1) {
            _ if delivered == page.positions.len() && !page.limited => at.sequence,
            Some(last) => page.positions[last],
            None => after,
        };
        Ok((Response::from_owner(reply.owner()), sealed, settled))
    }
}

impl Selection {
    fn advance_cursors(&mut self, cursors: &[dokime::domain::model::telemetry::wire::TelemetrySamplesReplyThroughStreamCursorsEntry]) {
        for cursor in cursors {
            match self.query.after_stream_cursors.get_mut(&cursor.key) {
                Some(value) => *value = cursor.value,
                None => {
                    self.query.after_stream_cursors.insert(cursor.key.clone(), cursor.value);
                }
            }
        }
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes: maximum,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::ListSamples>::new(served.clone()))?;
    dependencies.insert(Host::<wire::WatchSamples>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<wire::ListSamples>().operation::<wire::WatchSamples>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

/// An Empty Complete Page Keeps Every Cursor And Proves The Followed Stream Is Alive.
fn heartbeat() -> dokime::domain::model::telemetry::wire::TelemetrySamplesReply {
    dokime::domain::model::telemetry::wire::TelemetrySamplesReply {
        samples: Vec::new(),
        through_stream_cursors: Vec::new(),
        complete: true,
    }
}

fn selection_memory() -> Status {
    Status::resource_exhausted("telemetry selection allocation refused")
}

fn storage_failure(error: dokime::harmos::storage::Error) -> Status {
    ProjectionError::Storage(error).status()
}

fn bounded_limit(value: u32) -> Result<usize, Status> {
    let value = if value == 0 {
        usize::try_from(SERVER_DEFAULT_TELEMETRY_ITEMS).expect("validated telemetry default count")
    } else {
        usize::try_from(value).map_err(|_| Status::out_of_range("telemetry limit is unsupported"))?
    };
    // Asking for more than one page holds is never refused; the page clamps.
    Ok(value.min(usize::try_from(SERVER_MAXIMUM_TELEMETRY_ITEMS).expect("validated telemetry maximum count")))
}

fn fit_page(
    samples: &mut Vec<dokime::telemetry::TelemetrySample>, latest: bool, after: &BTreeMap<String, u64>, maximum: usize, streamed: bool,
    limit: usize,
) -> Result<(BTreeMap<String, u64>, usize), Status> {
    let selected = samples.len();
    let project = |count: usize| -> Result<_, Status> {
        let mut through = if latest { BTreeMap::new() } else { after.clone() };
        let start = if latest { selected - count } else { 0 };
        let mut projected = Vec::new();
        projected.try_reserve_exact(count).map_err(|_| selection_memory())?;
        for sample in &samples[start..start + count] {
            projected.push(projection::sample(sample)?);
            if !latest {
                let stream = dokime::domains::telemetry::telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
                through.entry(stream).and_modify(|value| *value = (*value).max(sample.sequence)).or_insert(sample.sequence);
            }
        }
        let cursors = through
            .iter()
            .map(|(key, value)| dokime::domain::model::telemetry::wire::TelemetrySamplesReplyThroughStreamCursorsEntry {
                key: key.clone(),
                value: *value,
            })
            .collect();
        let reply = dokime::domain::model::telemetry::wire::TelemetrySamplesReply {
            samples: projected,
            through_stream_cursors: cursors,
            complete: count == selected && request_is_complete(latest, selected, limit),
        };
        let bytes = if streamed { super::content::output_frame_len(&reply)? } else { reply.wire_len()? };
        Ok((through, bytes))
    };
    let (through, bytes) = project(selected)?;
    if bytes <= maximum {
        return Ok((through, bytes));
    }
    let mut low = 0;
    let mut high = selected;
    while low < high {
        let middle = low + (high - low).div_ceil(2);
        if project(middle)?.1 <= maximum {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    let (through, bytes) = project(low)?;
    if bytes > maximum || low == 0 && selected != 0 {
        return Err(Status::resource_exhausted("telemetry sample or cursor exceeds transport limit"));
    }
    if latest {
        samples.drain(..selected - low);
    } else {
        samples.truncate(low);
    }
    Ok((through, bytes))
}

fn bounded_schema_versions(values: Vec<u32>) -> Result<Vec<u32>, Status> {
    value::collection(values.len())?;
    Ok(values)
}

fn bounded_texts(values: Vec<String>, label: &str) -> Result<Vec<String>, Status> {
    value::collection(values.len())?;
    if values.iter().any(String::is_empty) {
        return Err(Status::invalid_argument(format!("{label} is empty")));
    }
    Ok(values)
}

fn stream(value: String) -> Result<IdentifierPattern, Status> {
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
