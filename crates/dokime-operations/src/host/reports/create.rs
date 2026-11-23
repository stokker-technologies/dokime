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
use std::fs::{self, File};
use std::future::Future;
use std::io::{Seek, Write};
use std::pin::Pin;
use std::time::Instant;

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};
use harmos_protocol::model::{FileMetadata, FileReference, History, Position, ReadCut};
use harmos_runtime::Identifier;
use prost::Message;
use uuid::Uuid;

// Local Imports
use super::{PreparedReport, capture};
use crate::host::files::transfer;
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::ReportSubject;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::report_subject::Kind;
use dokime_storage::persisted::Scope;
use dokime_storage::{ArchivedNativeFile, ExecutionState};

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
        // A Sealed Subject's Default Report Is Frozen Once; Download Delivers That File.
        if request.document_id.is_empty() {
            let store = context.host.store();
            let frozen = match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
                Some(Kind::Cycle(cycle)) if store.client_cycle(cycle)?.is_some_and(|cycle| cycle.sealed) => Some((String::new(), None)),
                Some(Kind::Execution(execution)) => match store.cut(execution)? {
                    cut if cut.state == ExecutionState::Sealed => {
                        Some((freeze(context, execution).await?.0, Some(sealed_cut(context, execution, cut.frontier))))
                    }
                    _ => None,
                },
                _ => None,
            };
            if let Some((name, resolved_at)) = frozen {
                let saved = super::get::handle(context, ReportsGetRequest { subject: request.subject, name, at: None }).await?;
                return Ok(ReportsCreateResponse {
                    report: saved.report,
                    file: saved.file,
                    resolved_at: resolved_at.or(saved.resolved_at),
                    receipt: Some(crate::host::views::receipt(context, &request.request_id)?),
                });
            }
        }
        prepare(context, request, None).await
    })
    .await
}

/// The Sealed Execution's Frozen Report Identity and File, Materialized Once by Its Seal or First Reader.
pub async fn freeze(context: &Context, execution: &str) -> Result<(String, FileReference)> {
    if let Some(frozen) = frozen(context, execution).await? {
        return Ok(frozen);
    }
    // The Host Settles Admitted Materialization After a Caller's Deadline Drops Its Receiver, Unless It Stops.
    let (owned, execution) = (context.clone(), execution.to_owned());
    let receiver = crate::host::stoppable_work(context, "Freeze Report", async move { materialize(&owned, &execution).await })?;
    receiver.await.map_err(|_| Error::Cancelled("Freeze Report: Reconnect After the Host Stops".into()))?
}

pub(crate) fn prepare(
    context: &Context, request: ReportsCreateRequest, scope: Option<Identifier>,
) -> Pin<Box<dyn Future<Output = Result<ReportsCreateResponse>> + Send + '_>> {
    Box::pin(async move {
        let started = Instant::now();
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Create Report: Supply a Nonempty Request ID".into()));
        }
        let name = format!("prepared-report/{}", request.request_id);
        let (prepared, revision) = administration::message::<PreparedReport>(context, &name).await?;
        if !revision.is_empty() {
            return prepared.response.ok_or_else(|| {
                Error::Precondition("Create Report: Use a New Request ID After Replacing Incompatible Prepared Data".into())
            });
        }
        let (mut prepared, rendered) = render(context, &request).await?;
        let scope = match scope {
            Some(scope) => scope,
            None => rendered_scope(&rendered)?,
        };
        let phase = Instant::now();
        let reference = upload(context, &rendered, scope.clone()).await?;
        let response = prepared.response.as_mut().unwrap();
        response.file = Some(reference);
        response.receipt = Some(crate::host::views::receipt(context, &request.request_id)?);
        let prepared = administration::retain(context, &name, scope, &prepared).await?;
        let response = prepared
            .response
            .ok_or_else(|| Error::Precondition("Create Report: Use a New Request ID After Replacing Incompatible Prepared Data".into()))?;
        tracing::debug!(phase = "upload_retention", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
        tracing::info!(report = %name, creation_ms = started.elapsed().as_secs_f64() * 1000.0, "Report Created");
        Ok(response)
    })
}

