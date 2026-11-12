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

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.evidence.list_records";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::EvidenceListRecordsRequest, v1::RecordPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::EvidenceListRecordsRequest) -> Result<v1::RecordPage> {
    super::complete(ID, context.settings.read_timeout, async {
        if request.after.is_some() || request.through.is_some() {
            return Err(Error::Invalid("ListRecords Cursors Are Not Yet Served by the New Host".into()));
        }
        let selection = request.selection.ok_or_else(|| Error::Invalid("ListRecords Requires an Execution Selection".into()))?;
        if !selection.sources.is_empty() || !selection.severities.is_empty() {
            return Err(Error::Invalid("ListRecords Source and Severity Filters Are Not Yet Served by the New Host".into()));
        }
        let identifier = super::projection::execution_scope(selection.scope.as_ref())?;
        let snapshot = context.host.store().read(identifier)?;
        super::projection::check_scope(selection.scope.as_ref().unwrap(), &snapshot)?;
        let mut records = super::projection::records(
            &snapshot,
            &selection.record_types,
            selection.scope.as_ref().and_then(|scope| scope.step_id.as_deref()),
        )?;
        if request.latest != 0 {
            records = records.split_off(records.len().saturating_sub(request.latest as usize));
        }
        super::projection::bounded(records.len(), context.settings.page_items)?;
        Ok(v1::RecordPage {
            records,
            replay_complete: true,
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
