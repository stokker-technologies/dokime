/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reports Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::ReportsCreateRequest, dokime_protocol::host::ReportsCreateResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::ReportsCreateRequest,
) -> Result<dokime_protocol::host::ReportsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let subject = request.subject.as_ref().ok_or_else(|| Error::Invalid("Create Report: Select Its Execution or Cycle".into()))?;
        let (mut file, metadata) = context.host.render_report(subject, &request.document_id, request.at.as_ref())?;
        let (retained, length, digest) = crate::host::content::transfer::temporary(
            file.try_clone().map_err(|error| Error::Invalid(format!("Create Report: {error}")))?,
        )?;
        let digest =
            (0..digest.len()).step_by(2).map(|index| u8::from_str_radix(&digest[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
        let name = format!("prepared-report/{}", request.request_id);
        let reference = harmos_protocol::model::FileReference {
            metadata: Some(harmos_protocol::model::FileMetadata {
                id: name.clone(),
                scope: metadata.test_execution_id.clone().or(Some(metadata.test_cycle_id.clone())),
                media_type: "text/html".into(),
                expected_length: Some(length),
                expected_sha256: Some(digest.clone()),
            }),
            length,
            sha256: digest,
            ..Default::default()
        };
        if let Some(execution) = &metadata.test_execution_id {
            use std::io::{Read, Seek};
            file.rewind().map_err(|error| Error::Invalid(format!("Create Report: {error}")))?;
            let mut html = Vec::new();
            file.read_to_end(&mut html).map_err(|error| Error::Invalid(format!("Create Report: {error}")))?;
            let scope = dokime_protocol::host::DocumentScope {
                cycle_id: metadata.test_cycle_id.clone(),
                execution_id: execution.clone(),
            };
            let (document, cut) = context.host.current_document(&scope, &request.document_id)?;
            context.host.store().save_administration(
                &name,
                &dokime_storage::Report {
                    execution: execution.clone(),
                    identifier: name.clone(),
                    execution_seal: cut.seal,
                    document: Some(document),
                    html,
                    seal: reference.sha256.clone(),
                },
            )?;
        }
        context.host.retain_report(retained, reference.clone())?;
        let position = context.host.store().evidence_position(None)?;
        Ok(dokime_protocol::host::ReportsCreateResponse {
            report: Some(metadata),
            file: Some(reference),
            resolved_at: Some(h::ReadCut {
                positions: vec![position],
                revisions: Vec::new(),
            }),
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
