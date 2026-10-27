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
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::{Code, Status};

// Local Imports
use super::super::{evidence::canonical as evidence, value::projection};
use super::wire;
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::domain::operations::history as contract;
use dokime::runtime::query::checkpoint_records::ExecutionRecordPage;

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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::ListCheckpointRecords, _: &mut (),
    ) -> Result<Response<dokime::domain::model::history::wire::CheckpointRecordPage>, Status> {
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let query = dokime::runtime::query::checkpoint_records::ExecutionRecordQuery {
            after: (!request.after.is_empty()).then(|| std::mem::take(&mut request.after)),
            limit: (request.limit != 0).then_some(request.limit as usize),
        };
        self.resources
            .runtime
            .execution_checkpoint_records(checkpoint, query)
            .await
            .map_err(wire::query_failure)
            .and_then(|page| project(page, self.resources.maximum).map_err(|source| wire::query_failure(source.into())))
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<List>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn project(
    page: dokime::harmos::storage::ReadBatch<ExecutionRecordPage>, maximum: usize,
) -> dokime::harmos::Result<Response<dokime::domain::model::history::wire::CheckpointRecordPage>> {
    let projected = page.try_map(|page| project_wire(page, maximum))?;
    if projected.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(Response::from_owner(projected.owner()))
}

fn project_wire(
    page: ExecutionRecordPage, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::history::wire::CheckpointRecordPage> {
    let mut records = projection::slots(page.records.len())?;
    for row in &page.records {
        records.push(dokime::domain::model::history::wire::SavedRecord {
            epoch: projection::text(row.epoch.as_str())?,
            record: Some(evidence::record(&row.record).map_err(original_error)?),
        });
    }
    let mut issues = projection::slots(page.issues.len())?;
    for issue in page.issues {
        issues.push(dokime::domain::model::history::wire::HistoryIssue {
            source: issue.source,
            dataset: issue.dataset,
            message: issue.message,
        });
    }
    let wire = dokime::domain::model::history::wire::CheckpointRecordPage {
        checkpoint: Some(super::wire::checkpoint_wire(page.checkpoint, maximum).map_err(original_error)?),
        records,
        next_after: page.next,
        issues,
        issues_truncated: page.issues_truncated,
    };
    Ok(wire)
}

fn original_error(source: Status) -> dokime::harmos::Error {
    if source.code() == Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("encode original checkpoint record", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical checkpoint record page" }
}

/* === Tests === */
