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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CyclesGetRequest, dokime_protocol::host::CyclesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::CyclesGetRequest,
) -> Result<dokime_protocol::host::CyclesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let cycle = crate::host::execution_state::cycle(context, &request.name)?;
        let value = dokime_protocol::host::CycleWritable {
            label: cycle.metadata.as_ref().map_or(String::new(), |metadata| metadata.label.clone()),
            paused: cycle.status == dokime_protocol::host::CycleStatus::Paused as i32,
            archived: cycle.archived,
            continue_automatically: cycle.continue_automatically,
        };
        let position = context.host.store().evidence_position(request.at.as_ref())?;
        Ok(dokime_protocol::host::CyclesGetResponse {
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
