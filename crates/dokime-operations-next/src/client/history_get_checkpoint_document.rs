/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Get Checkpoint Document

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.get_checkpoint_document";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryGetCheckpointDocumentRequest, v1::CheckpointDocument>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryGetCheckpointDocumentRequest) -> Result<v1::CheckpointDocument> {
    super::complete(ID, context.settings.read_timeout, async {
        let (checkpoint, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
        let document = super::history::document(context, &checkpoint, &cut)?;
        let (_, cycle) = context.host.store().sealed_cycle(&cut.execution.identifier)?;
        super::history::bounded(
            context,
            v1::CheckpointDocument {
                checkpoint: Some(checkpoint),
                document: Some(v1::HistoricalExecutionDocument {
                    cycle: Some(v1::HistoricalExecutionCycle {
                        cycle_id: cycle.cycle_id,
                        metadata: cycle.metadata,
                    }),
                    execution: Some(document),
                }),
                acquisition_finished: true,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
