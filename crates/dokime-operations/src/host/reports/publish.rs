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

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportOrigin;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::ReportsPublishResponse;
use dokime_protocol::host::report_subject::Kind;
use dokime_storage::Report;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ReportsPublishRequest, ReportsPublishResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: ReportsPublishRequest) -> Result<ReportsPublishResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
let prepared:Report=context.host.store().administration(&request.name)?.ok_or_else(||Error::NotFound("Publish Report: Create Its Prepared Report First".into()))?;
if request.expected_revision!=prepared.seal {return Err(Error::Conflict("Publish Report: Use the Prepared File Digest as Its Revision".into()));}
if !matches!(request.subject.as_ref().and_then(|subject|subject.kind.as_ref()),Some(Kind::Execution(execution)) if execution==&prepared.execution){return Err(Error::Invalid("Publish Report: Keep Its Prepared Execution Subject".into()));}
let report=context.host.publish_report(prepared)?;
let cut=context.host.store().metadata(&report.execution)?;
Ok(ReportsPublishResponse {report:Some(ReportMetadata {test_cycle_id:cut.execution.cycle,test_execution_id:Some(report.execution.clone()),identifier:report.identifier.clone(),metadata:report.document.and_then(|document|document.metadata),origin:ReportOrigin::Sealed as i32,..Default::default()}),seal:report.seal,receipt:Some(crate::host::views::receipt(context,&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
