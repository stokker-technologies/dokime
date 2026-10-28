/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use dokime_domain::model::execution::saved::ExecutionCheckpoint;

use std::collections::BTreeMap;

use harmos::storage::ObjectRevision;
use serde::{Deserialize, Serialize};

use crate::execution::storage::{ExecutionDocument, ExecutionMetadata};
use crate::runtime::query::executions::{self, ExecutionFilter, ExecutionOrder};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::execution::result::ResultStatus;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionBrowseQuery {
    pub after: Option<TestExecutionId>,
    pub limit: Option<usize>,
    pub filter: ExecutionFilter,
    pub order: ExecutionOrder,
}

struct Archived<'a> {
    cycles: &'a std::collections::BTreeSet<TestCycleId>,
    executions: &'a std::collections::BTreeSet<TestExecutionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionBrowseEntry {
    pub metadata: ExecutionMetadata,
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub availability: ExecutionAvailability,
    pub archived: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionBrowseIssue {
    pub source: Option<String>,
    pub dataset: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionBrowsePage {
    pub entries: Vec<ExecutionBrowseEntry>,
    pub next: Option<TestExecutionId>,
    pub discovery_complete: bool,
    pub conflicts: Vec<TestExecutionId>,
    pub omitted_conflicts: usize,
    pub issues: Vec<ExecutionBrowseIssue>,
    pub omitted_issues: usize,
    pub cycles: Vec<ExecutionBrowseCycle>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBrowseCycle {
    pub cycle_id: TestCycleId,
    pub label: Option<String>,
    pub phase: Option<dokime_domain::model::execution::result::CyclePhase>,
    pub lifecycle: ExecutionBrowseLifecycle,
    pub result_status: Option<ResultStatus>,
    pub case_count: usize,
    pub execution_count: usize,
    pub unrun_count: usize,
    pub queued_count: usize,
    pub running_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub blocked_count: usize,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
}

#[derive(Default)]
struct CycleFold<'a> {
    cases: std::collections::BTreeSet<&'a CaseId>,
    executions: usize,
    unrun: usize,
    queued: usize,
    running: usize,
    passed: usize,
    failed: usize,
    blocked: usize,
    interrupted: bool,
    uncheckpointed: bool,
    started_at: Option<&'a Timestamp>,
    finished_at: Option<&'a Timestamp>,
}

pub struct ExecutionCheckpointOutputs {
    view: crate::runtime::storage::execution::ExecutionView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionCheckpointDocument {
    pub checkpoint: ExecutionCheckpoint,
    pub document: ExecutionDocument,
    pub acquisition_finished: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionCheckpointEvidence {
    pub checkpoint: ExecutionCheckpoint,
    pub outputs: Vec<crate::runtime::query::ExecutionFileEntry>,
    pub artifacts: Vec<crate::artifacts::ArtifactRecord>,
    pub record_contracts: Vec<crate::records::RecordContract>,
    pub telemetry: Vec<super::checkpoint_telemetry::ExecutionTelemetryTable>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionCheckpointContext {
    pub checkpoint: ExecutionCheckpoint,
    pub scope: dokime_domain::model::execution::RecordScope,
    pub session_id: dokime_domain::model::identity::SessionId,
    pub cycle: crate::runtime::ExecutionCycleContext,
    pub execution_documents: BTreeMap<dokime_domain::model::identity::DocumentId, crate::document::DocumentDefinition>,
    pub prompts: Vec<crate::prompts::PromptInstance>,
    pub health: Vec<dokime_domain::model::execution::health::HealthItem>,
    pub notes: Vec<dokime_domain::model::execution::notes::OperatorNote>,
    pub traceability: Vec<dokime_domain::model::traceability::TraceabilitySnapshot>,
    pub appearance: crate::installed_plugins::RuntimeAppearanceFrame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionBrowseLifecycle {
    Running,
    Interrupted,
    Pending,
    Uncheckpointed,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionAvailability {
    Live,
    Stored,
    Interrupted,
    Uncheckpointed,
}

impl std::ops::Deref for ExecutionCheckpointOutputs {
    type Target = [dokime_domain::model::execution::saved::ExecutionOutput];

    fn deref(&self) -> &Self::Target {
        self.view.outputs().expect("checkpoint outputs were validated before retaining the view")
    }
}

impl ExecutionCheckpointOutputs {
    pub fn project<T>(
        self, project: impl FnOnce(&[dokime_domain::model::execution::saved::ExecutionOutput]) -> Result<T>,
    ) -> Result<harmos::storage::ReadBatch<T>> {
        self.view.project_revision(|revision| {
            let seal = revision.value.saved.as_ref().expect("checkpoint outputs were validated before retaining the view");
            project(&seal.source.outputs)
        })
    }
}
impl Archived<'_> {
    fn covers(&self, metadata: &ExecutionMetadata) -> bool {
        self.executions.contains(&metadata.execution_id) || self.cycles.contains(&metadata.cycle_id)
    }
}

impl CycleFold<'_> {
    fn finish(&self, cycle_id: &TestCycleId) -> ExecutionBrowseCycle {
        let ran = self.executions;
        let pending = self.queued > 0 || ran == 0;
        let lifecycle = if self.running > 0 {
            ExecutionBrowseLifecycle::Running
        } else if self.interrupted {
            ExecutionBrowseLifecycle::Interrupted
        } else if pending {
            ExecutionBrowseLifecycle::Pending
        } else if self.uncheckpointed {
            ExecutionBrowseLifecycle::Uncheckpointed
        } else {
            ExecutionBrowseLifecycle::Completed
        };
        let result_status = match lifecycle {
            ExecutionBrowseLifecycle::Completed if self.failed > 0 => Some(ResultStatus::Failed),
            ExecutionBrowseLifecycle::Completed if self.passed == ran => Some(ResultStatus::Passed),
            _ => None,
        };

        ExecutionBrowseCycle {
            cycle_id: cycle_id.clone(),
            label: None,
            phase: None,
            lifecycle,
            result_status,
            case_count: self.cases.len(),
            execution_count: ran,
            unrun_count: self.unrun,
            queued_count: self.queued,
            running_count: self.running,
            passed_count: self.passed,
            failed_count: self.failed,
            blocked_count: self.blocked,
            started_at: self.started_at.cloned(),
            finished_at: self.finished_at.cloned(),
        }
    }
}

impl RuntimeHandle {
    pub async fn checkpoint_execution(
        &self, checkpoint: &ExecutionCheckpoint,
    ) -> Result<crate::runtime::storage::execution::ExecutionView> {
        crate::runtime::storage::execution::ExecutionView::open_checkpoint(self.storage(), checkpoint).await?.ok_or_else(|| {
            Error::Admission(AdmissionFailure::NotFound {
                code: "execution_seal_missing".into(),
                message: "execution checkpoint has no retained exact seal".into(),
            })
        })
    }

    pub async fn execution_saved_checkpoint(&self, id: TestExecutionId) -> Result<ExecutionCheckpoint> {
        crate::runtime::storage::execution::ExecutionView::open(self.storage(), &id, harmos::storage::ObjectVersion::Latest)
            .await?
            .filter(|inspection| inspection.execution().saved.is_some())
            .ok_or_else(|| {
                Error::Admission(AdmissionFailure::NotFound {
                    code: "execution_seal_missing".into(),
                    message: "execution has no saved seal".into(),
                })
            })?
            .checkpoint()
    }

    pub async fn browse_executions(&self, query: ExecutionBrowseQuery) -> Result<harmos::storage::ReadBatch<ExecutionBrowsePage>> {
        let default_limit = 100;
        let limit = query.limit.unwrap_or(default_limit);
        if limit == 0 || limit as u64 > 1000 {
            return Err(error("execution page exceeds the configured limit"));
        }
        let session = self.storage().clone();
        let stored = stored_rows(crate::runtime::storage::history::executions(&session, self.storage().cursor_saved()).await?)?;
        let mut page = {
            let state = self.state_read();
            self.execution_storage.with_active_membership(|active| {
                let live_count = state.cycles.values().map(|cycle| state.executions_for_cycle(cycle).count()).sum();
                stored.try_map(|stored| {
                    let mut live = Vec::new();
                    live.try_reserve_exact(live_count).map_err(|_| error("history projection allocation failed"))?;
                    for cycle in state.cycles.values() {
                        for execution in state.executions_for_cycle(cycle) {
                            let mut metadata = ExecutionMetadata::new(execution);
                            let queued = state
                                .run_requests
                                .values()
                                .any(|request| !request.summary.phase.terminal() && request.pending.contains(&execution.test_execution_id));
                            if queued {
                                metadata.status = ExecutionStatus::Pending;
                            }
                            live.push(ExecutionBrowseEntry {
                                metadata,
                                checkpoint: None,
                                availability: if queued || active(&execution.test_execution_id) {
                                    ExecutionAvailability::Live
                                } else {
                                    ExecutionAvailability::Uncheckpointed
                                },
                                archived: false,
                            });
                        }
                    }
                    let archived = Archived {
                        cycles: &state.archived_cycles,
                        executions: &state.archived_executions,
                    };
                    browse_page(stored.iter().flat_map(|page| page.iter()), &live, &query, &archived, limit)
                })
            })?
        };
        // Keep Label Reads In The Same Admission Group As The Retained Response.
        for index in 0..page.cycles.len() {
            let from_live = {
                let state = self.state_read();
                if let Some(header) = state.cycles.get(&page.cycles[index].cycle_id) {
                    page = page.try_map(|mut page| -> Result<_> {
                        page.cycles[index].label = Some(header.metadata.label.clone());
                        page.cycles[index].phase = Some(header.phase);
                        Ok(page)
                    })?;
                    true
                } else {
                    false
                }
            };
            if from_live {
                continue;
            }
            if let Some(header) =
                crate::storage::objects::cycle::read(&session, &page.cycles[index].cycle_id, harmos::storage::ObjectVersion::Latest).await?
            {
                page = page.try_map_with(header, |mut page, header| -> Result<_> {
                    page.cycles[index].label = Some(header.value.metadata.label);
                    page.cycles[index].phase = Some(header.value.phase);
                    Ok(page)
                })?;
            }
        }
        Ok(page)
    }
    pub async fn execution_checkpoint_context(
        &self, checkpoint: ExecutionCheckpoint,
    ) -> Result<harmos::storage::ReadBatch<ExecutionCheckpointContext>> {
        let view = self.checkpoint_execution(&checkpoint).await?.into_snapshot().await?;
        view.project_captured(|_, captured| {
            let context = captured.context;
            Ok(ExecutionCheckpointContext {
                checkpoint,
                scope: context.scope,
                session_id: context.session_id,
                cycle: context.cycle,
                execution_documents: context.execution_documents,
                prompts: context.prompts,
                health: context.health,
                notes: context.notes,
                traceability: context.traceability,
                appearance: context.appearance,
            })
        })
    }

    pub async fn execution_checkpoint_outputs(&self, checkpoint: &ExecutionCheckpoint) -> Result<ExecutionCheckpointOutputs> {
        let view = self.checkpoint_execution(checkpoint).await?;
        view.outputs()?;
        Ok(ExecutionCheckpointOutputs { view })
    }

    pub async fn execution_checkpoint_evidence(
        &self, checkpoint: ExecutionCheckpoint,
    ) -> Result<harmos::storage::ReadBatch<ExecutionCheckpointEvidence>> {
        let view = self.checkpoint_execution(&checkpoint).await?.into_snapshot().await?;
        view.project_captured(|_, captured| {
            let record_count = contract_count(&captured.record_contracts)?;
            let telemetry_count = contract_count(&captured.telemetry_contracts)?;
            let mut record_contracts = projection_slots(record_count)?;
            let mut telemetry = projection_slots(telemetry_count)?;
            for versions in captured.record_contracts.into_values() {
                record_contracts.extend(versions.into_values());
            }
            for versions in captured.telemetry_contracts.into_values() {
                for contract in versions.into_values() {
                    telemetry.push(super::checkpoint_telemetry::ExecutionTelemetryTable {
                        table: contract.stream.as_str().to_owned(),
                        contract,
                    });
                }
            }
            let mut outputs = projection_slots(captured.outputs.len())?;
            let artifacts_count = captured
                .outputs
                .iter()
                .filter(|output| matches!(output.content, dokime_domain::model::execution::saved::ExecutionOutputContent::Artifact { .. }))
                .count();
            let mut artifacts = projection_slots(artifacts_count)?;
            for output in captured.outputs {
                outputs.push(crate::domains::documents::saved::entry(&checkpoint.execution_id, &output)?);
                if let dokime_domain::model::execution::saved::ExecutionOutputContent::Artifact { record, .. } = output.content {
                    artifacts.push(*record);
                }
            }
            Ok(ExecutionCheckpointEvidence {
                checkpoint,
                outputs,
                artifacts,
                record_contracts,
                telemetry,
            })
        })
    }

    pub async fn execution_checkpoint_document(
        &self, checkpoint: ExecutionCheckpoint,
    ) -> Result<harmos::storage::ReadBatch<ExecutionCheckpointDocument>> {
        let view = self.checkpoint_execution(&checkpoint).await?.into_snapshot().await?;
        view.project_captured(|mut revision, captured| {
            let cycle = captured.context.cycle.cycle;
            revision.value.saved = None;
            Ok(ExecutionCheckpointDocument {
                checkpoint,
                document: ExecutionDocument {
                    cycle: crate::execution::storage::ExecutionCycle {
                        cycle_id: cycle.test_cycle_id,
                        metadata: cycle.metadata,
                    },
                    execution: revision.value,
                },
                acquisition_finished: true,
            })
        })
    }
}

fn contract_count<T>(contracts: &BTreeMap<dokime_domain::model::identity::Identifier, BTreeMap<u32, T>>) -> Result<usize> {
    contracts.values().try_fold(0usize, |count, versions| {
        count.checked_add(versions.len()).ok_or_else(|| Error::Operation("checkpoint contract count overflows".into()))
    })
}

fn projection_slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| Error::Operation("checkpoint projection allocation failed".into()))?;
    Ok(values)
}

fn stored_rows(
    rows: crate::storage::projection::Pages<ObjectRevision<dokime_domain::model::execution::result::TestExecution>>,
) -> Result<crate::storage::projection::Pages<ExecutionBrowseEntry>> {
    Ok(crate::storage::projection::pages(rows, |row| {
        let checkpoint = row.value.saved.as_ref().map(|seal| ExecutionCheckpoint {
            execution_id: row.value.test_execution_id.clone(),
            position: row.position.clone(),
            version: row.version.clone(),
            evidence_through: seal.source.evidence_through.clone(),
        });
        Ok(ExecutionBrowseEntry {
            metadata: ExecutionMetadata::new(&row.value),
            availability: if checkpoint.is_some() { ExecutionAvailability::Stored } else { ExecutionAvailability::Interrupted },
            checkpoint,
            archived: false,
        })
    })?)
}
fn browse_page<'a>(
    stored: impl Iterator<Item = &'a ExecutionBrowseEntry>, live: &'a [ExecutionBrowseEntry], query: &ExecutionBrowseQuery,
    archived: &Archived<'_>, limit: usize,
) -> Result<ExecutionBrowsePage> {
    let admitted = |row: &ExecutionBrowseEntry| {
        executions::matches(&row.metadata, &query.filter) && query.filter.archive.admits(archived.covers(&row.metadata))
    };
    let mut candidates = BTreeMap::new();
    let mut checkpoints = BTreeMap::new();
    // Native Checkpoints Preserve Their Exact Revision Even When The Live Row Wins.
    for row in stored {
        checkpoints.insert(&row.metadata.execution_id, row.checkpoint.as_ref());
        if admitted(row) {
            candidates.insert(&row.metadata.execution_id, row);
        }
    }
    for row in live {
        let id = &row.metadata.execution_id;
        if candidates.contains_key(id) && !matches!(row.availability, ExecutionAvailability::Live) {
            continue;
        }
        if admitted(row) {
            candidates.insert(id, row);
        } else {
            candidates.remove(id);
        }
    }
    let mut selected = candidates.into_values().collect::<Vec<_>>();
    selected.sort_unstable_by(|left, right| executions::compare(&left.metadata, &right.metadata, query.order));
    let start = match &query.after {
        None => 0,
        Some(after) => {
            selected
                .iter()
                .position(|entry| &entry.metadata.execution_id == after)
                .ok_or_else(|| error("after execution ID is not in the filtered result"))?
                + 1
        }
    };
    let returned = selected.len().saturating_sub(start).min(limit);
    let more = selected.len().saturating_sub(start) > returned;

    // Fold The Whole Selection Into Per-Cycle Totals
    let mut folds = BTreeMap::<&TestCycleId, CycleFold>::new();
    for row in &selected {
        let metadata = &row.metadata;
        let fold = folds.entry(&metadata.cycle_id).or_default();
        if metadata.started_at.is_none() && metadata.status == ExecutionStatus::Skipped {
            fold.unrun += 1;
            continue;
        }
        let live = matches!(row.availability, ExecutionAvailability::Live);
        let checkpointed = row.checkpoint.is_some() || matches!(checkpoints.get(&metadata.execution_id), Some(Some(_)));
        fold.cases.insert(&metadata.case_id);
        fold.executions += 1;
        match metadata.status {
            ExecutionStatus::Pending => fold.queued += 1,
            ExecutionStatus::Running if live => fold.running += 1,
            ExecutionStatus::Passed => fold.passed += 1,
            ExecutionStatus::Failed => fold.failed += 1,
            ExecutionStatus::Blocked => fold.blocked += 1,
            _ => {}
        }
        fold.interrupted |= matches!(row.availability, ExecutionAvailability::Interrupted);
        fold.uncheckpointed |= matches!(row.availability, ExecutionAvailability::Uncheckpointed) && !checkpointed;
        if let Some(at) = &metadata.started_at {
            fold.started_at = Some(fold.started_at.map_or(at, |earliest| earliest.min(at)));
        }
        if let Some(at) = &metadata.finished_at {
            fold.finished_at = Some(fold.finished_at.map_or(at, |latest| latest.max(at)));
        }
    }
    let mut page_cycles = BTreeMap::new();
    for row in selected.iter().skip(start).take(returned) {
        let id = &row.metadata.cycle_id;
        page_cycles.entry(id).or_insert_with(|| &folds[id]);
    }

    let mut entries = Vec::with_capacity(returned);
    for row in selected.into_iter().skip(start).take(returned) {
        let mut entry = (*row).clone();
        entry.archived = archived.covers(&entry.metadata);
        if entry.checkpoint.is_none() {
            entry.checkpoint = checkpoints.get(&entry.metadata.execution_id).and_then(|checkpoint| checkpoint.cloned());
            if entry.checkpoint.is_some() && matches!(entry.availability, ExecutionAvailability::Uncheckpointed) {
                entry.availability = ExecutionAvailability::Stored;
            }
        }
        entries.push(entry);
    }
    let cycles = page_cycles.into_iter().map(|(id, fold)| fold.finish(id)).collect();
    let next = more.then(|| entries.last().expect("nonempty continued page").metadata.execution_id.clone());
    Ok(ExecutionBrowsePage {
        entries,
        next,
        discovery_complete: true,
        conflicts: Vec::new(),
        omitted_conflicts: 0,
        issues: Vec::new(),
        omitted_issues: 0,
        cycles,
    })
}

fn error(error: impl ToString) -> Error {
    Error::Operation(error.to_string())
}
