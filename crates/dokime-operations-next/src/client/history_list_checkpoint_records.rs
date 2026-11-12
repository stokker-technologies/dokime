/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History List Checkpoint Records

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.list_checkpoint_records";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryListCheckpointRecordsRequest, v1::CheckpointRecordPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryListCheckpointRecordsRequest) -> Result<v1::CheckpointRecordPage> {
    super::complete(ID, context.settings.read_timeout, async {
        let (checkpoint, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
        let mut after = super::history::after(&checkpoint, "records", &request.after)?;
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let mut snapshot = super::execution_state::read(context, &cut.execution.identifier)?;
        let mut records = Vec::new();
        let mut more = false;
        loop {
            let page = context.host.store().evidence_page(
                &cut.execution.identifier,
                after,
                cut.frontier,
                std::num::NonZeroUsize::new(context.settings.page_items).unwrap(),
            )?;
            if page.is_empty() {
                break;
            }
            after = page.last().unwrap().position;
            snapshot.evidence = page;
            for record in super::projection::records(&snapshot, &[], None)? {
                if records.len() == limit {
                    more = true;
                    break;
                }
                records.push(v1::SavedRecord {
                    epoch: context.host.identity().backend_id,
                    record: Some(record),
                });
            }
            if more {
                break;
            }
        }
        let next_after =
            more.then(|| super::history::continuation(&checkpoint, "records", records.last().unwrap().record.as_ref().unwrap().sequence));
        super::history::bounded(
            context,
            v1::CheckpointRecordPage {
                checkpoint: Some(checkpoint),
                records,
                next_after,
                ..Default::default()
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
