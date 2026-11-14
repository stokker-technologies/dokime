/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cases Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::CasesCreateRequest, dokime_protocol::host::CasesCreateResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::CasesCreateRequest,
) -> Result<dokime_protocol::host::CasesCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if !request.validate_only && request.request_id.trim().is_empty() {
            return Err(Error::Invalid("Register Selection: Supply a Stable Nonempty Idempotency Key".into()));
        }
        let plan = request.plan.ok_or_else(|| Error::Invalid("Create Cases: Supply a Plan".into()))?;
        if request.validate_only {
            return crate::host::planning::resolve(context, &plan).await;
        }
        if !plan.case_ids.is_empty() {
            return Err(Error::Invalid("Create Cases: Select Templates Instead of Already Registered Cases".into()));
        }
        let bytes = crate::host::paging::bytes(&plan);
        if let Some(reply) =
            context.host.store().registration(&request.request_id, &bytes).map_err(|error| Error::Precondition(error.to_string()))?
        {
            return Ok(reply);
        }
        let reply = crate::host::planning::resolve(context, &plan).await?;
        Ok(context.host.store().register_cases(Some(&request.request_id), &bytes, &plan.cycle_id, reply, context.settings.message_bytes)?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
