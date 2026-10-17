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
use harmos::operations::{Context, Operation, Response};
use tonic::Status;
// Local Imports
use super::super::value;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::content as wire;
use dokime_runtime::RuntimeHandle;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Read {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Read {
    type Declaration = wire::ReadExecutionFile;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ReadExecutionFile, _: &mut (),
    ) -> Result<Response<dokime_domain::model::content::wire::ExecutionFileWindow>, Status> {
        let runtime = &self.resources.runtime;
        let message_limit = self.resources.maximum;
        let (id, _) = execution(runtime, &request.execution_id)?;
        value::text(&request.relative_path)?;
        if request.relative_path.is_empty() {
            return Err(Status::invalid_argument("output name is required"));
        }
        if request.max_bytes > 64 * 1024 {
            return Err(Status::invalid_argument("read limit exceeds 64 KiB"));
        }
        let limit = if request.max_bytes == 0 { 64 * 1024 } else { request.max_bytes as usize };
        // Reserve room for the selected name, media type and scalar metadata.
        let limit = limit.min(message_limit.saturating_sub(request.relative_path.len() + 512));
        if limit == 0 {
            return Err(Status::resource_exhausted("configured message limit cannot contain file window"));
        }
        let page = runtime
            .execution_file_content(&id, std::mem::take(&mut request.relative_path), request.offset, limit)
            .await
            .map_err(read_failure)?
            .ok_or_else(|| Status::not_found("registered execution output not found"))?;
        super::window::project(&page, message_limit).map(Into::into).map_err(|source| read_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Read>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
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

/* === Private Functions === */
pub(super) fn read_failure(error: dokime_runtime::runtime::Error) -> Status {
    let status = match &error {
        dokime_runtime::runtime::Error::Harmos(harmos::Error::InvalidInput { field: "execution files cursor", .. }) => {
            Status::aborted("file listing changed; restart pagination")
        }
        dokime_runtime::runtime::Error::Harmos(harmos::Error::InvalidInput { field: "execution files after path", .. }) => {
            Status::invalid_argument("after path not found")
        }
        dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. })
        | dokime_runtime::runtime::Error::ExecutionFileTooLarge => {
            Status::resource_exhausted("execution output exceeds its original budget")
        }
        _ => Status::internal("cannot read execution output"),
    };
    super::super::error::detailed(status, &error)
}

/* === Tests === */
