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
use dokime_domain::operations::history as contract;
use dokime_runtime::RuntimeHandle;
use dokime_runtime::runtime::query::checkpoint_records::ExecutionRecordPage;

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
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::ListCheckpointRecords,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::history::wire::CheckpointRecordPage>, Status> {
        value::text(&request.after)?;
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let query = dokime_runtime::runtime::query::checkpoint_records::ExecutionRecordQuery {
            after: (!request.after.is_empty()).then(|| std::mem::take(&mut request.after)),
            limit: (request.limit != 0).then_some(request.limit as usize),
        };
        self.resources
            .runtime
            .execution_checkpoint_records(checkpoint, query)
            .await
            .map_err(wire::query_failure)
            .and_then(|page| {
                project(&page, self.resources.maximum)
                    .map(Into::into)
                    .map_err(|source| wire::query_failure(source.into()))
            })
    }
}

/* === Public Functions === */
fn project(
    page: &ExecutionRecordPage,
    maximum: usize,
) -> harmos::Result<dokime_domain::model::history::wire::CheckpointRecordPage> {
    let mut records = projection::slots(page.records.len())?;
    for row in &page.records {
        records.push(dokime_domain::model::history::wire::SavedRecord {
            epoch: projection::text(row.epoch.as_str())?,
            record: Some(evidence::record(&row.record).map_err(original_error)?),
        });
    }
    let mut issues = projection::slots(page.issues.len())?;
    for issue in &page.issues {
        issues.push(dokime_domain::model::history::wire::HistoryIssue {
            source: issue.source.as_deref().map(projection::text).transpose()?,
            dataset: issue.dataset.as_deref().map(projection::text).transpose()?,
            message: projection::text(&issue.message)?,
        });
    }
    let wire = dokime_domain::model::history::wire::CheckpointRecordPage {
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
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
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
