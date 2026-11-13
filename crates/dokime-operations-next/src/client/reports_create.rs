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

pub const ID: &str = "dokime.v1.Reports/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::ReportsCreateRequest, v1::ReportsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::ReportsCreateRequest) -> Result<v1::ReportsCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let subject=request.subject.as_ref().ok_or_else(||Error::Invalid("Create Report: Select Its Execution or Cycle".into()))?;
let (mut file,metadata)=context.host.render_report(subject,request.at.as_ref())?;
let (retained,length,digest)=super::content_transfer::temporary(file.try_clone().map_err(|error|Error::Invalid(format!("Create Report: {error}")))?)?;
let digest=(0..digest.len()).step_by(2).map(|index|u8::from_str_radix(&digest[index..index+2],16).unwrap()).collect::<Vec<_>>();
let name=format!("prepared-report/{}",request.request_id);
let reference=harmos_protocol::storage::FileReference {metadata:Some(harmos_protocol::storage::FileMetadata {id:name.clone(),scope:metadata.test_execution_id.clone().or(Some(metadata.test_cycle_id.clone())),media_type:"text/html".into(),expected_length:Some(length),expected_sha256:Some(digest.clone())}),length,sha256:digest,..Default::default()};
if let Some(execution)=&metadata.test_execution_id {
 use std::io::{Read,Seek};file.rewind().map_err(|error|Error::Invalid(format!("Create Report: {error}")))?;
 let mut html=Vec::new();file.read_to_end(&mut html).map_err(|error|Error::Invalid(format!("Create Report: {error}")))?;
 let scope=v1::DocumentScope {cycle_id:metadata.test_cycle_id.clone(),execution_id:execution.clone()};
 let (document,cut)=context.host.current_document(&scope,&request.document_id)?;
 context.host.store().save_administration(&name,&dokime_storage::Report {execution:execution.clone(),identifier:name.clone(),execution_seal:cut.seal,document:Some(document),html,seal:reference.sha256.clone()})?;
}
context.host.retain_report(retained,reference.clone())?;
let position=context.host.store().evidence_position(None)?;
Ok(v1::ReportsCreateResponse {report:Some(metadata),file:Some(reference),resolved_at:Some(h::ReadCut {positions:vec![position],revisions:Vec::new()}),receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
