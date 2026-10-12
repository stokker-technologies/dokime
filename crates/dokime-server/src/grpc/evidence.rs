/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod codec;
mod selection;
mod wire;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dokime.v1.Evidence.rs"));
}

/* === Re-Exports === */
pub(super) use generated::evidence_server::EvidenceServer;
pub(crate) use wire::{
    allocation_memory as historical_record_memory, copy_memory as historical_copy_memory,
    object_allocation as historical_object_memory, record as historical_record,
    scope_allocation as historical_scope_memory,
};

/* === Imports === */
// Standard Library Imports
use std::pin::Pin;
// Third Party Imports
use futures_util::{Stream, stream};
use prost::Message;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
// Local Imports
use super::error::detailed;
use codec::{Page, WirePage};
use dokime_app::{RuntimeHandle, runtime::CapturedRecords};
use dokime_protocol::v1;
use generated::evidence_server::Evidence;

use selection::Selection;

/* === Constants === */
const SCAN_LIMIT: usize = 1000;
const RECORD_LIMIT: usize = 100;
const PAGE_BYTES: usize = 512 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) stopping: watch::Receiver<bool>,
    pub(super) max_message_bytes: usize,
}

struct Reader {
    runtime: RuntimeHandle,
    source: CapturedRecords,
    selection: Selection,
    scanned: u64,
    through: u64,
    bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Reader {
    fn new(
        runtime: &RuntimeHandle,
        selection: Option<v1::CapturedRecordSelection>,
        after: Option<v1::Cursor>,
        through: Option<v1::Cursor>,
        bytes: usize,
    ) -> Result<Self, Status> {
        let source = runtime.captured_records();
        let selection = Selection::new(selection)?;
        let validate = |cursor: &v1::Cursor| {
            if cursor.epoch != source.epoch() || cursor.stream != selection.stream {
                return Err(Status::failed_precondition(
                    "record cursor epoch/selection mismatch; resynchronize without cursors",
                ));
            }
            if cursor.position > source.frontier() {
                return Err(Status::out_of_range("record cursor is ahead of runtime"));
            }
            Ok(cursor.position)
        };
        let scanned = after.as_ref().map(validate).transpose()?.unwrap_or(0);
        let through = through
            .as_ref()
            .map(validate)
            .transpose()?
            .unwrap_or(source.frontier());
        if scanned > through {
            return Err(Status::invalid_argument(
                "record cursor exceeds replay frontier",
            ));
        }
        Ok(Self {
            runtime: runtime.clone(),
            source,
            selection,
            scanned,
            through,
            bytes: bytes.min(PAGE_BYTES),
        })
    }
    fn cursor(&self, position: u64) -> v1::Cursor {
        v1::Cursor {
            epoch: self.source.epoch().into(),
            stream: self.selection.stream.clone(),
            position,
        }
    }
    async fn page(&mut self) -> Result<Page, Status> {
        use harmos::MemorySize;
        let preparation = self
            .runtime
            .storage()
            .prepare_production(std::num::NonZeroUsize::new(PAGE_BYTES).unwrap())
            .map_err(memory_error)?;
        let allowance = preparation.allowance().get();
        let mut scanned = self.scanned;
        let envelope_memory = std::mem::size_of::<WirePage>()
            .saturating_add(RECORD_LIMIT.saturating_mul(std::mem::size_of::<v1::CapturedRecord>()))
            .saturating_add(wire::copy_memory(self.source.epoch().len()).saturating_mul(3))
            .saturating_add(wire::copy_memory(self.selection.stream.len()).saturating_mul(3));
        if envelope_memory > allowance {
            return Err(Status::resource_exhausted(
                "record page envelope exceeds native memory allowance",
            ));
        }
        let page = v1::RecordPage {
            records: Vec::with_capacity(RECORD_LIMIT),
            scanned: Some(self.cursor(scanned)),
            through: Some(self.cursor(self.through)),
            replay_complete: false,
            retained_after: 0,
        };
        // Reserve the maximum cursor varint and final boolean before accepting
        // records. A rejected candidate is never acknowledged by scanned.
        let overhead = page.encoded_len() + 32;
        if overhead > self.bytes {
            return Err(Status::resource_exhausted(
                "record page envelope exceeds transport limit",
            ));
        }
        let mut bytes = overhead;
        let cursor_scratch = page
            .scanned
            .as_ref()
            .unwrap()
            .epoch
            .capacity()
            .saturating_add(page.scanned.as_ref().unwrap().stream.capacity());
        let envelope = WirePage(page);
        let mut memory = envelope.memory_size().saturating_add(cursor_scratch);
        let WirePage(mut page) = envelope;
        if memory > allowance {
            return Err(Status::resource_exhausted(
                "record page envelope exceeds native memory allowance",
            ));
        }
        for _ in 0..SCAN_LIMIT {
            let next = self
                .source
                .next(scanned, self.through)
                .await
                .map_err(capture_error)?;
            let Some(record) = next else {
                scanned = self.through;
                break;
            };
            if self.selection.matches(&record) {
                let candidate_memory = wire::allocation_memory(&record)?;
                if memory.saturating_add(candidate_memory) > allowance {
                    if page.records.is_empty() {
                        return Err(Status::resource_exhausted(
                            "captured record exceeds native memory allowance; cursor has not advanced",
                        ));
                    }
                    break;
                }
                let encoded = wire::record(&record)?;
                let size = encoded.encoded_len().saturating_add(10);
                if bytes.saturating_add(size) > self.bytes {
                    if page.records.is_empty() {
                        return Err(Status::resource_exhausted(
                            "captured record exceeds page limit; cursor has not advanced",
                        ));
                    }
                    break;
                }
                bytes += size;
                memory = memory.saturating_add(candidate_memory);
                page.records.push(encoded);
            }
            scanned = record.sequence;
            if page.records.len() == RECORD_LIMIT {
                break;
            }
        }
        page.scanned = Some(self.cursor(scanned));
        page.replay_complete = scanned == self.through;
        let output = preparation
            .finish(WirePage(page))
            .map_err(memory_error)?
            .try_into_value()
            .map_err(memory_error)?;
        self.scanned = scanned;
        Ok(Page(output))
    }
}

#[tonic::async_trait]
impl Evidence for Service {
    type WatchRecordsStream = Pin<Box<dyn Stream<Item = Result<Page, Status>> + Send + 'static>>;

