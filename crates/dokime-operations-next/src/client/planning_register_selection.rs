/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Register Selection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.planning.register_selection";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlanningRegisterSelectionRequest, v1::PreviewSelectionReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::PlanningRegisterSelectionRequest) -> Result<v1::PreviewSelectionReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.idempotency_key.trim().is_empty() { return Err(Error::Invalid("Register Selection: Supply a Stable Nonempty Idempotency Key".into())); }
        let plan = request.plan.ok_or_else(|| Error::Invalid("Register Selection: Supply a Plan".into()))?;
        if !plan.case_ids.is_empty() { return Err(Error::Invalid("Register Selection: Select Templates Instead of Already Registered Cases".into())); }
        let bytes = plan.encode_to_vec();
        if let Some(reply) = context.host.store().registration(&request.idempotency_key, &bytes).map_err(|error| Error::Conflict(error.to_string()))? { return Ok(reply); }
        let reply = super::planning::resolve(context, &plan).await?;
        Ok(context.host.store().register_cases(Some(&request.idempotency_key), &bytes, &plan.cycle_id, reply)?)
    }).await
}

/* === Private Functions === */

/* === Tests === */
