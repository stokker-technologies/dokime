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
use std::collections::VecDeque;
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::Response;
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

// Local Imports
use super::error::detailed;
use crate::grpc::transcode::Transcode;
use dokime::domain::model::definition::wire::RecordSchema;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::operations::evidence as v1;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::{RuntimeHandle, runtime::CapturedRecords};
use selection::Selection;

/* === Constants === */

const SERVER_EVIDENCE_PAGE_BYTES: u64 = 524_288;
const SERVER_EVIDENCE_RECORD_ITEMS: u64 = 1_000;
const SERVER_EVIDENCE_SCAN_ITEMS: u64 = 1_000;

/* === Macros === */

/* === Type Aliases === */

type Page = dokime::domain::model::evidence::wire::RecordPage;
type TailEntry = (usize, dokime::domain::model::evidence::wire::CapturedRecord);

/* === Structs === */

struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct Reader {
    scan_items: usize,
    record_items: usize,
    source: CapturedRecords,
    selection: Selection,
    scanned: u64,
    through: u64,
    bytes: usize,
}

// The newest matching records of a tail scan, bounded by count and bytes.
struct Tail {
    page: Page,
    kept: VecDeque<TailEntry>,
    keep: usize,
    budget: usize,
    bytes: usize,
    oversized: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Reader {
    fn new(
        runtime: &RuntimeHandle, selection: Option<dokime::domain::model::evidence::wire::CapturedRecordSelection>, after: Option<Cursor>,
        through: Option<Cursor>, bytes: usize,
    ) -> Result<Self, Status> {
        let scan_items = usize::try_from(SERVER_EVIDENCE_SCAN_ITEMS).expect("validated record scan limit");
        let record_items = usize::try_from(SERVER_EVIDENCE_RECORD_ITEMS).expect("validated record page count");
        let page_bytes = usize::try_from(SERVER_EVIDENCE_PAGE_BYTES).expect("validated record page bytes");
        let session = runtime.storage().clone();
        let source = runtime.captured_records_in(&session);
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
        Ok(Self {
            source,
            selection,
            scanned,
            through,
            bytes: bytes.min(page_bytes),
            scan_items,
            record_items,
        })
    }
    fn cursor(&self, position: u64) -> dokime::harmos::Result<Cursor> {
        Ok(Cursor {
            epoch: super::value::projection::text(self.source.epoch())?,
            stream: super::value::projection::text(&self.selection.stream)?,
            position,
        })
    }
    fn empty_page(&self, capacity: usize) -> Result<dokime::harmos::storage::ReadBatch<Page>, Status> {
        let seed = dokime::harmos::storage::ReadBatch::new(());
        seed.try_map(|_| {
            Ok::<_, dokime::harmos::Error>(Page {
                records: super::value::projection::slots(capacity)?,
                scanned: Some(self.cursor(self.through)?),
                through: Some(self.cursor(self.through)?),
                replay_complete: true,
                retained_after: 0,
            })
        })
        .map_err(memory_error)
    }

    async fn page(&mut self) -> Result<dokime::harmos::storage::ReadBatch<Page>, Status> {
        let mut scanned = self.scanned;
        let mut page = self.empty_page(self.record_items)?;
        // Final Frontier And Completion Bound Every Intermediate Cursor Encoding.
        let mut bytes = page.wire_len()?;

        if bytes > self.bytes {
            return Err(exhausted());
        }
        for _ in 0..self.scan_items {
            let next = self.source.next(scanned, self.through).await.map_err(capture_error)?;
            let Some(record) = next else {
                scanned = self.through;
                break;
            };
            if self.selection.matches(&record) {
                let mut accepted = false;
                page = page
                    .try_map(|mut page| {
                        let encoded = canonical::record(&record).map_err(projection_error)?;
                        let size = record_wire_size(&encoded).map_err(projection_error)?;
                        let total = bytes.checked_add(size).ok_or_else(|| projection_error(exhausted()))?;
                        if total > self.bytes {
                            if page.records.is_empty() {
                                return Err(projection_error(Status::resource_exhausted(
                                    "captured record exceeds page limit; cursor has not advanced",
                                )));
                            }
                            return Ok(page);
                        }
                        bytes = total;
                        accepted = true;
                        page.records.push(encoded);
                        Ok::<_, dokime::harmos::Error>(page)
                    })
                    .map_err(memory_error)?;
                if !accepted {
                    break;
                }
            }
            scanned = record.sequence;
            if page.records.len() == self.record_items {
                break;
            }
        }
        page = page
            .try_map(|mut page| {
                page.scanned.as_mut().expect("page has scanned cursor").position = scanned;
                page.replay_complete = scanned == self.through;
                Ok::<_, dokime::harmos::Error>(page)
            })
            .map_err(memory_error)?;
        self.scanned = scanned;
        Ok(page)
    }

    async fn tail(&mut self, latest: usize) -> Result<dokime::harmos::storage::ReadBatch<Page>, Status> {
        let keep = latest.min(self.record_items);
        let page = self.empty_page(0)?;
        let overhead = page.wire_len()?;
        let budget = self.bytes.checked_sub(overhead).ok_or_else(exhausted)?;
        let mut window = page
            .try_map(|page| {
                let mut kept = VecDeque::new();
                kept.try_reserve_exact(keep).map_err(|_| projection_error(exhausted()))?;

                Ok::<_, dokime::harmos::Error>(Tail {
                    page,
                    kept,
                    keep,
                    budget,
                    bytes: 0,
                    oversized: false,
                })
            })
            .map_err(memory_error)?;
        let mut scanned = self.scanned;
        while let Some(record) = self.source.next(scanned, self.through).await.map_err(capture_error)? {
            scanned = record.sequence;
            if !self.selection.matches(&record) {
                continue;
            }

            window = window
                .try_map(|mut window| {
                    window.offer(&record).map_err(projection_error)?;
                    Ok::<_, dokime::harmos::Error>(window)
                })
                .map_err(memory_error)?;
        }
        let page = window
            .try_map(|mut window| {
                if window.oversized {
                    return Err(projection_error(Status::resource_exhausted("newest captured record exceeds page limit")));
                }
                let mut records = super::value::projection::slots(window.kept.len())?;
                for (_, record) in window.kept {
                    records.push(record);
                }
                window.page.records = records;
                Ok::<_, dokime::harmos::Error>(window.page)
            })
            .map_err(memory_error)?;
        self.scanned = self.through;
        Ok(page)
    }
}

impl Tail {
    fn offer(&mut self, record: &dokime::runtime::CapturedRecord) -> Result<(), Status> {
        let encoded = canonical::record(record)?;
        let size = record_wire_size(&encoded)?;
        self.oversized = size > self.budget;
        while !self.kept.is_empty() {
            let total = self.bytes.checked_add(size).ok_or_else(exhausted)?;
            if self.kept.len() < self.keep && total <= self.budget {
                break;
            }
            let (bytes, record) = self.kept.pop_front().expect("nonempty tail");
            self.bytes -= bytes;
            drop(record);
        }
        if self.oversized || self.keep == 0 {
            return Ok(());
        }
        self.bytes = self.bytes.checked_add(size).ok_or_else(exhausted)?;
        self.kept.push_back((size, encoded));
        Ok(())
    }
}

impl Serve<v1::ListRecords> for Resources {
    fn serve<'a>(
        &'a self, mut request: v1::ListRecords, _: Context<'a, v1::ListRecords>,
    ) -> Served<'a, dokime::domain::model::evidence::wire::RecordPage> {
        Box::pin(async move {
            if request.after.is_some() != request.through.is_some() {
                return Err(Status::invalid_argument("pagination requires both after and through cursors"));
            }
            if request.latest > 0 && request.after.is_some() {
                return Err(Status::invalid_argument("a tail request takes no cursors"));
            }
            let mut reader =
                Reader::new(&self.runtime, request.selection.take(), request.after.take(), request.through.take(), self.max_message_bytes)?;
            let page = match request.latest {
                0 => reader.page().await?,
                latest => reader.tail(latest as usize).await?,
            };
            Ok(Response::from_owner(page.owner()))
        })
    }
}