    async fn list_records(
        &self,
        request: Request<v1::ListRecordsRequest>,
    ) -> Result<Response<Page>, Status> {
        let request = request.into_inner();
        if request.after.is_some() != request.through.is_some() {
            return Err(Status::invalid_argument(
                "pagination requires both after and through cursors",
            ));
        }
        let mut reader = Reader::new(
            &self.runtime,
            request.selection,
            request.after,
            request.through,
            self.max_message_bytes,
        )?;
        Ok(Response::new(reader.page().await?))
    }

    async fn watch_records(
        &self,
        request: Request<v1::WatchRecordsRequest>,
    ) -> Result<Response<Self::WatchRecordsStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let request = request.into_inner();
        let reader = Reader::new(
            &self.runtime,
            request.selection,
            request.after,
            None,
            self.max_message_bytes,
        )?;
        let frames = stream::unfold(
            Some((reader, self.stopping.clone(), true)),
            |state| async move {
                let (mut reader, mut stopping, initial) = state?;
                if *stopping.borrow() {
                    return None;
                }
                let next = async {
                    if !initial && reader.scanned == reader.through {
                        match reader
                            .source
                            .wait_after(reader.scanned)
                            .await
                            .map_err(capture_error)?
                        {
                            Some(through) => reader.through = through,
                            None => return Ok(None),
                        }
                    }
                    reader.page().await.map(Some)
                };
                let result = tokio::select! { biased;
                    _ = stopping.changed() => return None,
                    result = next => result,
                };
                match result {
                    Ok(Some(page)) => Some((Ok(page), Some((reader, stopping, false)))),
                    Ok(None) => None,
                    Err(error) => Some((Err(error), None)),
                }
            },
        );
        Ok(Response::new(Box::pin(frames)))
    }

    async fn get_captured_record_schema(
        &self,
        request: Request<v1::GetCapturedRecordSchemaRequest>,
    ) -> Result<Response<v1::RecordSchema>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        if request.epoch != self.runtime.captured_records().epoch() {
            return Err(Status::failed_precondition(
                "record schema epoch mismatch; resynchronize",
            ));
        }
        super::value::text(&request.record_type)?;
        let kind = dokime_app::core::Identifier::new(request.record_type)
            .map_err(|_| Status::invalid_argument("invalid record type"))?;
        let contract = snapshot
            .state()
            .historical_record_contract(&kind, request.schema_version)
            .ok_or_else(|| Status::not_found("exact admitted record schema is unavailable"))?;
        let schema = super::catalog::schemas::record(contract)?;
        if schema.encoded_len() > self.max_message_bytes {
            return Err(Status::resource_exhausted(
                "record schema exceeds transport response budget",
            ));
        }
        Ok(Response::new(schema))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn memory_error(error: harmos::Error) -> Status {
    let error = dokime_app::Error::from(error);
    detailed(
        Status::resource_exhausted(format!("captured record output unavailable: {error}")),
        &error,
    )
}

fn capture_error(error: dokime_app::Error) -> Status {
    detailed(
        Status::unavailable(format!("captured record source unavailable: {error}")),
        &error,
    )
}

/* === Tests === */
