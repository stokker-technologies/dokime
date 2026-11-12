/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// History Delete Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error,Result,client::Context};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

pub const ID: &str = "dokime.history.delete_execution";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> { super::declaration::<v1::HistoryDeleteExecutionRequest,v1::DeleteExecutionResponse>(ID,settings.mutation_timeout) }
pub async fn handle(context: &Context,request: v1::HistoryDeleteExecutionRequest) -> Result<v1::DeleteExecutionResponse> { super::complete(ID,context.settings.mutation_timeout,async {
        if request.request_identifier.trim().is_empty() { return Err(Error::Invalid("Delete Execution: Supply a Stable Nonempty Request Identifier".into())); }
        let supplied = request.checkpoint.as_ref().ok_or_else(|| Error::Invalid("Delete Execution: Inspect the Sealed Checkpoint First".into()))?;
        let store = context.host.store();
        if !store.deletion(&request.request_identifier,&supplied.execution_identifier,&supplied.version)? {
            let (checkpoint,_) = super::history::selected(context,Some(supplied))?;
            store.delete_sealed_execution(&request.request_identifier,&checkpoint.execution_identifier,&checkpoint.version)?;
            context.host.signal_change();
        }
        Ok(v1::DeleteExecutionResponse { request_identifier: request.request_identifier,execution_identifier: supplied.execution_identifier.clone(),local_complete: true })
}).await }

/* === Private Functions === */

/* === Tests === */
