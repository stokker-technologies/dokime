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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Reports/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::ReportsPublishRequest, dokime_protocol::client::ReportsPublishResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::ReportsPublishRequest,
) -> Result<dokime_protocol::client::ReportsPublishResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let prepared:dokime_storage::Report=context.host.store().administration(&request.name)?.ok_or_else(||Error::NotFound("Publish Report: Create Its Prepared Report First".into()))?;
if request.expected_revision!=prepared.seal {return Err(Error::Conflict("Publish Report: Use the Prepared File Digest as Its Revision".into()));}
if !matches!(request.subject.as_ref().and_then(|subject|subject.kind.as_ref()),Some(dokime_protocol::client::report_subject::Kind::Execution(execution)) if execution==&prepared.execution){return Err(Error::Invalid("Publish Report: Keep Its Prepared Execution Subject".into()));}
let report=context.host.publish_report(prepared)?;
let cut=context.host.store().metadata(&report.execution)?;
Ok(dokime_protocol::client::ReportsPublishResponse {report:Some(dokime_protocol::client::ReportMetadata {test_cycle_id:cut.execution.cycle,test_execution_id:Some(report.execution.clone()),identifier:report.identifier.clone(),metadata:report.document.and_then(|document|document.metadata),origin:dokime_protocol::client::ReportOrigin::Sealed as i32,..Default::default()}),seal:report.seal,receipt:Some(super::views::receipt(context,&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
