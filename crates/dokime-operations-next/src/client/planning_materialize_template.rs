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

use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.planning.materialize_template";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlanningMaterializeTemplateRequest, v1::PreviewSelectionReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::PlanningMaterializeTemplateRequest) -> Result<v1::PreviewSelectionReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let selection = request.selection.ok_or_else(|| Error::Invalid("Materialize Template: Supply a Selection".into()))?;
        let plan = v1::SelectionPlan { selections: vec![selection], ..Default::default() };
        let reply = super::planning::resolve(context, &plan).await?;
        Ok(context.host.store().register_cases(None, &plan.encode_to_vec(), &plan.cycle_id, reply)?)
    }).await
}

/* === Private Functions === */

/* === Tests === */
