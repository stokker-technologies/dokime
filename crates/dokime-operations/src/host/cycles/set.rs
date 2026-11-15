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

// Local Imports
use crate::host::ClientSettings;
use crate::host::RunAction;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CycleStatus;
use dokime_protocol::host::CyclesGetRequest;
use dokime_protocol::host::CyclesSetRequest;
use dokime_protocol::host::CyclesSetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cycles/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CyclesSetRequest, CyclesSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: CyclesSetRequest) -> Result<CyclesSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let _guard = context.administration.lock().await;
        let store = context.host.store();
        if let Some(response) = store.cycle_request(&request)? {
            return Ok(response);
        }
        let stored = store.client_cycle(&request.name)?;
        let previous = crate::host::cycles::get::handle(
            context,
            CyclesGetRequest {
                name: request.name.clone(),
                ..Default::default()
            },
        )
        .await?;
        if previous.revision != request.expected_revision {
            return Err(Error::Conflict("Set Cycle: Get Its Current Revision".into()));
        }
        let value = request.value.clone().ok_or_else(|| Error::Invalid("Set Cycle: Supply the Whole Writable Value".into()))?;
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
                context
                    .host
                    .control_execution(&row.execution.identifier, if value.paused { RunAction::Pause } else { RunAction::Resume })?;
            }
        }
        cycle.status = if value.paused { CycleStatus::Paused } else { CycleStatus::Ready } as i32;
        let response = CyclesSetResponse {
            revision: crate::host::paging::bytes(&value),
            value: Some(value),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        };
        store.save_cycle_request(&request, stored.as_ref(), &cycle, &response)?;
        context.host.signal_change();
        Ok(response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CyclesSetRequest, CyclesSetResponse, context, "dokime.host.v1", "Cycles", "Set");
