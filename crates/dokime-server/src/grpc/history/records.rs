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
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::{
    Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::{Code, Status};

// Local Imports
use super::super::{
    evidence::canonical as evidence,
    value::{self, projection},
};
use super::wire;
use dokime_app::RuntimeHandle;
use dokime_app::runtime::query::checkpoint_records::ExecutionRecordPage;
use dokime_contract::operations::history as contract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct List {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for List {
    type Declaration = contract::ListCheckpointRecords;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    fn new(resources: Arc<Resources>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::ListCheckpointRecordsRequest,
        _: &mut (),
    ) -> Result<Response<contract::CheckpointRecordPage>, Status> {
        value::text(&request.after)?;
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let query = dokime_app::runtime::query::checkpoint_records::ExecutionRecordQuery {
            after: (!request.after.is_empty()).then(|| std::mem::take(&mut request.after)),
            limit: (request.limit != 0).then_some(request.limit as usize),
        };
        self.resources
            .runtime
            .execution_checkpoint_records_projected(checkpoint, query, |page, allowance| {
                project(page, allowance, self.resources.maximum)
            })
            .await
            .map_err(wire::query_failure)?
            .try_into_response()
            .map_err(|source| wire::query_failure(source.into()))
    }
}

/* === Public Functions === */
fn project(
    page: &ExecutionRecordPage,
    allowance: NonZeroUsize,
    maximum: usize,
) -> harmos::Result<contract::CheckpointRecordPage> {
    // The Original Native Page Remains Charged Separately. These Bounds Admit
    // Every Wire Slot And Copied Field Before Constructing The Target Graph.
    let graph = graph_memory(page)?;
    if graph > allowance.get() {
        return Err(exhausted());
    }
    let mut records = projection::slots(page.records.len())?;
    for row in &page.records {
        records.push(contract::SavedRecord {
            epoch: projection::text(row.epoch.as_str())?,
            record: Some(evidence::record(&row.record).map_err(original_error)?),
        });
    }
    let mut issues = projection::slots(page.issues.len())?;
    for issue in &page.issues {
        issues.push(contract::HistoryIssue {
            source: issue.source.as_deref().map(projection::text).transpose()?,
            dataset: issue.dataset.as_deref().map(projection::text).transpose()?,
            message: projection::text(&issue.message)?,
        });
    }
    let wire = contract::CheckpointRecordPage {
        checkpoint: Some(
            super::wire::checkpoint_wire_ref(&page.checkpoint).map_err(original_error)?,
        ),
        records,
        next_after: page.next.as_deref().map(projection::text).transpose()?,
        issues,
        issues_truncated: page.issues_truncated,
    };
    if wire.encoded_len() > maximum {
        return Err(exhausted());
    }
    Ok(wire)
}

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn graph_memory(page: &ExecutionRecordPage) -> harmos::Result<usize> {
    let checkpoint = &page.checkpoint;
    let mut bytes = std::mem::size_of::<contract::CheckpointRecordPage>();
    for value in [
        page.records
            .len()
            .checked_mul(std::mem::size_of::<contract::SavedRecord>())
            .ok_or_else(exhausted)?,
        page.issues
            .len()
            .checked_mul(std::mem::size_of::<contract::HistoryIssue>())
            .ok_or_else(exhausted)?,
        checkpoint.execution_id.as_str().len(),
        harmos::Message::encoded_len(&checkpoint.version),
        checkpoint.position.history.store.as_str().len(),
        checkpoint.position.history.epoch.as_str().len(),
        checkpoint.evidence_through.history.store.as_str().len(),
        checkpoint.evidence_through.history.epoch.as_str().len(),
        page.next.as_ref().map_or(0, String::len),
    ] {
        bytes = bytes.checked_add(value).ok_or_else(exhausted)?;
    }
    for row in &page.records {
        let record_memory = evidence::memory(&row.record).map_err(original_error)?;
        bytes = bytes
            .checked_add(row.epoch.as_str().len())
            .and_then(|bytes| bytes.checked_add(record_memory))
            .ok_or_else(exhausted)?;
    }
    for issue in &page.issues {
        for length in [
            issue.message.len(),
            issue.source.as_ref().map_or(0, String::len),
            issue.dataset.as_ref().map_or(0, String::len),
        ] {
            bytes = bytes.checked_add(length).ok_or_else(exhausted)?;
        }
    }
    Ok(bytes)
}
fn original_error(source: Status) -> harmos::Error {
    if source.code() == Code::ResourceExhausted {
        return exhausted();
    }
    harmos::Error::backend(
        "encode original checkpoint record",
        harmos::BackendFailure::Corrupt,
        source,
    )
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint record page",
    }
}
/* === Tests === */
