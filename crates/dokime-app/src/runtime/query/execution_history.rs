/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */
pub use dokime::execution::saved::ExecutionCheckpoint;

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, num::NonZeroUsize};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectRevision, QueryValue, ReadLimits, SharedQueryValue},
};
use serde::{Deserialize, Serialize};
// Local Imports
use crate::execution::storage::{ExecutionDocument, ExecutionMetadata};
use crate::runtime::query::executions::{self, ExecutionFilter, ExecutionOrder};
use crate::{Error, Result, RuntimeHandle};
use dokime::core::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionBrowseQuery {
    pub after: Option<TestExecutionId>,
    pub limit: Option<usize>,
    pub filter: ExecutionFilter,
    pub order: ExecutionOrder,
}

/// The archive marks a browse reads once, for every row it selects.
struct Archived {
    cycles: std::collections::BTreeSet<dokime::core::TestCycleId>,
    executions: std::collections::BTreeSet<TestExecutionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionBrowseEntry {
    pub metadata: ExecutionMetadata,
    pub checkpoint: Option<ExecutionCheckpoint>,
    pub availability: ExecutionAvailability,
    /// Whether an operator archived this execution or its cycle.
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
    pub artifacts: Vec<dokime::artifacts::ArtifactRecord>,
    pub record_contracts: Vec<crate::records::RecordContract>,
    pub telemetry: Vec<super::checkpoint_telemetry::ExecutionTelemetryTable>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionCheckpointContext {
    pub checkpoint: ExecutionCheckpoint,
    pub scope: dokime::core::RecordScope,
    pub session_id: dokime::core::SessionId,
    pub cycle: crate::runtime::ExecutionCycleContext,
    pub execution_documents:
        BTreeMap<dokime::core::DocumentId, crate::document::DocumentDefinition>,
    pub prompts: Vec<dokime::prompts::PromptInstance>,
    pub health: Vec<crate::execution::HealthItem>,
    pub notes: Vec<crate::execution::OperatorNote>,
    pub traceability: Vec<crate::traceability::TraceabilitySnapshot>,
    pub appearance: crate::installed_plugins::RuntimeAppearanceFrame,
}

struct BrowseRows {
    pages: Vec<SharedQueryValue<Vec<ExecutionBrowseEntry>>>,
}

/* === Enums === */
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionAvailability {
    Live,
    Stored,
    Interrupted,
    Uncheckpointed,
}

/* === Traits === */

/* === Implementations === */
impl Archived {
    /// An execution reads as archived when it, or its whole cycle, is.
    fn covers(&self, metadata: &ExecutionMetadata) -> bool {
        self.executions.contains(&metadata.execution_id) || self.cycles.contains(&metadata.cycle_id)
    }
}

impl MemorySize for ExecutionCheckpointEvidence {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.checkpoint.memory_size(),
                std::mem::size_of_val(&self.checkpoint),
            ),
            (
                self.outputs.memory_size(),
                std::mem::size_of_val(&self.outputs),
            ),
            (
                self.artifacts.memory_size(),
                std::mem::size_of_val(&self.artifacts),
            ),
            (
                self.record_contracts.memory_size(),
                std::mem::size_of_val(&self.record_contracts),
            ),
            (
                self.telemetry.memory_size(),
                std::mem::size_of_val(&self.telemetry),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl MemorySize for ExecutionBrowseEntry {
    fn memory_size(&self) -> usize {
        let metadata = self.metadata.memory_size();
        let checkpoint = self.checkpoint.memory_size();
        if [metadata, checkpoint].contains(&usize::MAX) {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<ExecutionMetadata>()))
            .saturating_add(
                checkpoint.saturating_sub(std::mem::size_of::<Option<ExecutionCheckpoint>>()),
            )
    }
}

impl MemorySize for ExecutionBrowseIssue {
    fn memory_size(&self) -> usize {
        let source = self.source.memory_size();
        let dataset = self.dataset.memory_size();
        if [source, dataset].contains(&usize::MAX) {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(source.saturating_sub(std::mem::size_of::<Option<String>>()))
            .saturating_add(dataset.saturating_sub(std::mem::size_of::<Option<String>>()))
            .saturating_add(self.message.capacity())
    }
}

impl MemorySize for ExecutionBrowsePage {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.entries.memory_size(),
                std::mem::size_of_val(&self.entries),
            ),
            (self.next.memory_size(), std::mem::size_of_val(&self.next)),
            (
                self.conflicts.memory_size(),
                std::mem::size_of_val(&self.conflicts),
            ),
            (
                self.issues.memory_size(),
                std::mem::size_of_val(&self.issues),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl MemorySize for ExecutionCheckpointContext {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.checkpoint.memory_size(),
                std::mem::size_of_val(&self.checkpoint),
            ),
            (self.scope.memory_size(), std::mem::size_of_val(&self.scope)),
            (
                self.session_id.memory_size(),
                std::mem::size_of_val(&self.session_id),
            ),
            (self.cycle.memory_size(), std::mem::size_of_val(&self.cycle)),
            (
                self.execution_documents.memory_size(),
                std::mem::size_of_val(&self.execution_documents),
            ),
            (
                self.prompts.memory_size(),
                std::mem::size_of_val(&self.prompts),
            ),
            (
                self.health.memory_size(),
                std::mem::size_of_val(&self.health),
            ),
            (self.notes.memory_size(), std::mem::size_of_val(&self.notes)),
            (
                self.traceability.memory_size(),
                std::mem::size_of_val(&self.traceability),
            ),
            (
                self.appearance.memory_size(),
                std::mem::size_of_val(&self.appearance),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl MemorySize for ExecutionCheckpointDocument {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.checkpoint.memory_size(),
                std::mem::size_of_val(&self.checkpoint),
            ),
            (
                self.document.memory_size(),
                std::mem::size_of_val(&self.document),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl MemorySize for BrowseRows {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(self.pages.capacity().saturating_mul(
            std::mem::size_of::<SharedQueryValue<Vec<ExecutionBrowseEntry>>>(),
        ))
    }
}

impl RuntimeHandle {
    pub async fn checkpoint_execution(
        &self,
        checkpoint: &ExecutionCheckpoint,
    ) -> Result<crate::runtime::storage::execution::ExecutionView> {
        let config = self.config();
        let settings = &config.storage;
        let storage = self.storage();
        let view = storage.view_with_limits(settings.interactive).await?;
        let inspection = crate::runtime::storage::execution::ExecutionView::open_checkpoint(
            storage,
            view,
            checkpoint,
            settings.decoded_bytes,
        )
        .await?;
        inspection.ok_or_else(|| Error::NotFound {
            code: "execution_seal_missing".into(),
            message: "execution checkpoint has no retained exact seal".into(),
        })
    }

    pub async fn execution_saved_checkpoint<U: MemorySize>(
        &self,
        id: TestExecutionId,
        map: impl FnOnce(&ExecutionCheckpoint, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let config = self.config();
        let settings = &config.storage;
        let storage = self.storage();
        let limit = settings.decoded_bytes;
        let view = storage.view_with_limits(settings.interactive).await?;
        let inspection = crate::runtime::storage::execution::ExecutionView::open(
            storage,
            view,
            &id,
            harmos::storage::ObjectVersion::Latest,
            limit,
        )
        .await?
        .filter(|inspection| inspection.execution().saved.is_some())
        .ok_or_else(|| Error::NotFound {
            code: "execution_seal_missing".into(),
            message: "execution has no saved seal".into(),
        })?;
        let checkpoint = inspection.checkpoint(limit)?;
        inspection
            .view()
            .project_owned(checkpoint, limit, |checkpoint, allowance| {
                map(&checkpoint, allowance)
            })
            .map_err(Into::into)
    }

    pub async fn browse_executions<T: MemorySize>(
        &self,
        query: ExecutionBrowseQuery,
        map: impl FnOnce(&ExecutionBrowsePage, NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        let limit = query.limit.unwrap_or(100);
        if !(1..=1000).contains(&limit) {
            return Err(error("execution page limit must be 1..=1000"));
        }
        let config = self.config();
        let settings = &config.storage;
        let storage = self.storage();
        let view = storage.view_with_limits(settings.interactive).await?;
        let projection = settings.decoded_bytes;
        let mut reader = crate::runtime::storage::history::Reader::open(
            storage,
            &view,
            ReadLimits {
                // Execution Objects Include Structured Outputs; Read One Per Page.
                rows: NonZeroUsize::new(1).unwrap(),
                bytes: settings.decoded_bytes,
            },
            projection,
        )?;
        let mut rows = storage
            .produce(
                NonZeroUsize::new(std::mem::size_of::<BrowseRows>())
                    .expect("nonzero browse container"),
                |_| Ok(BrowseRows { pages: Vec::new() }),
            )?
            .try_into_value()?;
        while let Some(page) = reader.next().await? {
            let metadata = reader.project(&page, |rows, allowance| stored_rows(rows, allowance))?;
            drop(page);
            let next = reader.project(&rows, |rows, allowance| {
                let count = rows.pages.len().checked_add(1).ok_or_else(overflow)?;
                let bytes = std::mem::size_of::<BrowseRows>().saturating_add(count.saturating_mul(
                    std::mem::size_of::<SharedQueryValue<Vec<ExecutionBrowseEntry>>>(),
                ));
                require_memory(bytes, allowance)?;
                let mut pages = Vec::new();
                pages.try_reserve_exact(count).map_err(|_| exhausted())?;
                if pages.capacity() > count {
                    return Err(exhausted());
                }
                pages.extend(rows.pages.iter().cloned());
                pages.push(metadata.try_into_shared()?);
                Ok(BrowseRows { pages })
            })?;
            rows = next;
        }
        let live = reader.project(&rows, |_, allowance| {
            let active = self
                .execution_storage
                .active_ids(allowance)
                .map_err(projection_error)?;
            let state = self.state_read();
            let count = state
                .cycles
                .values()
                .map(|cycle| cycle.execution_ids.len())
                .try_fold(0usize, |count, length| {
                    count.checked_add(length).ok_or_else(overflow)
                })?;
            let mut bytes = std::mem::size_of::<Vec<ExecutionBrowseEntry>>()
                .saturating_add(count.saturating_mul(std::mem::size_of::<ExecutionBrowseEntry>()));
            for cycle in state.cycles.values() {
                for execution in state.executions_for_cycle(cycle) {
                    bytes = bytes.saturating_add(ExecutionMetadata::construction_memory(execution));
                }
            }
            require_memory(bytes, allowance)?;
            let mut live = Vec::new();
            live.try_reserve_exact(count).map_err(|_| exhausted())?;
            if live.capacity() > count {
                return Err(exhausted());
            }
            for cycle in state.cycles.values() {
                for execution in state.executions_for_cycle(cycle) {
                    live.push(ExecutionBrowseEntry {
                        metadata: ExecutionMetadata::new(execution),
                        checkpoint: None,
                        availability: if active.binary_search(&execution.test_execution_id).is_ok()
                        {
                            ExecutionAvailability::Live
                        } else {
                            ExecutionAvailability::Uncheckpointed
                        },
                        archived: false,
                    });
                }
            }
            Ok(live)
        })?;
        let archived = {
            let state = self.state_read();
            Archived {
                cycles: state.archived_cycles.clone(),
                executions: state.archived_executions.clone(),
            }
        };
        let page = reader.project(&rows, |rows, allowance| {
            browse_page(rows, &live, &query, &archived, limit, allowance)
        })?;
        reader.project(&page, |page, allowance| map(page, allowance))
    }
    pub async fn execution_checkpoint_context<T: MemorySize>(
        &self,
        checkpoint: ExecutionCheckpoint,
        map: impl FnOnce(ExecutionCheckpointContext, NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        let view = view.snapshot(self.storage()).await?;
        let limit = self
            .config()
            .storage
            .decoded_bytes
            .min(view.view().decoded_bytes()?);
        view.project_context(limit, |context, allowance| {
            let bytes = std::mem::size_of::<ExecutionCheckpointContext>()
                .saturating_add(checkpoint.memory_size())
                .saturating_add(context.memory_size());
            require_memory(bytes, allowance)?;
            let context = context.clone();
            let context = ExecutionCheckpointContext {
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
            };
            map(context, allowance)
        })
    }

    pub async fn execution_checkpoint_outputs_projected<T: MemorySize>(
        &self,
        checkpoint: ExecutionCheckpoint,
        map: impl FnOnce(
            &ExecutionCheckpoint,
            &[dokime::execution::saved::ExecutionOutput],
            NonZeroUsize,
        ) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        let limit = self
            .config()
            .storage
            .decoded_bytes
            .min(view.view().decoded_bytes()?);
        view.project_outputs(limit, |outputs, allowance| {
            map(&checkpoint, outputs, allowance)
        })
    }

    pub async fn execution_checkpoint_evidence<U: MemorySize>(
        &self,
        checkpoint: ExecutionCheckpoint,
        map: impl FnOnce(ExecutionCheckpointEvidence, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        let view = view.snapshot(self.storage()).await?;
        let outputs = view.outputs()?;
        let limit = self
            .config()
            .storage
            .decoded_bytes
            .min(view.view().decoded_bytes()?);
        view.project_source(limit, |captured, allowance| {
            let output_bytes = outputs.iter().fold(0usize, |bytes, output| {
                bytes
                    .saturating_add(output.memory_size())
                    .saturating_add(
                        std::mem::size_of::<crate::runtime::query::ExecutionFileEntry>(),
                    )
                    .saturating_add(checkpoint.execution_id.memory_size())
                    .saturating_add(output.relative_path.len().saturating_mul(2))
                    .saturating_add(output.file.metadata.media_type.len())
                    .saturating_add(output.created_at.memory_size())
            });
            let construction = std::mem::size_of::<ExecutionCheckpointEvidence>()
                .saturating_add(checkpoint.memory_size())
                .saturating_add(output_bytes.saturating_mul(2))
                .saturating_add(captured.record_contracts.memory_size().saturating_mul(2))
                .saturating_add(captured.telemetry_contracts.memory_size().saturating_mul(3));
            require_memory(construction, allowance)?;
            let evidence = ExecutionCheckpointEvidence {
                outputs: outputs
                    .iter()
                    .map(|output| {
                        crate::domains::documents::saved::entry(&checkpoint.execution_id, output)
                    })
                    .collect::<harmos::Result<Vec<_>>>()?,
                artifacts: outputs
                    .iter()
                    .filter_map(|output| match &output.content {
                        dokime::execution::saved::ExecutionOutputContent::Artifact {
                            record,
                            ..
                        } => Some(record.as_ref().clone()),
                        _ => None,
                    })
                    .collect(),
                record_contracts: captured
                    .record_contracts
                    .values()
                    .flat_map(|versions| versions.values().cloned())
                    .collect(),
                telemetry: captured
                    .telemetry_contracts
                    .values()
                    .flat_map(|versions| versions.values())
                    .map(
                        |contract| super::checkpoint_telemetry::ExecutionTelemetryTable {
                            table: contract.stream.to_string(),
                            contract: contract.clone(),
                        },
                    )
                    .collect(),
                checkpoint,
            };
            map(evidence, allowance)
        })
    }

    pub async fn execution_checkpoint_document<T: MemorySize>(
        &self,
        checkpoint: ExecutionCheckpoint,
        map: impl FnOnce(ExecutionCheckpointDocument, NonZeroUsize) -> harmos::Result<T>,
    ) -> Result<QueryValue<T>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        let view = view.snapshot(self.storage()).await?;
        let limit = self
            .config()
            .storage
            .decoded_bytes
            .min(view.view().decoded_bytes()?);
        view.project_context(limit, |context, allowance| {
            let execution = view.execution();
            let cycle = &context.cycle.cycle;
            let bytes = std::mem::size_of::<ExecutionCheckpointDocument>()
                .saturating_add(checkpoint.memory_size())
                .saturating_add(cycle.metadata.memory_size())
                .saturating_add(cycle.test_cycle_id.memory_size())
                .saturating_add(execution.memory_size());
            require_memory(bytes, allowance)?;
            let mut execution = execution.clone();
            execution.saved = None;
            let document = ExecutionCheckpointDocument {
                checkpoint,
                document: ExecutionDocument {
                    cycle: crate::execution::storage::ExecutionCycle {
                        cycle_id: cycle.test_cycle_id.clone(),
                        metadata: cycle.metadata.clone(),
                    },
                    execution,
                },
                acquisition_finished: true,
            };
            map(document, allowance)
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn stored_rows(
    rows: &[ObjectRevision<crate::execution::TestExecution>],
    allowance: NonZeroUsize,
) -> harmos::Result<Vec<ExecutionBrowseEntry>> {
    let mut bytes = std::mem::size_of::<Vec<ExecutionBrowseEntry>>().saturating_add(
        rows.len()
            .saturating_mul(std::mem::size_of::<ExecutionBrowseEntry>()),
    );
    for row in rows {
        bytes = bytes.saturating_add(ExecutionMetadata::construction_memory(&row.value));
        if let Some(seal) = &row.value.saved {
            bytes = bytes
                .saturating_add(row.value.test_execution_id.as_str().len())
                .saturating_add(row.position.memory_size())
                .saturating_add(row.version.memory_size())
                .saturating_add(seal.source.evidence_through.memory_size());
        }
    }
    require_memory(bytes, allowance)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(rows.len())
        .map_err(|_| exhausted())?;
    if entries.capacity() > rows.len() {
        return Err(exhausted());
    }
    for row in rows {
        let checkpoint = row.value.saved.as_ref().map(|seal| ExecutionCheckpoint {
            execution_id: row.value.test_execution_id.clone(),
            position: row.position.clone(),
            version: row.version.clone(),
            evidence_through: seal.source.evidence_through.clone(),
        });
        entries.push(ExecutionBrowseEntry {
            metadata: ExecutionMetadata::new(&row.value),
            availability: if checkpoint.is_some() {
                ExecutionAvailability::Stored
            } else {
                ExecutionAvailability::Interrupted
            },
            checkpoint,
            archived: false,
        });
    }
    Ok(entries)
}

fn browse_page(
    rows: &BrowseRows,
    live: &[ExecutionBrowseEntry],
    query: &ExecutionBrowseQuery,
    archived: &Archived,
    limit: usize,
    allowance: NonZeroUsize,
) -> harmos::Result<ExecutionBrowsePage> {
    let admitted = |row: &ExecutionBrowseEntry| {
        executions::matches(&row.metadata, &query.filter)
            && query.filter.archive.admits(archived.covers(&row.metadata))
    };
    let count = rows.pages.iter().try_fold(live.len(), |count, page| {
        count.checked_add(page.len()).ok_or_else(overflow)
    })?;
    let index = harmos::btree_map_memory::<&TestExecutionId, &ExecutionBrowseEntry>(count);
    let slots = count.saturating_mul(std::mem::size_of::<&ExecutionBrowseEntry>());
    let mut scratch = 0usize;
    for row in rows.pages.iter().flat_map(|page| page.iter()).chain(live) {
        scratch = scratch
            .max(executions::selection_memory(&row.metadata, &query.filter))
            .max(executions::ordering_memory(&row.metadata, query.order));
    }
    let checkpoints =
        harmos::btree_map_memory::<&TestExecutionId, Option<&ExecutionCheckpoint>>(count);
    let reserved = index
        .saturating_add(checkpoints)
        .saturating_add(slots)
        .saturating_add(scratch);
    require_memory(reserved, allowance)?;
    let mut candidates = BTreeMap::new();
    let mut checkpoints = BTreeMap::new();
    // Native Checkpoints Preserve Their Exact Revision Even When The Live Row Wins.
    for row in rows.pages.iter().flat_map(|page| page.iter()) {
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
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(candidates.len())
        .map_err(|_| exhausted())?;
    if selected.capacity() > count {
        return Err(exhausted());
    }
    selected.extend(candidates.into_values());
    selected.sort_unstable_by(|left, right| {
        executions::compare(&left.metadata, &right.metadata, query.order)
    });
    let start = match &query.after {
        None => 0,
        Some(after) => {
            selected
                .iter()
                .position(|entry| &entry.metadata.execution_id == after)
                .ok_or(harmos::Error::InvalidInput {
                    field: "execution continuation",
                    reason: "after execution ID is not in the filtered result",
                })?
                + 1
        }
    };
    let returned = selected.len().saturating_sub(start).min(limit);
    let more = selected.len().saturating_sub(start) > returned;
    let mut bytes = reserved
        .saturating_add(std::mem::size_of::<ExecutionBrowsePage>())
        .saturating_add(returned.saturating_mul(std::mem::size_of::<ExecutionBrowseEntry>()));
    for row in selected.iter().skip(start).take(returned) {
        bytes = bytes.saturating_add(row.memory_size());
        if let Some(Some(checkpoint)) = checkpoints.get(&row.metadata.execution_id) {
            bytes = bytes.saturating_add(checkpoint.memory_size());
        }
    }
    if more {
        bytes = bytes.saturating_add(
            selected[start + returned - 1]
                .metadata
                .execution_id
                .as_str()
                .len(),
        );
    }
    require_memory(bytes, allowance)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(returned)
        .map_err(|_| exhausted())?;
    if entries.capacity() > returned {
        return Err(exhausted());
    }
    for row in selected.into_iter().skip(start).take(returned) {
        let mut entry = (*row).clone();
        entry.archived = archived.covers(&entry.metadata);
        if entry.checkpoint.is_none() {
            entry.checkpoint = checkpoints
                .get(&entry.metadata.execution_id)
                .and_then(|checkpoint| checkpoint.cloned());
            if entry.checkpoint.is_some()
                && matches!(entry.availability, ExecutionAvailability::Uncheckpointed)
            {
                entry.availability = ExecutionAvailability::Stored;
            }
        }
        entries.push(entry);
    }
    let next = more.then(|| {
        entries
            .last()
            .expect("nonempty continued page")
            .metadata
            .execution_id
            .clone()
    });
    Ok(ExecutionBrowsePage {
        entries,
        next,
        discovery_complete: true,
        conflicts: Vec::new(),
        omitted_conflicts: 0,
        issues: Vec::new(),
        omitted_issues: 0,
    })
}

fn require_memory(bytes: usize, allowance: NonZeroUsize) -> harmos::Result<()> {
    if bytes == usize::MAX || bytes > allowance.get() {
        return Err(exhausted());
    }
    Ok(())
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "execution browse projection",
    }
}
fn overflow() -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "execution browse",
        reason: "memory overflows",
    }
}
fn projection_error(error: Error) -> harmos::Error {
    match error {
        Error::Harmos(error) => error,
        error => harmos::Error::backend(
            "project execution browse",
            harmos::BackendFailure::Corrupt,
            error,
        ),
    }
}

fn error(error: impl ToString) -> Error {
    Error::Operation(error.to_string())
}

/* === Tests === */
