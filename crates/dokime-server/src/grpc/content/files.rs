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

// Third Party Imports
use tonic::Status;
// Local Imports
use super::super::value;
use dokime_app::{RuntimeHandle, core::TestExecutionId};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn execution(
    runtime: &RuntimeHandle,
    id: &str,
) -> Result<(TestExecutionId, String), Status> {
    let id = TestExecutionId::from_uuid(id.to_owned())
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let snapshot = runtime.watch_state().latest();
    if !snapshot.state().executions.contains_key(&id) {
        return Err(Status::not_found("execution not found in current runtime"));
    }
    Ok((id, snapshot.state().process_epoch().to_string()))
}

pub(super) async fn list(
    runtime: &RuntimeHandle,
    request: v1::ListExecutionFilesRequest,
    message_limit: usize,
) -> Result<super::super::charged::Bytes, Status> {
    let (id, epoch) = execution(runtime, &request.execution_id)?;
    value::text(&request.after_path)?;
    if request.limit > 100 {
        return Err(Status::invalid_argument("page limit exceeds 100"));
    }
    if !request.after_path.is_empty() && request.cursor.is_none() {
        return Err(Status::invalid_argument("subsequent page requires cursor"));
    }
    if let Some(cursor) = &request.cursor
        && (cursor.epoch != epoch || cursor.execution_id != id.as_str())
    {
        return Err(Status::failed_precondition(
            "file cursor epoch or execution mismatch",
        ));
    }
    runtime
        .execution_files_projected(&id, |entries, allowance| {
            super::inventory::encode(entries, &id, &epoch, &request, allowance, message_limit)
        })
        .await
        .map_err(read_failure)
}

pub(super) async fn read(
    runtime: &RuntimeHandle,
    request: v1::ReadExecutionFileRequest,
    message_limit: usize,
) -> Result<super::super::charged::Bytes, Status> {
    let (id, _) = execution(runtime, &request.execution_id)?;
    value::text(&request.relative_path)?;
    if request.relative_path.is_empty() {
        return Err(Status::invalid_argument("output name is required"));
    }
    if request.max_bytes > 64 * 1024 {
        return Err(Status::invalid_argument("read limit exceeds 64 KiB"));
    }
    let limit = if request.max_bytes == 0 {
        64 * 1024
    } else {
        request.max_bytes as usize
    };
    // Reserve room for the selected name, media type and scalar metadata.
    let limit = limit.min(message_limit.saturating_sub(request.relative_path.len() + 512));
    if limit == 0 {
        return Err(Status::resource_exhausted(
            "configured message limit cannot contain file window",
        ));
    }
    runtime
        .execution_file_content_projected(
            &id,
            request.relative_path,
            request.offset,
            limit,
            |content| super::codec::projection(content, message_limit),
            |content, allowance| super::codec::encode(content, allowance, message_limit),
        )
        .await
        .map_err(read_failure)?
        .ok_or_else(|| Status::not_found("registered execution output not found"))
}

/* === Private Functions === */
fn read_failure(error: dokime_app::Error) -> Status {
    let status = match &error {
        dokime_app::Error::Harmos(harmos::Error::InvalidInput {
            field: "execution files cursor",
            ..
        }) => Status::aborted("file listing changed; restart pagination"),
        dokime_app::Error::Harmos(harmos::Error::InvalidInput {
            field: "execution files after path",
            ..
        }) => Status::invalid_argument("after path not found"),
        dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. })
        | dokime_app::Error::ExecutionFileTooLarge => {
            Status::resource_exhausted("execution output exceeds its original budget")
        }
        _ => Status::internal("cannot read execution output"),
    };
    super::super::error::detailed(status, &error)
}

/* === Tests === */
