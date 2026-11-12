/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Preview Selection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.planning.preview_selection";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlanningPreviewSelectionRequest, v1::PreviewSelectionReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PlanningPreviewSelectionRequest) -> Result<v1::PreviewSelectionReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let plan = request.plan.ok_or_else(|| Error::Invalid("Preview Selection: Supply a Plan".into()))?;
        super::planning::resolve(context, &plan).await
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
