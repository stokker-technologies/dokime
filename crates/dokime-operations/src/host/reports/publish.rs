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
        let seal = file.sha256.clone();
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
        let prepared = Report {
            execution: prepared.execution,
            identifier: request.name,
            document: prepared.document,
            execution_seal: prepared.execution_seal,
            html,
            seal,
        };
        let report = context.host.store().publish_report(
            &prepared.execution,
            &prepared.execution_seal,
            prepared.document.ok_or_else(|| Error::Invalid("Publish Report: Restore Its Frozen Document".into()))?,
            prepared.html,
        )?;
        let key = format!("published-report/{}", report.identifier);
        let scope = Scope::Execution(&report.execution).identifier()?;
        administration::retain(context, &key, scope, &retained).await?;
        let cut = context.host.store().metadata(&report.execution)?;
        tracing::info!(publication_ms = started.elapsed().as_secs_f64() * 1000.0, "Report Published");
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
            receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ReportsPublishRequest, ReportsPublishResponse, context, "dokime.host.v1", "Reports", "Publish");