pub(crate) async fn render(context: &Context, request: &ReportsCreateRequest) -> Result<(PreparedReport, ArchivedNativeFile)> {
    let name = format!("prepared-report/{}", request.request_id);
    let (capture, resolved_at, prepared) = Box::pin(capture::collect(context, request)).await?;
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
    let mut file =
        tempfile::NamedTempFile::new().map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.write_all(&template[..insertion])
        .map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.write_all(data.as_bytes()).map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.write_all(&template[insertion + marker.len()..])
        .map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    file.rewind().map_err(|source| Error::Invalid(format!("Create Report: Write Its Temporary HTML: {source}")))?;
    let (retained, length, digest) = crate::host::content::transfer::temporary(
        file.reopen().map_err(|source| Error::Invalid(format!("Create Report: Read Its Temporary HTML: {source}")))?,
    )?;
    drop(retained);
    let retained = file.into_temp_path();
    tracing::debug!(phase = "template", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let digest = (0..digest.len()).step_by(2).map(|index| u8::from_str_radix(&digest[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
    let scope = match &metadata.test_execution_id {
        Some(execution) => Scope::Execution(execution),
        None => Scope::Cycle(&metadata.test_cycle_id),
    }
    .identifier()?;
    let mut prepared = prepared.unwrap_or_default();
    prepared.response = Some(ReportsCreateResponse {
        report: Some(metadata),
        resolved_at: Some(resolved_at),
        ..Default::default()
    });
    let reference = FileReference {
        metadata: Some(FileMetadata {
            id: name,
            scope: Some(scope.to_string()),
            media_type: "text/html".into(),
            ..Default::default()
        }),
        length,
        sha256: digest,
        ..Default::default()
    };
    Ok((prepared, ArchivedNativeFile { path: retained, reference }))
}

/* === Private Functions === */

async fn materialize(context: &Context, execution: &str) -> Result<(String, FileReference)> {
    let _admitted = context.reports.lock().await;
    if let Some(frozen) = frozen(context, execution).await? {
        return Ok(frozen);
    }
    let started = Instant::now();
    let request = ReportsCreateRequest {
        subject: Some(ReportSubject {
            kind: Some(Kind::Execution(execution.into())),
        }),
        request_id: Uuid::now_v7().to_string(),
        ..Default::default()
    };
    let (prepared, rendered) = render(context, &request).await?;
    let file = upload(context, &rendered, rendered_scope(&rendered)?).await?;
    let html = fs::read(&rendered.path).map_err(|source| Error::Invalid(format!("Freeze Report: Read Its Temporary HTML: {source}")))?;
    let document = prepared.document.ok_or_else(|| Error::Invalid("Freeze Report: Capture Its Document".into()))?;
    let report = super::publish::publish(context, execution, &prepared.execution_seal, document, html, &file).await?;
    context.host.store().retain_ranks(execution)?;
    context.host.signal_change();
    tracing::info!(%execution, creation_ms = started.elapsed().as_secs_f64() * 1000.0, "Report Frozen");
    Ok((report.identifier, file))
}

/// The First Published Report Whose Native File Is Retained, Whether Sealing or a Download Published It.
pub(super) async fn frozen(context: &Context, execution: &str) -> Result<Option<(String, FileReference)>> {
    for identifier in context.host.store().report_identifiers(execution)? {
        let (file, revision) = administration::message::<FileReference>(context, &format!("published-report/{identifier}")).await?;
        if !revision.is_empty() {
            return Ok(Some((identifier, file)));
        }
    }
    Ok(None)
}

async fn upload(context: &Context, rendered: &ArchivedNativeFile, scope: Identifier) -> Result<FileReference> {
    let metadata = rendered.reference.metadata.clone().unwrap();
    let retained =
        File::open(&rendered.path).map_err(|source| Error::Invalid(format!("Create Report: Reopen Its Temporary HTML: {source}")))?;
    transfer::publication(
        context,
        "Create Report",
        &format!("{ID}/{}", metadata.id),
        FileMetadata {
            scope: Some(scope.to_string()),
            expected_length: Some(rendered.reference.length),
            expected_sha256: Some(rendered.reference.sha256.clone()),
            ..metadata
        },
        |offset, data| crate::host::content::transfer::read_temporary(&retained, offset, data),
        || context.host.shutting_down(),
    )
    .await
}

/// The Cut a Sealed Execution's Report Captured: Its Fixed Frontier in Its Own History.
fn sealed_cut(context: &Context, execution: &str, frontier: u64) -> ReadCut {
    ReadCut {
        positions: vec![Position {
            history: Some(History {
                store: context.host.identity().backend_id,
                epoch: execution.into(),
            }),
            sequence: frontier,
        }],
        revisions: Vec::new(),
    }
}

fn rendered_scope(rendered: &ArchivedNativeFile) -> Result<Identifier> {
    let scope = rendered.reference.metadata.as_ref().and_then(|metadata| metadata.scope.clone()).unwrap();
    Identifier::new(scope).map_err(|source| Error::Invalid(format!("Create Report: {source}")))
}

/* === Tests === */

crate::native::unary!(ReportsCreateRequest, ReportsCreateResponse, context, "dokime.host.v1", "Reports", "Create");
