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

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CasesCreateResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Cases/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<CasesCreateRequest, CasesCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: CasesCreateRequest) -> Result<CasesCreateResponse> {
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
        if let Some(reply) = super::storage::registration(context, &request.request_id, &bytes).await? {
            return Ok(reply);
        }
        let reply = crate::host::planning::resolve(context, &plan).await?;
        let reply = super::storage::register(context, &request.request_id, &bytes, &plan.cycle_id, reply).await?;
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(CasesCreateRequest, CasesCreateResponse, context, "dokime.host.v1", "Cases", "Create");
