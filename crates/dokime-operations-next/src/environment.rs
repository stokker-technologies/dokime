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

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::context::{BackendEnvironmentRequest, BackendEnvironmentResponse};

/* === Constants === */

pub const ID: &str = "dokime.context.v1.Backend/Environment";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<harmos_protocol::v1::OperationContract> {
    use prost::Message;
    crate::declaration::<BackendEnvironmentRequest, BackendEnvironmentResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, _: BackendEnvironmentRequest) -> Result<BackendEnvironmentResponse> {
    resources.check_action_scope(&resources.execution, ID)?;
    Ok(BackendEnvironmentResponse {
        backend_pid: std::process::id(),
        in_step: !resources.step.is_empty(),
    })
}

/* === Private Functions === */

/* === Tests === */
