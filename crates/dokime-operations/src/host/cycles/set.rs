/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycles Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CyclesSetRequest, dokime_protocol::host::CyclesSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::CyclesSetRequest,
) -> Result<dokime_protocol::host::CyclesSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        let previous = crate::host::cycles::get::handle(
            context,
            dokime_protocol::host::CyclesGetRequest {
                name: request.name.clone(),
                ..Default::default()
            },
        )
        .await?;
        if previous.revision != request.expected_revision {
            return Err(Error::Conflict("Set Cycle: Get Its Current Revision".into()));
        }
        let value = request.value.ok_or_else(|| Error::Invalid("Set Cycle: Supply the Whole Writable Value".into()))?;
        let mut cycle = previous.cycle.unwrap();
        if cycle.sealed {
            return Err(Error::Precondition("Set Cycle: Select an Unsealed Cycle".into()));
        }
        if value.label.trim().is_empty() {
            return Err(Error::Invalid("Set Cycle: Supply a Nonempty Label".into()));
        }
        cycle.metadata.get_or_insert_default().label = value.label.clone();
        cycle.archived = value.archived;
        cycle.continue_automatically = value.continue_automatically;
        for row in crate::host::execution_state::snapshots(context, &request.name)? {
            let controls = context.host.controls(&row.execution.identifier)?;
            if controls.can_cancel && controls.paused != value.paused {
                context.host.control_execution(
                    &row.execution.identifier,
                    if value.paused { crate::host::RunAction::Pause } else { crate::host::RunAction::Resume },
                )?;
            }
        }
        cycle.status =
            if value.paused { dokime_protocol::host::CycleStatus::Paused } else { dokime_protocol::host::CycleStatus::Ready } as i32;
        context.host.store().save_cycle(&cycle)?;
        context.host.signal_change();
        Ok(dokime_protocol::host::CyclesSetResponse {
            revision: crate::host::paging::bytes(&value),
            value: Some(value),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
