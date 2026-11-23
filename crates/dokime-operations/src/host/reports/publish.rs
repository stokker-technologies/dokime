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
use std::time::Instant;

// Third Party Imports
use harmos_operations::storage::{GetFileBytes, GetFileBytesRequest};
use harmos_protocol::model::FileReference;
use harmos_runtime::storage;

// Local Imports
use super::PreparedReport;
use crate::host::files::transfer;
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportMetadata;
use dokime_protocol::host::ReportOrigin;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::ReportsPublishResponse;
use dokime_protocol::host::report_subject::Kind;
use dokime_protocol::model::Document;
use dokime_storage::Report;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ReportsPublishRequest) -> Result<ReportsPublishResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let started = Instant::now();
        let (prepared, revision) = administration::message::<PreparedReport>(context, &request.name).await?;
        if revision.is_empty() {
            // Create Returns a Sealed Execution's Frozen Report, Which Sealing Already Published.
            if let Some(Kind::Execution(execution)) = request.subject.as_ref().and_then(|subject| subject.kind.as_ref())
                && let Some((identifier, file)) = super::create::frozen(context, execution).await?
                && file.metadata.as_ref().is_some_and(|metadata| metadata.id == request.name)
            {
                if request.expected_revision != file.sha256 {
                    return Err(Error::Conflict("Publish Report: Use the Frozen File Digest as Its Revision".into()));
                }
                let report = context.host.store().reports(execution)?.into_iter().find(|report| report.identifier == identifier);
                let report = report.ok_or_else(|| Error::Precondition("Publish Report: Restore Its Frozen Report".into()))?;
                return response(context, report, &request.request_id);
            }
            return Err(Error::NotFound("Publish Report: Create Its Prepared Report First".into()));
        }
        let file = prepared
            .response
            .and_then(|response| response.file)
            .ok_or_else(|| Error::Precondition("Publish Report: Restore Its Prepared File Reference".into()))?;
        if request.expected_revision != file.sha256 {
            return Err(Error::Conflict("Publish Report: Use the Prepared File Digest as Its Revision".into()));
        }
        let subject_matches = matches!(
            request.subject.as_ref().and_then(|subject| subject.kind.as_ref()),
            Some(Kind::Execution(execution)) if execution == &prepared.execution
        );
        if !subject_matches {
            return Err(Error::Invalid("Publish Report: Keep Its Prepared Execution Subject".into()));
        }
        transfer::authorize(context, &file)?;
        let retained = file.clone();
        let length = file.length;
        let file = storage::FileReference::try_from(file).map_err(|cause| Error::Invalid(format!("Publish Report: {cause}")))?;
        let max_bytes = transfer::chunk_bytes(context).await? as u64;
        let mut html = Vec::new();
        while (html.len() as u64) < length {
            let request = GetFileBytesRequest {
                file: Some(file.clone()),
                offset: html.len() as u64,
                max_bytes,
                at: None,
            };
            let response = context.storage::<GetFileBytes>(request).await?;
            if response.data.is_empty() {
                return Err(Error::Precondition("Publish Report: Restore the Complete Prepared File".into()));
            }
            html.extend(response.data);
        }
        let document = prepared.document.ok_or_else(|| Error::Invalid("Publish Report: Restore Its Frozen Document".into()))?;
        let report = publish(context, &prepared.execution, &prepared.execution_seal, document, html, &retained).await?;
        tracing::info!(publication_ms = started.elapsed().as_secs_f64() * 1000.0, "Report Published");
        response(context, report, &request.request_id)
    })
    .await
}

/// Publish a Rendered Snapshot of a Sealed Execution and Retain Its Native File for Download.
pub(super) async fn publish(
    context: &Context, execution: &str, execution_seal: &[u8], document: Document, html: Vec<u8>, file: &FileReference,
) -> Result<Report> {
    let report = context.host.store().publish_report(execution, execution_seal, document, html)?;
    let key = format!("published-report/{}", report.identifier);
    administration::retain(context, &key, Scope::Execution(execution).identifier()?, file).await?;
    Ok(report)
}

/* === Private Functions === */

fn response(context: &Context, report: Report, request_id: &str) -> Result<ReportsPublishResponse> {
    let cut = context.host.store().metadata(&report.execution)?;
    Ok(ReportsPublishResponse {
        report: Some(ReportMetadata {
            test_cycle_id: cut.execution.cycle,
            test_execution_id: Some(report.execution.clone()),
            identifier: report.identifier.clone(),
            metadata: report.document.and_then(|document| document.metadata),
            origin: ReportOrigin::Sealed as i32,
            ..Default::default()
        }),
        seal: report.seal,
        receipt: Some(crate::host::views::receipt(context, request_id)?),
    })
}

/* === Tests === */

crate::native::unary!(ReportsPublishRequest, ReportsPublishResponse, context, "dokime.host.v1", "Reports", "Publish");
