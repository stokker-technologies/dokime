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

// Third Party Imports
use harmos_operations::storage::{Info, InfoRequest};
use harmos_protocol::model::Position;

// Local Imports
use crate::{Error, Resources, Result};
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

pub async fn handle(resources: &Resources, request: BackendEnvironmentRequest) -> Result<BackendEnvironmentResponse> {
    resources.check_action_scope(&resources.execution, ID)?;
    let mut context = resources.context.as_ref().clone();
    context.caller = Some(resources.clone());
    let info = context.storage::<Info>(InfoRequest {}).await?;
    let resolved_at = info.saved.map(Position::from);
    if request.at.as_ref().is_some_and(|at| Some(at) != resolved_at.as_ref()) {
        return Err(Error::Precondition(format!("{ID}: Its Storage Position Is Unavailable; Resynchronize")));
    }
    Ok(BackendEnvironmentResponse {
        backend_pid: std::process::id(),
        resolved_at,
        in_step: !resources.step.is_empty(),
    })
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(BackendEnvironmentRequest, BackendEnvironmentResponse, resources, "dokime.host.v1", "Backend", "Environment");
