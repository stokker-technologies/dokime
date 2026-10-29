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

pub(in crate::grpc) use super::super::enums::canonical::{execution_file_kind as kind, execution_file_state as state};

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

// Local Imports
use dokime::RuntimeHandle;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::content as wire;
use dokime::operations::hosting::{Host, Serve, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::ReadExecutionFile> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ReadExecutionFile, _: Context<'a, wire::ReadExecutionFile>,
    ) -> Served<'a, dokime::domain::model::content::wire::ExecutionFileWindow> {
        Box::pin(async move {
            let runtime = &self.runtime;
            let message_limit = self.maximum;
            let (id, _) = execution(runtime, &request.execution_id)?;
            if request.relative_path.is_empty() {
                return Err(Status::invalid_argument("output name is required"));
            }
            if request.max_bytes > 64 * 1024 {
                return Err(Status::invalid_argument("file window exceeds 65536 bytes"));
            }
            let limit = if request.max_bytes == 0 { 64 * 1024 } else { request.max_bytes as usize };
            // The Projection Fits The Complete Encoded Window After Reading.
            let limit = limit.min(message_limit);
            if limit == 0 {
                return Err(Status::resource_exhausted("configured message limit cannot contain file window"));
            }
            let page = runtime
                .execution_file_content(&id, request.relative_path, request.offset, limit)
                .await
                .map_err(read_failure)?
                .ok_or_else(|| Status::not_found("registered execution output not found"))?;
            super::window::owned(page, message_limit).map_err(|source| read_failure(source.into()))
        })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::ReadExecutionFile>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<wire::ReadExecutionFile>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

pub(super) fn execution(runtime: &RuntimeHandle, id: &str) -> Result<(TestExecutionId, String), Status> {
    let id = TestExecutionId::from_uuid(id.to_owned()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let snapshot = runtime.watch_state().latest();
    if !snapshot.state().executions.contains_key(&id) {
        return Err(Status::not_found("execution not found in current runtime"));
    }
    Ok((id, snapshot.state().process_epoch().to_string()))
}

pub(super) fn read_failure(error: dokime::runtime::Error) -> Status {
    let status = match &error {
        dokime::runtime::Error::Harmos(dokime::harmos::Error::InvalidInput { field: "execution files cursor", .. }) => {
            Status::aborted("file listing changed; restart pagination")
        }
        dokime::runtime::Error::Harmos(dokime::harmos::Error::InvalidInput { field: "execution files after path", .. }) => {
            Status::invalid_argument("after path not found")
        }
        dokime::runtime::Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. }) => {
            Status::resource_exhausted("execution output exceeds its original budget")
        }
        _ => Status::internal("cannot read execution output"),
    };
    super::super::error::detailed(status, &error)
}

/* === Private Functions === */

/* === Tests === */
