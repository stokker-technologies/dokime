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
use std::io::{Seek, Write};
use std::time::Instant;

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};
use harmos_protocol::model::FileMetadata;
use prost::Message;

// Local Imports
use super::{PreparedReport, capture};
use crate::host::files::uploads::{append, create, delete, publish};
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest};
use dokime_protocol::host::report_subject::Kind;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Reports/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ReportsCreateRequest) -> Result<ReportsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if request.document_id.is_empty()
            && let Some(Kind::Cycle(cycle)) = request.subject.as_ref().and_then(|subject| subject.kind.as_ref())
            && context.host.store().client_cycle(cycle)?.is_some_and(|cycle| cycle.sealed)
        {
            let saved = super::get::handle(
                context,
                ReportsGetRequest {
                    subject: request.subject,
                    ..Default::default()
                },
            )
            .await?;
            return Ok(ReportsCreateResponse {
                report: saved.report,
                file: saved.file,
                resolved_at: saved.resolved_at,
                receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
            });
        }
        prepare(context, request).await
    })
    .await
}

pub(crate) async fn prepare(context: &Context, request: ReportsCreateRequest) -> Result<ReportsCreateResponse> {
    let started = Instant::now();
    if request.request_id.is_empty() {
        return Err(Error::Invalid("Create Report: Supply a Nonempty Request ID".into()));
    }
    let name = format!("prepared-report/{}", request.request_id);
    let (prepared, revision) = administration::message::<PreparedReport>(context, &name).await?;
    if !revision.is_empty() {
        return prepared
            .response
            .ok_or_else(|| Error::Precondition("Create Report: Use a New Request ID After Replacing Incompatible Prepared Data".into()));
    }
    let (capture, resolved_at, prepared) = capture::collect(context, &request).await?;
    let metadata = capture.report.clone().ok_or_else(|| Error::Invalid("Create Report: Capture Its Metadata".into()))?;
    let phase = Instant::now();
    let template = context.host.report_template();
    let marker = b"<!-- DOKIME_REPORT_DATA -->";
    let insertion = template
        .windows(marker.len())
        .position(|window| window == marker)
        .ok_or_else(|| Error::Invalid("Create Report: Rebuild the Template with `mise run generate`".into()))?;
    let data = STANDARD.encode(capture.encode_to_vec());
    tracing::debug!(phase = "serialization", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let phase = Instant::now();
    let mut file = tempfile::tempfile().map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.write_all(&template[..insertion])
        .map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.write_all(data.as_bytes()).map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.write_all(&template[insertion + marker.len()..])
        .map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.rewind().map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    let (retained, length, digest) = crate::host::content::transfer::temporary(file)?;
    tracing::debug!(phase = "template", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let phase = Instant::now();
    let digest = (0..digest.len()).step_by(2).map(|index| u8::from_str_radix(&digest[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
    let scope = match &metadata.test_execution_id {
        Some(execution) => Scope::Execution(execution),
        None => Scope::Cycle(&metadata.test_cycle_id),
    };
    let scope = scope.identifier()?;
    let created = create::derived(
        context,
        UploadsCreateRequest {
            metadata: Some(FileMetadata {
                id: name.clone(),
                scope: Some(scope.to_string()),
                media_type: "text/html".into(),
                expected_length: Some(length),
                expected_sha256: Some(digest.clone()),
            }),
            request_id: format!("{ID}/{name}/create"),
        },
    )
    .await?;
    let upload = created.upload.ok_or_else(|| Error::Precondition("Create Report: Create Its Native Upload".into()))?;
    let result = async {
        let mut offset = 0;
        while offset < length {
            let count = (length - offset).min(created.chunk_bytes) as usize;
            let mut data = vec![0; count];
            crate::host::content::transfer::read_temporary(&retained, offset, &mut data)?;
            append::handle(
                context,
                UploadsAppendRequest {
                    upload: Some(upload.clone()),
                    offset,
                    data,
                    request_id: format!("{ID}/{name}/{offset}"),
                },
            )
            .await?;
            offset += count as u64;
        }
        let published = publish::handle(
            context,
            UploadsPublishRequest {
                upload: Some(upload.clone()),
                length,
                sha256: digest,
                request_id: format!("{ID}/{name}/publish"),
            },
        )
        .await?;
        published.upload.ok_or_else(|| Error::Precondition("Create Report: Publish Its Native File".into()))
    }
    .await;
    let reference = match result {
        Ok(reference) => reference,
        Err(source) => {
            if let Err(cleanup) = delete::handle(
                context,
                UploadsDeleteRequest {
                    upload: Some(upload),
                    request_id: uuid::Uuid::now_v7().to_string(),
                },
            )
            .await
            {
                return Err(Error::Invalid(format!("Create Report: {source}; Remove Its Failed Upload: {cleanup}")));
            }
            return Err(source);
        }
    };

    let mut prepared = prepared.unwrap_or_default();
    prepared.response = Some(ReportsCreateResponse {
        report: Some(metadata),
        file: Some(reference),
        resolved_at: Some(resolved_at),
        receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
    });
    let prepared = administration::retain(context, &name, scope, &prepared).await?;
    let response = prepared
        .response
        .ok_or_else(|| Error::Precondition("Create Report: Use a New Request ID After Replacing Incompatible Prepared Data".into()))?;
    let reference =
        response.file.clone().ok_or_else(|| Error::Precondition("Create Report: Restore Its Retained File Reference".into()))?;
    context.host.retain_report(reference)?;
    tracing::debug!(phase = "upload_retention", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    tracing::info!(report = %name, creation_ms = started.elapsed().as_secs_f64() * 1000.0, "Report Created");
    Ok(response)
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ReportsCreateRequest, ReportsCreateResponse, context, "dokime.host.v1", "Reports", "Create");
