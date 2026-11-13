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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Cycles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CyclesGetRequest, v1::CyclesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::CyclesGetRequest) -> Result<v1::CyclesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let cycle = super::execution_state::cycle(context, &request.name)?;
        let value = v1::CycleWritable {
            label: cycle.metadata.as_ref().map_or(String::new(), |metadata| metadata.label.clone()),
            paused: cycle.status == v1::CycleStatus::Paused as i32,
            archived: cycle.archived,
            continue_automatically: cycle.continue_automatically,
        };
        let position = context.host.store().evidence_position(request.at.as_ref())?;
        Ok(v1::CyclesGetResponse {
            revision: super::paging::bytes(&value),
            value: Some(value),
            cycle: Some(cycle),
            resolved_at: Some(position),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
