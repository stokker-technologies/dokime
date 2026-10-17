/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod canonical;
mod selection;

/* === Re-Exports === */
/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use crate::grpc::transcode::Transcode;
use harmos::domain::Portable;
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;
// Local Imports
use super::error::detailed;
use dokime_domain::model::definition::wire::RecordSchema;
use dokime_domain::model::execution::wire_cursor::Cursor;
use dokime_domain::operations::evidence as v1;
use dokime_runtime::{RuntimeHandle, runtime::CapturedRecords};

use selection::Selection;

/* === Constants === */
const SCAN_LIMIT: usize = 1000;
const RECORD_LIMIT: usize = 100;
const PAGE_BYTES: usize = 512 * 1024;

/* === Macros === */

/* === Type Aliases === */
type Page = dokime_domain::model::evidence::wire::RecordPage;

/* === Structs === */
struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct Reader {
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
        runtime: &RuntimeHandle, selection: Option<dokime_domain::model::evidence::wire::CapturedRecordSelection>, after: Option<Cursor>,
        through: Option<Cursor>, bytes: usize,
    ) -> Result<Self, Status> {
        let source = runtime.captured_records();
        let selection = Selection::new(selection)?;
        let validate = |cursor: &Cursor| {
            if cursor.epoch != source.epoch() || cursor.stream != selection.stream {
                return Err(Status::failed_precondition("record cursor epoch/selection mismatch; resynchronize without cursors"));
            }
            if cursor.position > source.frontier() {
                return Err(Status::out_of_range("record cursor is ahead of runtime"));
            }
            Ok(cursor.position)
        };
        let scanned = after.as_ref().map(validate).transpose()?.unwrap_or(0);
        let through = through.as_ref().map(validate).transpose()?.unwrap_or(source.frontier());
        if scanned > through {
            return Err(Status::invalid_argument("record cursor exceeds replay frontier"));
        }
        Ok(Self { source, selection, scanned, through, bytes: bytes.min(PAGE_BYTES) })
    }
    fn cursor(&self, position: u64) -> Cursor {
        Cursor { epoch: self.source.epoch().into(), stream: self.selection.stream.clone(), position }
    }
    async fn page(&mut self) -> Result<Page, Status> {
        let mut scanned = self.scanned;
        let mut page = dokime_domain::model::evidence::wire::RecordPage {
            records: Vec::with_capacity(RECORD_LIMIT),
            scanned: Some(self.cursor(scanned)),
            through: Some(self.cursor(self.through)),
            replay_complete: false,
            retained_after: 0,
        };
        // Reserve the maximum cursor varint and final boolean before accepting
        // records. A rejected candidate is never acknowledged by scanned.
        let overhead = page.wire_len()? + 32;
        if overhead > self.bytes {
            return Err(Status::resource_exhausted("record page envelope exceeds transport limit"));
        }
        let mut bytes = overhead;
        for _ in 0..SCAN_LIMIT {
            let next = self.source.next(scanned, self.through).await.map_err(capture_error)?;
            let Some(record) = next else {
                scanned = self.through;
                break;
            };
            if self.selection.matches(&record) {
                let encoded = canonical::record(&record)?;
                let size = encoded.wire_len()?.saturating_add(10);
                if bytes.saturating_add(size) > self.bytes {
                    if page.records.is_empty() {
                        return Err(Status::resource_exhausted("captured record exceeds page limit; cursor has not advanced"));
                    }
                    break;
                }
                bytes += size;
                page.records.push(encoded);
            }
            scanned = record.sequence;
            if page.records.len() == RECORD_LIMIT {
                break;
            }
        }
        page.scanned = Some(self.cursor(scanned));
        page.replay_complete = scanned == self.through;
        self.scanned = scanned;
        Ok(page)
    }
}

struct List {
    resources: Arc<Resources>,
}
struct Watch {
    resources: Arc<Resources>,
}
struct Schema {
    resources: Arc<Resources>,
}
impl Operation for List {
    type Declaration = v1::ListRecords;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::ListRecords, _: &mut (),
    ) -> Result<Response<dokime_domain::model::evidence::wire::RecordPage>, Status> {
        if request.after.is_some() != request.through.is_some() {
            return Err(Status::invalid_argument("pagination requires both after and through cursors"));
        }
        let mut reader = Reader::new(
            &self.resources.runtime,
            request.selection.take(),
            request.after.take(),
            request.through.take(),
            self.resources.max_message_bytes,
        )?;
        Ok(reader.page().await?.into())
    }
}
impl Operation for Watch {
    type Declaration = v1::WatchRecords;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::evidence::wire::RecordPage>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::WatchRecords, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let mut reader =
            Reader::new(&self.resources.runtime, request.selection.take(), request.after.take(), None, self.resources.max_message_bytes)?;
        let mut initial = true;
        loop {
            if !initial && reader.scanned == reader.through {
                match reader.source.wait_after(reader.scanned).await.map_err(capture_error)? {
                    Some(through) => reader.through = through,
                    None => return Ok(().into()),
                }
            }
            output.send_owned(reader.page().await?.into()).await.map_err(memory_error)?;
            initial = false;
        }
    }
}
impl Operation for Schema {
    type Declaration = v1::GetCapturedRecordSchema;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut v1::GetCapturedRecordSchema, _: &mut (),
    ) -> Result<Response<RecordSchema>, Status> {
        let snapshot = self.resources.runtime.watch_state().latest();
        if request.epoch != self.resources.runtime.captured_records().epoch() {
            return Err(Status::failed_precondition("record schema epoch mismatch; resynchronize"));
        }
        super::value::text(&request.record_type)?;
        let kind = dokime_domain::model::identity::Identifier::new(std::mem::take(&mut request.record_type))
            .map_err(|_| Status::invalid_argument("invalid record type"))?;
        let contract = snapshot
            .state()
            .historical_record_contract(&kind, request.schema_version)
            .ok_or_else(|| Status::not_found("exact admitted record schema is unavailable"))?;
        let schema = super::catalog::schemas::record(contract)?;
        if schema.wire_len()? > self.resources.max_message_bytes {
            return Err(Status::resource_exhausted("record schema exceeds transport response budget"));
        }
        Ok(schema.into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), max_message_bytes: maximum });
    runtime.operations().register_wire_operation::<List>("dokime", dokime_runtime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Watch>("dokime", dokime_runtime::app::Lanes::Open, resources.clone())?;
    runtime.operations().register_wire_operation::<Schema>("dokime", dokime_runtime::app::Lanes::Open, resources)?;
    Ok(())
}

/* === Private Functions === */
fn memory_error(error: harmos::Error) -> Status {
    let error = dokime_runtime::runtime::Error::from(error);
    detailed(Status::resource_exhausted(format!("captured record output unavailable: {error}")), &error)
}

fn capture_error(error: dokime_runtime::runtime::Error) -> Status {
    detailed(Status::unavailable(format!("captured record source unavailable: {error}")), &error)
}

/* === Tests === */
