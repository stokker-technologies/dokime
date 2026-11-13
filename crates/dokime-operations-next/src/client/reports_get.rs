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

pub const ID: &str = "dokime.v1.Reports/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ReportsGetRequest, v1::ReportsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ReportsGetRequest) -> Result<v1::ReportsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
if let Some((_,reference))=context.host.temporary_report(&request.name)? {return Ok(v1::ReportsGetResponse {file:Some(reference),report:None,resolved_at:request.at});}
let execution=match request.subject.as_ref().and_then(|subject|subject.kind.as_ref()){Some(v1::report_subject::Kind::Execution(execution))=>execution,_=>return Err(Error::Invalid("Get Report: Select Its Published Execution".into()))};
let report=context.host.store().reports(execution)?.into_iter().find(|report|request.name.is_empty() || report.identifier==request.name).ok_or_else(||Error::NotFound("Get Report: Publish the Selected Report First".into()))?;
let cut=context.host.store().metadata(execution)?;
let mut file=tempfile::tempfile().map_err(|error|Error::Invalid(format!("Get Report: {error}")))?;
use std::io::Write;file.write_all(&report.html).map_err(|error|Error::Invalid(format!("Get Report: {error}")))?;
let (_,length,digest)=super::content_transfer::temporary(file.try_clone().map_err(|error|Error::Invalid(format!("Get Report: {error}")))?)?;
let digest=(0..digest.len()).step_by(2).map(|index|u8::from_str_radix(&digest[index..index+2],16).unwrap()).collect::<Vec<_>>();
let reference=harmos_protocol::storage::FileReference {metadata:Some(harmos_protocol::storage::FileMetadata {id:report.identifier.clone(),scope:Some(execution.clone()),media_type:"text/html".into(),expected_length:Some(length),expected_sha256:Some(digest.clone())}),length,sha256:digest,..Default::default()};
context.host.retain_report(file,reference.clone())?;
Ok(v1::ReportsGetResponse {report:Some(v1::ReportMetadata {test_cycle_id:cut.execution.cycle,test_execution_id:Some(execution.clone()),identifier:report.identifier,metadata:report.document.and_then(|document|document.metadata),origin:v1::ReportOrigin::Sealed as i32,..Default::default()}),file:Some(reference),resolved_at:request.at})
    }).await
}

/* === Private Functions === */

/* === Tests === */
