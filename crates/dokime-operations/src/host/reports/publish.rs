/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Publish

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ReportsPublishRequest, dokime_protocol::host::ReportsPublishResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ReportsPublishRequest,
) -> Result<dokime_protocol::host::ReportsPublishResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
let prepared:dokime_storage::Report=context.host.store().administration(&request.name)?.ok_or_else(||Error::NotFound("Publish Report: Create Its Prepared Report First".into()))?;
if request.expected_revision!=prepared.seal {return Err(Error::Conflict("Publish Report: Use the Prepared File Digest as Its Revision".into()));}
if !matches!(request.subject.as_ref().and_then(|subject|subject.kind.as_ref()),Some(dokime_protocol::host::report_subject::Kind::Execution(execution)) if execution==&prepared.execution){return Err(Error::Invalid("Publish Report: Keep Its Prepared Execution Subject".into()));}
let report=context.host.publish_report(prepared)?;
let cut=context.host.store().metadata(&report.execution)?;
Ok(dokime_protocol::host::ReportsPublishResponse {report:Some(dokime_protocol::host::ReportMetadata {test_cycle_id:cut.execution.cycle,test_execution_id:Some(report.execution.clone()),identifier:report.identifier.clone(),metadata:report.document.and_then(|document|document.metadata),origin:dokime_protocol::host::ReportOrigin::Sealed as i32,..Default::default()}),seal:report.seal,receipt:Some(crate::host::views::receipt(context,&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
