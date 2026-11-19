/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Capture Native Report Responses

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::Instant;

// Third Party Imports
use harmos_protocol::model::{Declarations, FileReference, History, PageRequest, Position, ReadCut};
use prost::Message;

// Local Imports
use super::PreparedReport;
use crate::host::{Context, documents};
use crate::{Error, Result};
use dokime_protocol::host::{self, content, evidence, executions, plots};
use dokime_protocol::model::{ArtifactScope, EvidenceScope, Suites};
use dokime_storage::ExecutionState;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn collect(
    context: &Context, request: &host::ReportsCreateRequest,
) -> Result<(host::ReportCapture, ReadCut, Option<PreparedReport>)> {
    let phase = Instant::now();
    let subject = request
        .subject
        .as_ref()
        .and_then(|subject| subject.kind.as_ref())
        .ok_or_else(|| Error::Invalid("Create Report: Select Its Execution or Cycle".into()))?;
    let store = context.host.store();
    let (mut reader, _cancellation) = store.read_session()?;
    reader.reuse_sample_reads()?;
    let (cycle, execution) = match subject {
        host::report_subject::Kind::Execution(id) => (reader.metadata(id)?.execution.cycle, Some(id.clone())),
        host::report_subject::Kind::Cycle(id) => (id.clone(), None),
    };
    let mut cuts = Vec::new();
    let mut after = String::new();
    loop {
        let page = reader.cycle_execution_page(&cycle, &after, NonZeroUsize::new(context.settings.page_items).unwrap())?;
        if page.is_empty() {
            break;
        }
        for id in page {
            after = id.clone();
            if execution.as_ref().is_some_and(|selected| selected != &id) {
                continue;
            }
            let cut = reader.cut(&id)?;
            if cut.state != ExecutionState::Sealed {
                return Err(Error::Precondition(format!("Create Report: Wait until Execution {id} Is Sealed")));
            }
            cuts.push(cut);
        }
    }
    if cuts.is_empty() && execution.is_some() {
        return Err(Error::Invalid("Create Report: Select an Existing Execution or Cycle with Sealed Executions".into()));
    }
    let resolved_at = ReadCut {
        positions: cuts
            .iter()
            .map(|cut| Position {
                history: Some(History {
                    store: context.host.identity().backend_id,
                    epoch: cut.execution.identifier.clone(),
                }),
                sequence: cut.frontier,
            })
            .collect(),
        revisions: Vec::new(),
    };
    tracing::debug!(phase = "resolve_cut", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let phase = Instant::now();
    let mut capture = host::ReportCapture {
        report: Some(host::ReportMetadata {
            test_cycle_id: cycle.clone(),
            test_execution_id: execution.clone(),
            identifier: if execution.is_some() { "execution".into() } else { "cycle".into() },
            generated_at: Some(crate::host::projection::timestamp(context.host.now_ns())),
            origin: host::ReportOrigin::Current as i32,
            ..Default::default()
        }),
        overview: Some(crate::host::backend::overview::get::handle(context, host::backend::OverviewGetRequest::default()).await?),
        cycle: Some(
            crate::host::cycles::get::handle(
                context,
                host::CyclesGetRequest {
                    name: cycle.clone(),
                    ..Default::default()
                },
            )
            .await?,
        ),
        appearance: Some(crate::host::backend::appearance::handle(context, host::BackendAppearanceRequest::default()).await?),
        ..Default::default()
    };
    if execution.is_none()
        && let Some(cycle) = capture.cycle.as_mut().and_then(|cycle| cycle.cycle.as_mut())
        && cycle.phase == host::CyclePhase::Sealing as i32
    {
        cycle.phase = host::CyclePhase::Sealed as i32;
        cycle.sealed = true;
        if let Some(report) = capture.report.as_mut() {
            report.origin = host::ReportOrigin::Sealed as i32;
        }
    }
    let cases = crate::host::cases::list::handle(
        context,
        host::CasesListRequest {
            parent: cycle.clone(),
            ..Default::default()
        },
    )
    .await?;
    for case in cases.cases {
        capture.cases.push(host::ReportCase {
            detail: Some(
                crate::host::cases::get::handle(
                    context,
                    host::CasesGetRequest {
                        parent: cycle.clone(),
                        name: case.case_id.clone(),
                        ..Default::default()
                    },
                )
                .await?,
            ),
            parameters: Some(
                crate::host::cases::parameters::handle(
                    context,
                    host::CasesParametersRequest {
                        parent: cycle.clone(),
                        name: case.case_id,
                        ..Default::default()
                    },
                )
                .await?,
            ),
        });
    }
    tracing::debug!(phase = "planning", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let mut suites = BTreeSet::new();
    let mut prepared = None;
    for cut in &cuts {
        let id = &cut.execution.identifier;
        let declarations = Declarations::decode(cut.execution.catalog.as_slice())?;
        for section in declarations.sections {
            if section.r#type != "dokime.model.v1.Suites" {
                continue;
            }
            for suite in Suites::decode(section.payload.as_slice())?.suites {
                let key = suite.metadata.as_ref().map(|metadata| metadata.identifier.clone()).unwrap_or_default();
                if suites.insert(key) {
                    capture.suites.push(host::SuitesGetResponse { value: Some(suite), ..Default::default() });
                }
            }
        }
        let scope = host::DocumentScope {
            cycle_id: cycle.clone(),
            execution_id: id.clone(),
        };
        let (document, _) = context.host.current_document(&scope, &request.document_id)?;
        if execution.is_some() {
            prepared = Some(PreparedReport {
                execution: id.clone(),
                execution_seal: cut.seal.clone(),
                document: Some(document.clone()),
                ..Default::default()
            });
        }
        let phase = Instant::now();
        let sections = document.sections.len();
        let document = documents::get::response_at_with_reader(
            context,
            &host::DocumentsGetRequest {
                scope: Some(scope),
                document_id: request.document_id.clone(),
                ..Default::default()
            },
            document,
            cut,
            &reader,
        )
        .await?;
        tracing::debug!(phase = "documents", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, sections, "Report Stage");
        let phase = Instant::now();
        let mut requests = Vec::new();
        for frame in &document.frames {
            let Some(host::document_frame_frame::Kind::Binding(binding)) = frame.frame.as_ref().and_then(|frame| frame.kind.as_ref())
            else {
                continue;
            };
            let Some(host::document_binding_result::Kind::Plot(plot)) = binding.result.as_ref().and_then(|result| result.kind.as_ref())
            else {
                continue;
            };
            let plot = plot.request.as_ref().ok_or_else(|| Error::Invalid("Create Report: Restore the Bound Plot Request".into()))?;
            requests.push(plot.clone());
        }
        let envelopes = crate::host::plots::snapshots::list::snapshots_at(
            &reader,
            cut,
            requests,
            context.settings.page_items,
            context.host.now_ns(),
            &context.host.identity().epoch,
        )
        .await?;
        tracing::debug!(phase = "plots", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, plots = envelopes.len(), "Report Stage");
        let phase = Instant::now();
        let mut captured = host::ReportExecution {
            detail: Some(
                crate::host::executions::get::handle(context, host::ExecutionsGetRequest { name: id.clone(), ..Default::default() })
                    .await?,
            ),
            inputs: Some(crate::host::executions::parameters::response_at(&reader, cut, host::ExecutionParameterSet::Inputs as i32)?),
            outputs: Some(crate::host::executions::parameters::response_at(&reader, cut, host::ExecutionParameterSet::Outputs as i32)?),
            steps: Some(crate::host::executions::steps::response_at(
                &reader,
                cut,
                NonZeroUsize::new(context.settings.page_items).unwrap(),
            )?),
            document: Some(document),
            plots: Some(plots::SnapshotsListResponse {
                envelopes,
                resolved_at: Some(ReadCut {
                    positions: vec![resolved_at.positions[capture.executions.len()].clone()],
                    revisions: Vec::new(),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        tracing::debug!(phase = "execution", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
        let phase = Instant::now();
        let mut at = None;
        let mut token = String::new();
        loop {
            let reply = crate::host::executions::schemas::get::handle(
                context,
                executions::SchemasGetRequest {
                    name: id.clone(),
                    at: at.clone(),
                    page: Some(PageRequest { page_token: token, ..Default::default() }),
                },
            )
            .await?;
            at = reply.resolved_at.clone();
            token = reply.next_page_token.clone();
            captured.schemas.push(reply);
            if token.is_empty() {
                break;
            }
        }
        tracing::debug!(phase = "schemas", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
        let phase = Instant::now();
        let mut at = None;
        let mut token = String::new();
        loop {
            let reply = crate::host::evidence::traceability::get::handle(
                context,
                evidence::TraceabilityGetRequest {
                    scope: Some(EvidenceScope {
                        cycle_id: Some(cycle.clone()),
                        execution_id: Some(id.clone()),
                        ..Default::default()
                    }),
                    at: at.clone(),
                    page: Some(PageRequest { page_token: token, ..Default::default() }),
                    ..Default::default()
                },
            )
            .await?;
            at = reply.resolved_at.clone();
            token = reply.next_page_token.clone();
            captured.traceability.push(reply);
            if token.is_empty() {
                break;
            }
        }
        tracing::debug!(phase = "traceability", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
        let phase = Instant::now();
        let mut at = None;
        let mut token = String::new();
        loop {
            let reply = crate::host::content::execution_files::list::handle(
                context,
                content::ExecutionFilesListRequest {
                    parent: id.clone(),
                    at: at.clone(),
                    page: Some(PageRequest { page_token: token, ..Default::default() }),
                },
            )
            .await?;
            at = reply.resolved_at;
            token = reply.next_page_token;
            for file in reply.files {
                let reply = crate::host::content::execution_files::get::handle(
                    context,
                    content::ExecutionFilesGetRequest {
                        parent: id.clone(),
                        name: file.relative_path,
                        ..Default::default()
                    },
                )
                .await?;
                if let Some(file) = &reply.file {
                    capture.files.push(read_file(context, file).await?);
                }
                captured.files.push(reply);
            }
            if token.is_empty() {
                break;
            }
        }
        tracing::debug!(phase = "execution_files", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
        capture.executions.push(captured);
    }
    if execution.is_none() && !cuts.is_empty() {
        let scope = host::DocumentScope {
            cycle_id: cycle.clone(),
            execution_id: String::new(),
        };
        let (document, fixed) = context.host.current_document(&scope, "default")?;
        capture.document = Some(
            documents::get::response_at(
                context,
                &host::DocumentsGetRequest {
                    scope: Some(scope),
                    document_id: "default".into(),
                    ..Default::default()
                },
                document,
                &fixed,
            )
            .await?,
        );
    }
    let phase = Instant::now();
    let selection = Some(evidence::AnnotationSelection {
        owner: Some(evidence::AnnotationOwner {
            owner: Some(evidence::AnnotationOwnerOwner {
                kind: Some(match &execution {
                    Some(id) => evidence::annotation_owner_owner::Kind::Execution(evidence::ExecutionNoteOwner {
                        cycle_id: cycle.clone(),
                        execution_id: id.clone(),
                    }),
                    None => evidence::annotation_owner_owner::Kind::CycleId(cycle.clone()),
                }),
            }),
        }),
        include_context: false,
    });
    capture.notes = crate::host::evidence::notes::list::capture(context, selection.clone()).await?;
    tracing::debug!(phase = "notes", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let phase = Instant::now();
    let mut at = None;
    let mut token = String::new();
    loop {
        let reply = crate::host::evidence::health_events::list::handle(
            context,
            evidence::HealthEventsListRequest {
                selection: selection.clone(),
                at: at.clone(),
                page: Some(PageRequest { page_token: token, ..Default::default() }),
            },
        )
        .await?;
        at = reply.resolved_at.clone();
        token = reply.next_page_token.clone();
        capture.health.push(reply);
        if token.is_empty() {
            break;
        }
    }
    tracing::debug!(phase = "health", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let phase = Instant::now();
    let scope = EvidenceScope {
        cycle_id: Some(cycle),
        execution_id: execution,
        ..Default::default()
    };
    let mut at = None;
    let mut token = String::new();
    loop {
        let reply = crate::host::evidence::records::list::handle(
            context,
            evidence::RecordsListRequest {
                selection: Some(evidence::CapturedRecordSelection {
                    scope: Some(scope.clone()),
                    ..Default::default()
                }),
                at: at.clone(),
                page: Some(PageRequest { page_token: token, ..Default::default() }),
                ..Default::default()
            },
        )
        .await?;
        at = reply.resolved_at.clone();
        token = reply.next_page_token.clone();
        capture.records.push(reply);
        if token.is_empty() {
            break;
        }
    }
    tracing::debug!(
        phase = "records",
        elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0,
        records = capture.records.iter().map(|page| page.records.len()).sum::<usize>(),
        "Report Stage"
    );
    let phase = Instant::now();
    let mut at = None;
    let mut token = String::new();
    loop {
        let reply = crate::host::content::artifacts::list::handle(
            context,
            content::ArtifactsListRequest {
                scope: Some(ArtifactScope {
                    cycle_id: scope.cycle_id.clone(),
                    execution_id: scope.execution_id.clone(),
                    ..Default::default()
                }),
                at: at.clone(),
                page: Some(PageRequest { page_token: token, ..Default::default() }),
            },
        )
        .await?;
        at = reply.resolved_at;
        token = reply.next_page_token;
        for artifact in reply.artifacts {
            let reply = crate::host::content::artifacts::get::handle(
                context,
                content::ArtifactsGetRequest {
                    name: artifact.artifact_id,
                    ..Default::default()
                },
            )
            .await?;
            if let Some(file) = &reply.file {
                capture.files.push(read_file(context, file).await?);
            }
            capture.artifacts.push(reply);
        }
        if token.is_empty() {
            break;
        }
    }
    tracing::debug!(phase = "artifacts", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    let phase = Instant::now();
    if let Some(appearance) = capture.appearance.as_ref().and_then(|reply| reply.appearance.as_ref()) {
        for logo in &appearance.logos {
            if let Some(file) = logo.asset.as_ref().and_then(|asset| asset.file.as_ref()) {
                capture.files.push(read_file(context, file).await?);
            }
        }
    }
    tracing::debug!(phase = "logos", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Report Stage");
    Ok((capture, resolved_at, prepared))
}

/* === Private Functions === */

async fn read_file(context: &Context, file: &FileReference) -> Result<host::ReportFile> {
    let mut captured = host::ReportFile {
        file: Some(file.clone()),
        reads: Vec::new(),
    };
    let mut offset = 0;
    let mut at = None;
    while offset < file.length {
        let reply = crate::host::files::read::handle(
            context,
            host::FilesReadRequest {
                file: Some(file.clone()),
                offset,
                max_bytes: 0,
                at: at.clone(),
            },
        )
        .await?;
        if reply.data.is_empty() || reply.total_length != file.length || reply.sha256 != file.sha256 {
            return Err(Error::Invalid("Create Report: Restore the Immutable File Length and Digest".into()));
        }
        offset += reply.data.len() as u64;
        at = reply.resolved_at.clone();
        captured.reads.push(reply);
    }
    Ok(captured)
}

/* === Tests === */
