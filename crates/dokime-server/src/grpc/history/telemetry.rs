/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime::harmos::{Context, Response};
use tonic::{Code, Status};

use super::super::{telemetry::projection as samples, value::projection};
use super::wire;
use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
use dokime::operations::history as contract;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::query::checkpoint_telemetry::ExecutionTelemetryPage;

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(contract::ListCheckpointTelemetry),
}

impl Serve<contract::ListCheckpointTelemetry> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::ListCheckpointTelemetry, _: Context<'a, contract::ListCheckpointTelemetry>,
    ) -> Served<'a, dokime::domain::model::history::wire::CheckpointTelemetryPage> {
        Box::pin(async move {
            let stream = dokime::domain::model::identity::Identifier::new(std::mem::take(&mut request.table))
                .map_err(|_| Status::invalid_argument("native telemetry stream identifier is required"))?;
            let checkpoint = wire::checkpoint(request.checkpoint.take())?;
            let query = dokime::runtime::query::checkpoint_telemetry::ExecutionTelemetryQuery {
                after: (!request.after.is_empty()).then(|| std::mem::take(&mut request.after)),
                limit: (request.limit != 0).then_some(request.limit as usize),
            };
            dokime::app::live_runtime(&self.runtime)
                .await?
                .execution_checkpoint_telemetry(checkpoint, &stream, query)
                .await
                .map_err(wire::query_failure)
                .and_then(|page| project(page, self.maximum).map_err(|source| wire::query_failure(source.into())))
        })
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<contract::ListCheckpointTelemetry>>(Host::<contract::ListCheckpointTelemetry>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

fn project(
    page: dokime::harmos::storage::ReadBatch<ExecutionTelemetryPage>, maximum: usize,
) -> dokime::harmos::Result<Response<dokime::domain::model::history::wire::CheckpointTelemetryPage>> {
    let projected = page.try_map(|page| project_wire(page, maximum))?;
    if projected.wire_len()? > maximum {
        return Err(exhausted());
    }
    Ok(Response::from_owner(projected.owner()))
}

fn project_wire(
    page: ExecutionTelemetryPage, maximum: usize,
) -> dokime::harmos::Result<dokime::domain::model::history::wire::CheckpointTelemetryPage> {
    let mut records = projection::slots(page.rows.len())?;
    for row in &page.rows {
        records.push(dokime::domain::model::history::wire::SavedTelemetrySample {
            epoch: projection::text(row.epoch.as_str())?,
            ordinal: row.ordinal,
            sample: Some(samples::sample(&row.sample).map_err(original_error)?),
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
    let wire = dokime::domain::model::history::wire::CheckpointTelemetryPage {
        checkpoint: Some(super::wire::checkpoint_wire(page.checkpoint, maximum).map_err(original_error)?),
        table: page.table,
        rows: records,
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
    dokime::harmos::Error::backend("encode original saved telemetry", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical saved telemetry page",
    }
}
