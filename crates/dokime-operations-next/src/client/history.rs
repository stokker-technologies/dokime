/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// History

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use std::{collections::BTreeMap, num::NonZeroUsize};
use prost::Message;
use sha2::{Digest,Sha256};
use crate::{Error,Result,client::Context};
use dokime_protocol::v1;
use dokime_storage::{ExecutionCut,ExecutionState};

// Standard Library Imports

// Third Party Imports

// Local Imports


/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn hex(bytes: &[u8]) -> String { bytes.iter().map(|byte| format!("{byte:02x}")).collect() }


pub(super) fn position(context: &Context, sequence: u64, selection: &str) -> v1::StorageCursor {
    v1::StorageCursor { history: Some(v1::StoreHistory { store: context.host.identity().backend_id,epoch: format!("v1/{selection}") }),sequence }
}
pub(super) fn checkpoint(context: &Context, execution: &str) -> Result<v1::ExecutionCheckpoint> {
    super::execution_state::read(context,execution)?;
    let cut = context.host.store().cut(execution)?;
    if cut.state != ExecutionState::Sealed { return Err(Error::NotFound("Get Execution Checkpoint: Wait for the Execution's Existing Saved Seal".into())); }
    let (sequence,_) = context.host.store().sealed_cycle(execution)?;
    Ok(v1::ExecutionCheckpoint { execution_identifier: execution.into(),position: Some(position(context,sequence,"seals")),version: cut.seal,evidence_through: Some(position(context,cut.frontier,"evidence")) })
}
pub(super) fn selected(context: &Context, supplied: Option<&v1::ExecutionCheckpoint>) -> Result<(v1::ExecutionCheckpoint,ExecutionCut)> {
    let supplied = supplied.ok_or_else(|| Error::Invalid("Read Checkpoint: Supply the Token Returned by GetExecutionCheckpoint".into()))?;
    let current = checkpoint(context,&supplied.execution_identifier)?;
    if supplied != &current { return Err(Error::Conflict("Read Checkpoint: Inspect the Current Saved Seal and Use Its Exact Token".into())); }
    let cut = context.host.store().cut(&supplied.execution_identifier)?;
    Ok((current,cut))
}
pub(super) fn bounded<T: Message>(context: &Context, reply: T) -> Result<T> {
    if reply.encoded_len() > context.settings.message_bytes { return Err(Error::Invalid("Read History: Reduce the Page or Select Bounded Evidence Sections".into())); }
    Ok(reply)
}
pub(super) fn continuation(checkpoint: &v1::ExecutionCheckpoint, selection: &str, position: u64) -> String {
    let mut digest = Sha256::new(); digest.update(checkpoint.encode_to_vec()); digest.update(selection.as_bytes());
    format!("{}/{position}",hex(&digest.finalize()))
}
pub(super) fn after(checkpoint: &v1::ExecutionCheckpoint, selection: &str, supplied: &str) -> Result<u64> {
    if supplied.is_empty() { return Ok(0); }
    let position = supplied.rsplit_once('/').and_then(|(_,position)| position.parse::<u64>().ok()).ok_or_else(|| Error::Invalid("Read History: Use the Returned Continuation Token".into()))?;
    if position == 0 || continuation(checkpoint,selection,position) != supplied { return Err(Error::Conflict("Read History: Keep the Same Checkpoint and Selection While Paging".into())); }
    Ok(position)
}
pub(super) fn metadata(context: &Context, cut: &ExecutionCut) -> Result<v1::HistoricalExecutionMetadata> {
    let (case_id,label,_) = context.host.store().execution_presentation(&cut.execution.identifier)?;
    let template_id = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) { Some(v1::origin::Kind::PluginRun(run)) => Some(run.template.clone()),_ => None };
    Ok(v1::HistoricalExecutionMetadata { execution_id: cut.execution.identifier.clone(),cycle_id: Some(cut.execution.cycle.clone()),case_id,title: label,status: match cut.state { ExecutionState::Open => v1::ExecutionState::Open,ExecutionState::Closed => v1::ExecutionState::Closed,ExecutionState::Sealed => v1::ExecutionState::Sealed } as i32,template_id,origin: cut.execution.origin.clone(),outcome: cut.execution.outcome.clone(),..Default::default() })
}
pub(super) fn document(context: &Context, checkpoint: &v1::ExecutionCheckpoint, cut: &ExecutionCut) -> Result<v1::HistoricalTestExecution> {
    let snapshot = super::execution_state::evidence(context,&cut.execution.identifier)?;
    let catalog = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice())?;
    let mut documents = Vec::new();
    let mut template = None;
    let mut suite = None;
    let mut suite_id = String::new();
    let mut definitions = Vec::new();
    let mut parameters = Vec::new();
    if let Some(v1::origin::Kind::PluginRun(run)) = cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
        for section in &catalog.sections {
            if section.r#type != "dokime.v1.Suites" { continue; }
            for entry in v1::Suites::decode(section.payload.as_slice())?.suites {
                for value in entry.templates { if value.metadata.as_ref().is_some_and(|metadata| metadata.identifier == run.template) { suite = entry.metadata.clone(); suite_id = suite.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default(); template = value.metadata; definitions = value.parameters; documents = value.documents; } }
            }
        }
        if let Some(case) = &run.test_case { let mut values: Vec<_> = case.parameters.iter().collect(); values.sort_by_key(|(name,_)| *name); parameters = values.into_iter().map(|(name,value)| v1::MaterializedParameter { name: name.clone(),value: Some(value.clone()),source: v1::ParameterSource::Definition as i32,..Default::default() }).collect(); }
    }
    for report in context.host.store().reports(&cut.execution.identifier)? { if let Some(document) = report.document && !documents.contains(&document) { documents.push(document); } }
    let mut steps = Vec::new();
    if let Some(origin) = &cut.execution.origin && let Ok(case) = super::execution_state::case(origin) {
        for step in &case.steps {
            let identifier = step.metadata.as_ref().unwrap().identifier.clone();
            let started_at = snapshot.evidence.iter().find(|entry| entry.step == identifier && entry.kind == "step:start").map(|entry| super::projection::timestamp(entry.receipt_ns));
            let finished_at = snapshot.evidence.iter().rev().find(|entry| entry.step == identifier && entry.kind == "step:end").map(|entry| super::projection::timestamp(entry.receipt_ns));
            let status = if snapshot.evidence.iter().any(|entry| entry.step == identifier && entry.kind == "step:failure") { v1::StepStatus::Failed } else if finished_at.is_some() { v1::StepStatus::Passed } else if started_at.is_some() { v1::StepStatus::Cancelled } else { v1::StepStatus::Skipped };
            steps.push(v1::HistoricalStep { step_id: identifier,metadata: step.metadata.clone(),status: status as i32,started_at,finished_at,..Default::default() });
        }
    }
    let _ = checkpoint;
    Ok(v1::HistoricalTestExecution { execution_id: cut.execution.identifier.clone(),status: v1::ExecutionState::Sealed as i32,scope: Some(super::projection::scope(&snapshot)),origin: cut.execution.origin.clone(),outcome: cut.execution.outcome.clone(),schemas: Some(super::projection::schemas(&snapshot)?),suite,template,suite_id,parameter_definitions: definitions,input_parameters: parameters,document_definitions: documents,steps,..Default::default() })
}
pub(super) fn artifacts(context: &Context, cut: &ExecutionCut) -> Result<Vec<v1::ArtifactMetadata>> {
    let mut transfers: BTreeMap<String,(v1::ArtifactMetadata,Sha256,u64)> = BTreeMap::new();
    let mut after = 0;
    loop {
        let page = context.host.store().evidence_page(&cut.execution.identifier,after,cut.frontier,NonZeroUsize::new(context.settings.page_items).unwrap())?;
        if page.is_empty() { break; }
        after = page.last().unwrap().position;
        for entry in page {
            if entry.kind != "artifact" { continue; }
            let chunk = v1::PublishArtifact::decode(entry.payload.as_slice())?;
            let value = transfers.entry(chunk.transfer_id.clone()).or_insert_with(|| (v1::ArtifactMetadata { artifact_id: format!("{}/{}",cut.execution.identifier,chunk.transfer_id),upload_id: chunk.transfer_id.clone(),filename: chunk.name.clone(),content_type: Some(chunk.mime.clone()),scope: Some(v1::ArtifactScope { execution_id: Some(cut.execution.identifier.clone()),cycle_id: Some(cut.execution.cycle.clone()),..Default::default() }),created_at: Some(super::projection::timestamp(entry.receipt_ns)),..Default::default() },Sha256::new(),0));
            if chunk.sequence != value.2 || value.0.filename != chunk.name || value.0.content_type.as_deref() != Some(chunk.mime.as_str()) { return Err(Error::Conflict("Read Checkpoint Artifact: Restore the Valid Ordered Artifact Evidence".into())); }
            value.2 += 1;
            value.0.size_bytes += chunk.data.len() as u64;
            value.1.update(&chunk.data);
            if chunk.last { value.0.state = v1::ArtifactState::Complete as i32; value.0.sha256 = hex(&value.1.clone().finalize()); }
            if chunk.aborted { value.0.state = v1::ArtifactState::Cancelled as i32; }
        }
    }
    Ok(transfers.into_values().map(|(metadata,_,_)| metadata).collect())
}
pub(super) fn artifact(context: &Context, cut: &ExecutionCut, id: &str) -> Result<v1::ArtifactMetadata> {
    let metadata = artifacts(context,cut)?.into_iter().find(|metadata| metadata.artifact_id == id).ok_or_else(|| Error::NotFound("Read Checkpoint Artifact: Choose an Artifact from GetCheckpointEvidence".into()))?;
    if metadata.state != v1::ArtifactState::Complete as i32 { return Err(Error::Conflict("Read Checkpoint Artifact: Select a Completed Artifact".into())); }
    Ok(metadata)
}

/* === Private Functions === */

/* === Tests === */
