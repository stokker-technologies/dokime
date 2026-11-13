/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Reports/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ReportsPublishRequest, v1::ReportsPublishResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ReportsPublishRequest) -> Result<v1::ReportsPublishResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let prepared:dokime_storage::Report=context.host.store().administration(&request.name)?.ok_or_else(||Error::NotFound("Publish Report: Create Its Prepared Report First".into()))?;
if request.expected_revision!=prepared.seal {return Err(Error::Conflict("Publish Report: Use the Prepared File Digest as Its Revision".into()));}
if !matches!(request.subject.as_ref().and_then(|subject|subject.kind.as_ref()),Some(v1::report_subject::Kind::Execution(execution)) if execution==&prepared.execution){return Err(Error::Invalid("Publish Report: Keep Its Prepared Execution Subject".into()));}
let report=context.host.store().publish_report(&prepared.execution,&prepared.execution_seal,prepared.document.ok_or_else(||Error::Invalid("Publish Report: Restore Its Frozen Document".into()))?,prepared.html)?;
let cut=context.host.store().metadata(&report.execution)?;
Ok(v1::ReportsPublishResponse {report:Some(v1::ReportMetadata {test_cycle_id:cut.execution.cycle,test_execution_id:Some(report.execution.clone()),identifier:report.identifier.clone(),metadata:report.document.and_then(|document|document.metadata),origin:v1::ReportOrigin::Sealed as i32,..Default::default()}),seal:report.seal,receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
