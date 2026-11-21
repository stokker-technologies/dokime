/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Invocation-Scoped Host Environment

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process;

// Third Party Imports
use harmos_operations::storage::{Info, InfoRequest};
use harmos_protocol::model::Position;

// Local Imports
use crate::host::Context;
use crate::{Error, Result};
use dokime_protocol::host::BackendEnvironmentRequest;
use dokime_protocol::host::BackendEnvironmentResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Backend/Environment";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: BackendEnvironmentRequest) -> Result<BackendEnvironmentResponse> {
    let info = context.storage::<Info>(InfoRequest {}).await?;
    let resolved_at = info.saved.map(Position::from);
    if request.at.as_ref().is_some_and(|at| Some(at) != resolved_at.as_ref()) {
        return Err(Error::Precondition(format!("{ID}: Its Storage Position Is Unavailable; Resynchronize")));
    }
    Ok(BackendEnvironmentResponse {
        backend_pid: process::id(),
        resolved_at,
        in_step: context.caller.as_ref().is_some_and(|caller| !caller.step.is_empty()),
    })
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(BackendEnvironmentRequest, BackendEnvironmentResponse, plugin, "dokime.host.v1", "Backend", "Environment");
