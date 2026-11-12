/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.create_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ExecutionCreateCycleRequest, v1::CreateCycleReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ExecutionCreateCycleRequest) -> Result<v1::CreateCycleReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.metadata.is_some() || request.context.is_some() {
            return Err(Error::Invalid("CreateCycle Metadata and MutationContext Are Not Yet Served by the New Host".into()));
        }
        if request.origins.is_empty() || request.origins.len() > context.settings.max_cases {
            return Err(Error::Invalid("CreateCycle Requires Frozen Origins Within the Case Bound".into()));
        }
        if !request.continue_automatically {
            return Err(Error::Invalid("CreateCycle Requires Automatic Execution for Explicit Origins".into()));
        }
        let cycle = context.host.start(request.origins).await?;
        Ok(v1::CreateCycleReply {
            cycle_id: cycle.identifier,
            execution_ids: cycle.executions,
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
