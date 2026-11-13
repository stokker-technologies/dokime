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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Cases/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::CasesCreateRequest, v1::CasesCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::CasesCreateRequest) -> Result<v1::CasesCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if !request.validate_only && request.request_id.trim().is_empty() {
            return Err(Error::Invalid("Register Selection: Supply a Stable Nonempty Idempotency Key".into()));
        }
        let plan = request.plan.ok_or_else(|| Error::Invalid("Create Cases: Supply a Plan".into()))?;
        if request.validate_only {
            return super::planning::resolve(context, &plan).await;
        }
        if !plan.case_ids.is_empty() {
            return Err(Error::Invalid("Create Cases: Select Templates Instead of Already Registered Cases".into()));
        }
        let bytes = super::paging::bytes(&plan);
        if let Some(reply) =
            context.host.store().registration(&request.request_id, &bytes).map_err(|error| Error::Precondition(error.to_string()))?
        {
            return Ok(reply);
        }
        let reply = super::planning::resolve(context, &plan).await?;
        Ok(context.host.store().register_cases(Some(&request.request_id), &bytes, &plan.cycle_id, reply, context.settings.message_bytes)?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
