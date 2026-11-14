/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::CycleStatus;
use dokime_protocol::host::CycleWritable;
use dokime_protocol::host::CyclesGetRequest;
use dokime_protocol::host::CyclesGetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CyclesGetRequest, CyclesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: CyclesGetRequest) -> Result<CyclesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let cycle = crate::host::execution_state::cycle(context, &request.name)?;
        let value = CycleWritable {
            label: cycle.metadata.as_ref().map_or(String::new(), |metadata| metadata.label.clone()),
            paused: cycle.status == CycleStatus::Paused as i32,
            archived: cycle.archived,
            continue_automatically: cycle.continue_automatically,
        };
        let position = context.host.store().evidence_position(request.at.as_ref())?;
        Ok(CyclesGetResponse {
            revision: crate::host::paging::bytes(&value),
            value: Some(value),
            cycle: Some(cycle),
            resolved_at: Some(position),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