impl Serve<v1::WatchRecords> for Resources {
    fn serve<'a>(&'a self, mut request: v1::WatchRecords, ctx: Context<'a, v1::WatchRecords>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let mut reader = Reader::new(&self.runtime, request.selection.take(), request.after.take(), None, self.max_message_bytes)?;
                let mut initial = true;
                loop {
                    if !initial && reader.scanned == reader.through {
                        match reader.source.wait_after(reader.scanned).await.map_err(capture_error)? {
                            Some(through) => reader.through = through,
                            None => return Ok(().into()),
                        }
                    }
                    let page = reader.page().await?;
                    ctx.streams().send_owned(Response::from_owner(page.owner())).await.map_err(memory_error)?;
                    initial = false;
                }
            })
            .await
        })
    }
}

impl Serve<v1::GetCapturedRecordSchema> for Resources {
    fn serve<'a>(&'a self, request: v1::GetCapturedRecordSchema, _: Context<'a, v1::GetCapturedRecordSchema>) -> Served<'a, RecordSchema> {
        Box::pin(async move {
            let snapshot = self.runtime.watch_state().latest();
            if request.epoch != self.runtime.captured_records().epoch() {
                return Err(Status::failed_precondition("record schema epoch mismatch; resynchronize"));
            }
            let kind = dokime::domain::model::identity::Identifier::new(request.record_type)
                .map_err(|_| Status::invalid_argument("invalid record type"))?;
            let contract = snapshot
                .state()
                .historical_record_contract(&kind, request.schema_version)
                .ok_or_else(|| Status::not_found("exact admitted record schema is unavailable"))?;
            let schema = super::catalog::schemas::record(contract)?;
            if schema.wire_len()? > self.max_message_bytes {
                return Err(Status::resource_exhausted("record schema exceeds transport response budget"));
            }
            Ok(schema.into())
        })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), super::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes: maximum,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<v1::ListRecords>::new(served.clone()))?;
    dependencies.insert(Host::<v1::WatchRecords>::new(served.clone()))?;
    dependencies.insert(Host::<v1::GetCapturedRecordSchema>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<v1::ListRecords>()
        .operation::<v1::WatchRecords>()
        .operation::<v1::GetCapturedRecordSchema>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

/* === Private Functions === */

fn record_wire_size(record: &dokime::domain::model::evidence::wire::CapturedRecord) -> Result<usize, Status> {
    let body = record.wire_len()?;
    let length = u64::try_from(body).map_err(|_| exhausted())?;
    // RecordPage.records Is Declared As Repeated Message Field 1.
    body.checked_add(prost::encoding::key_len(1))
        .and_then(|size| size.checked_add(prost::encoding::encoded_len_varint(length)))
        .ok_or_else(exhausted)
}

fn exhausted() -> Status {
    Status::resource_exhausted("captured record page capacity exceeded")
}

fn projection_error(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return dokime::harmos::Error::ResourceExhausted { resource: "captured record projection" };
    }
    dokime::harmos::Error::backend("project captured record", dokime::harmos::BackendFailure::Corrupt, source)
}

fn memory_error(error: dokime::harmos::Error) -> Status {
    let error = dokime::runtime::Error::from(error);
    detailed(Status::resource_exhausted(format!("captured record output unavailable: {error}")), &error)
}

fn capture_error(error: dokime::runtime::Error) -> Status {
    detailed(Status::unavailable(format!("captured record source unavailable: {error}")), &error)
}

/* === Tests === */
