/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Explicit Execution Report Publication

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const ID: &str = "dokime.reports.publish_execution_report";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ReportsPublishExecutionReportRequest, v1::ReportMetadata>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ReportsPublishExecutionReportRequest) -> Result<v1::ReportMetadata> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let document = request.document.ok_or_else(|| Error::Invalid("Publish Report: Supply the Selected Document".into()))?;
        let snapshot = context.host.store().cut(&request.execution_id)?;
        let report = context.host.publish_report(&request.execution_id, document)?;
        Ok(v1::ReportMetadata {
            test_cycle_id: snapshot.execution.cycle,
            test_execution_id: Some(request.execution_id),
            identifier: report.identifier,
            metadata: report.document.and_then(|document| document.metadata),
            origin: v1::ReportOrigin::Sealed as i32,
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
