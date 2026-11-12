/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Execution Get Cycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.get_cycle";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::ExecutionGetCycleRequest, v1::CycleSummary>(ID, settings.read_timeout) }
pub async fn handle(context: &Context, request: v1::ExecutionGetCycleRequest) -> Result<v1::CycleSummary> {
    super::complete(ID, context.settings.read_timeout, async {
        super::execution_state::cycle(context, &request.cycle_id)
    }).await
}

/* === Private Functions === */

/* === Tests === */
