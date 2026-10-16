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
#[cfg(feature = "storage")]
pub use dokime_contract::execution::PageCursor;
pub use dokime_contract::execution::{ExecutionCheckpoint, SavedEvidenceError};

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeSet;
// Third Party Imports
use harmos::storage::{FileReference, ObjectVersionToken, StorageCursor};
use serde::{Deserialize, Serialize};
// Local Imports
use super::result::{CycleStatus, CycleView, ExecutionStatus, TestCycle, TestExecution};
use crate::artifacts::{ArtifactRecord, ArtifactStatus};
use crate::core::{Metadata, NodeId, TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */
pub(crate) const MAX_OUTPUTS: usize = 256;
pub const EXECUTION_CONTEXT_MEDIA_TYPE: &str = "application/vnd.dokime.execution-context+protobuf";
pub const EXECUTION_CONTEXT_SCHEMA_VERSION: u32 = 1;
pub const MATERIALIZED_REPORT_INPUT_MEDIA_TYPE: &str =
    "application/vnd.dokime.materialized-report-input+json";
pub const MATERIALIZED_REPORT_INPUT_SCHEMA_VERSION: u32 = 1;
pub const MATERIALIZED_REPORT_INPUT_SUFFIX: &str = ".materialized-input";

/* === Macros === */
macro_rules! owned_memory {
    ($type:ty, $($field:ident),+ $(,)?) => {
        impl harmos::MemorySize for $type {
            fn memory_size(&self) -> usize {
                let mut bytes = std::mem::size_of::<Self>();
                $(
                    let owned = harmos::MemorySize::memory_size(&self.$field);
                    if owned == usize::MAX {
                        return usize::MAX;
                    }
                    bytes = bytes.saturating_add(
                        owned.saturating_sub(std::mem::size_of_val(&self.$field)),
                    );
                )+
                bytes
            }
        }
    };
}

/* === Type Aliases === */
pub type Result<T> = std::result::Result<T, SavedEvidenceError>;

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRevision {
    pub execution_id: TestExecutionId,
    pub position: StorageCursor,
    pub version: ObjectVersionToken,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSource {
    pub owner_node_id: NodeId,
    pub evidence_through: StorageCursor,
    pub execution: ExecutionRevision,
    pub context: FileReference,
    pub outputs: Vec<ExecutionOutput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionOutput {
    pub relative_path: String,
    pub created_at: Timestamp,
    pub file: FileReference,
    pub stream_state: Option<SavedStreamState>,
    pub content: ExecutionOutputContent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSeal {
    pub source: ExecutionSource,
    pub materialized_input: FileReference,
    pub report: FileReference,
    pub dependencies_saved: StorageCursor,
    pub sealed_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedReplicaFile {
    pub source: FileReference,
    pub local: FileReference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleSeal {
    pub owner_node_id: NodeId,
    pub evidence_through: StorageCursor,
    pub executions: Vec<ExecutionRevision>,
    pub report: FileReference,
    pub dependencies_saved: StorageCursor,
    pub sealed_at: Timestamp,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionOutputContent {
    File,
    Artifact {
        record: Box<ArtifactRecord>,
        metadata: Metadata,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedStreamState {
    Finalized,
    Interrupted,
}

/* === Traits === */

/* === Implementations === */
owned_memory!(ExecutionRevision, execution_id, position, version);
owned_memory!(
    ExecutionSource,
    owner_node_id,
    evidence_through,
    execution,
    context,
    outputs,
);
owned_memory!(
    ExecutionOutput,
    relative_path,
    created_at,
    file,
    stream_state,
    content
);
owned_memory!(
    ExecutionSeal,
    source,
    materialized_input,
    report,
    dependencies_saved,
    sealed_at
);
owned_memory!(VerifiedReplicaFile, source, local);
owned_memory!(
    CycleSeal,
    owner_node_id,
    evidence_through,
    executions,
    report,
    dependencies_saved,
    sealed_at,
);

impl harmos::MemorySize for ExecutionOutputContent {
    fn memory_size(&self) -> usize {
        let Self::Artifact { record, metadata } = self else {
            return std::mem::size_of::<Self>();
        };
        let fields = [
            (record.memory_size(), std::mem::size_of_val(record)),
            (metadata.memory_size(), std::mem::size_of::<Metadata>()),
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

impl harmos::MemorySize for SavedStreamState {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl ExecutionSource {
    pub fn validate(&self, id: &TestExecutionId) -> Result<()> {
        if &self.execution.execution_id != id || self.execution.position.sequence == 0 {
            return Err(SavedEvidenceError::Invalid(
                "source execution identity differs",
            ));
        }
        self.execution
            .version
            .check_position(&self.execution.position)?;
        self.evidence_through
            .history
            .check(&self.execution.position.history)?;
        check_file(&self.context, id)?;
        self.evidence_through
            .history
            .check(&self.context.position.history)?;
        if self.context.position.sequence <= self.evidence_through.sequence
            || self.context.position.sequence <= self.execution.position.sequence
        {
            return Err(SavedEvidenceError::Invalid(
                "execution context must follow fixed evidence",
            ));
        }
        if self.context.metadata.media_type != EXECUTION_CONTEXT_MEDIA_TYPE {
            return Err(SavedEvidenceError::Invalid(
                "context must use the native execution context media type",
            ));
        }
        validate_outputs(id, &self.evidence_through, &self.outputs)?;
        if self
            .outputs
            .iter()
            .any(|output| output.file.metadata.id == self.context.metadata.id)
        {
            return Err(SavedEvidenceError::Invalid(
                "context reuses an output file identity",
            ));
        }
        Ok(())
    }
}

impl ExecutionSeal {
    pub fn validate(&self, execution: &TestExecution) -> Result<()> {
        if matches!(
            execution.status,
            ExecutionStatus::Pending | ExecutionStatus::Running
        ) || execution.finished_at.is_none()
            || execution.active_step_attempt.is_some()
        {
            return Err(SavedEvidenceError::Invalid("execution has not settled"));
        }
        self.source.validate(&execution.test_execution_id)?;
        validate_materialized_input(
            &self.materialized_input,
            &execution.test_execution_id,
            &self.source.context,
            &self.dependencies_saved,
        )?;
        if self
            .files()
            .filter(|file| file.metadata.id == self.materialized_input.metadata.id)
            .count()
            != 1
        {
            return Err(SavedEvidenceError::Invalid(
                "materialized input reuses a source file identity",
            ));
        }
        check_file(&self.report, &execution.test_execution_id)?;
        if self.report.metadata.media_type != "text/html" || self.report.length == 0 {
            return Err(SavedEvidenceError::Invalid(
                "final report must be nonempty HTML",
            ));
        }
        if self
            .files()
            .filter(|file| file.metadata.id == self.report.metadata.id)
            .count()
            != 1
        {
            return Err(SavedEvidenceError::Invalid(
                "report reuses a source file identity",
            ));
        }
        require_saved(&self.dependencies_saved, &self.source.evidence_through)?;
        require_saved(&self.dependencies_saved, &self.source.execution.position)?;
        require_saved(&self.dependencies_saved, &self.source.context.position)?;
        require_saved(&self.dependencies_saved, &self.report.position)?;
        if self.report.position.sequence <= self.source.context.position.sequence {
            return Err(SavedEvidenceError::Invalid(
                "report must follow frozen execution context",
            ));
        }
        Ok(())
    }

    pub fn files(&self) -> impl Iterator<Item = &FileReference> {
        std::iter::once(&self.source.context)
            .chain(self.source.outputs.iter().map(|output| &output.file))
            .chain(std::iter::once(&self.materialized_input))
            .chain(std::iter::once(&self.report))
    }

    pub fn check_replica_dependencies(
        &self,
        execution: &TestExecution,
        source_received: &StorageCursor,
        local_data_admitted: &StorageCursor,
        local_saved: &StorageCursor,
        verified: &[VerifiedReplicaFile],
    ) -> Result<()> {
        self.validate(execution)?;
        require_saved(source_received, &self.dependencies_saved)?;
        require_saved(local_saved, local_data_admitted)?;
        if verified.len() != self.source.outputs.len() + 3 {
            return Err(SavedEvidenceError::Invalid("replica file set differs"));
        }
        for required in self.files() {
            let mut matches = verified.iter().filter(|file| &file.source == required);
            let received = matches.next().ok_or(SavedEvidenceError::Invalid(
                "replica is missing an exact source file",
            ))?;
            if matches.next().is_some()
                || received.local.metadata.id != required.metadata.id
                || received.local.metadata.scope != required.metadata.scope
                || received.local.metadata.media_type != required.metadata.media_type
                || received.local.length != required.length
                || received.local.sha256 != required.sha256
            {
                return Err(SavedEvidenceError::Invalid(
                    "replica file identity or verified integrity differs",
                ));
            }
            check_file(&received.local, &execution.test_execution_id)?;
            require_saved(local_saved, &received.local.position)?;
        }
        Ok(())
    }
}

impl CycleSeal {
    pub fn validate(&self, cycle: &TestCycle) -> Result<()> {
        self.validate_membership(
            &cycle.test_cycle_id,
            cycle.lifecycle,
            cycle.execution_ids.iter(),
        )
    }

    pub fn validate_view(&self, cycle: &CycleView) -> Result<()> {
        if cycle
            .test_executions
            .iter()
            .any(|execution| execution.test_cycle_id != cycle.test_cycle_id)
        {
            return Err(SavedEvidenceError::Invalid(
                "execution belongs to another cycle",
            ));
        }
        self.validate_membership(
            &cycle.test_cycle_id,
            cycle.lifecycle,
            cycle
                .test_executions
                .iter()
                .map(|execution| &execution.test_execution_id),
        )
    }

    pub(crate) fn validate_membership<'a>(
        &self,
        cycle_id: &TestCycleId,
        lifecycle: CycleStatus,
        ids: impl ExactSizeIterator<Item = &'a TestExecutionId>,
    ) -> Result<()> {
        validate_cycle_members(lifecycle, ids, &self.evidence_through, &self.executions)?;
        if self
            .report
            .metadata
            .scope
            .as_ref()
            .map(|scope| scope.as_str())
            != Some(cycle_id.as_str())
            || self.report.metadata.media_type != "text/html"
            || self.report.length == 0
            || self
                .report
                .metadata
                .expected_length
                .is_some_and(|length| length != self.report.length)
            || self
                .report
                .metadata
                .expected_sha256
                .is_some_and(|hash| hash != self.report.sha256)
        {
            return Err(SavedEvidenceError::Invalid(
                "cycle final report identity or integrity differs",
            ));
        }
        require_saved(&self.dependencies_saved, &self.evidence_through)?;
        require_saved(&self.dependencies_saved, &self.report.position)?;
        if self.report.position.sequence <= self.evidence_through.sequence {
            return Err(SavedEvidenceError::Invalid(
                "cycle report must follow member evidence",
            ));
        }
        Ok(())
    }
}

/* === Public Functions === */
pub fn validate_cycle_revisions(
    cycle: &TestCycle,
    evidence_through: &StorageCursor,
    revisions: &[ExecutionRevision],
) -> Result<()> {
    validate_cycle_members(
        cycle.lifecycle,
        cycle.execution_ids.iter(),
        evidence_through,
        revisions,
    )
}

pub fn output_validation_memory(outputs: &[ExecutionOutput]) -> usize {
    let artifacts = outputs
        .iter()
        .filter(|output| matches!(&output.content, ExecutionOutputContent::Artifact { .. }))
        .count();
    harmos::btree_map_memory::<&String, ()>(outputs.len())
        .saturating_add(harmos::btree_map_memory::<&str, ()>(outputs.len()))
        .saturating_add(harmos::btree_map_memory::<&crate::core::ArtifactId, ()>(
            artifacts,
        ))
}

pub fn validate_outputs(
    id: &TestExecutionId,
    through: &StorageCursor,
    outputs: &[ExecutionOutput],
) -> Result<()> {
    if outputs.len() > MAX_OUTPUTS {
        return Err(SavedEvidenceError::Invalid("too many execution outputs"));
    }
    let mut paths = BTreeSet::new();
    let mut files = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for output in outputs {
        output.created_at.to_datetime()?;
        validate_output_path(&output.relative_path)?;
        if !paths.insert(&output.relative_path) || !files.insert(output.file.metadata.id.as_str()) {
            return Err(SavedEvidenceError::Invalid("duplicate output identity"));
        }
        if let ExecutionOutputContent::Artifact { record, metadata } = &output.content {
            record.created_at.to_datetime()?;
            if !artifacts.insert(&record.artifact_id)
                || record.scope.test_execution_id.as_ref() != Some(id)
                || metadata.identifier.as_str() != record.artifact_id.as_str()
                || record.created_at != output.created_at
                || record.size_bytes != output.file.length
                || record.status == ArtifactStatus::Staged
                || !artifact_digest_matches(record.sha256.as_deref(), &output.file.sha256)
            {
                return Err(SavedEvidenceError::Invalid(
                    "saved artifact differs from its original owner or finished bytes",
                ));
            }
            let expected = match record.status {
                ArtifactStatus::Complete => SavedStreamState::Finalized,
                ArtifactStatus::Cancelled | ArtifactStatus::Rejected => {
                    SavedStreamState::Interrupted
                }
                ArtifactStatus::Staged => {
                    return Err(SavedEvidenceError::Invalid(
                        "saved artifact has not settled",
                    ));
                }
            };
            if output.stream_state != Some(expected)
                || output.file.metadata.media_type
                    != record
                        .content_type
                        .as_deref()
                        .unwrap_or("application/octet-stream")
            {
                return Err(SavedEvidenceError::Invalid(
                    "saved artifact attribution differs from its output descriptor",
                ));
            }
        }
        check_file(&output.file, id)?;
        require_saved(through, &output.file.position)?;
    }
    Ok(())
}

pub fn validate_output_path(path: &str) -> Result<()> {
    validate_relative_output_path(path)?;
    if path == "reports/final.html" {
        return Err(SavedEvidenceError::Invalid(
            "reports/final.html is reserved for the final report",
        ));
    }
    Ok(())
}

pub fn validate_relative_output_path(relative_path: &str) -> Result<()> {
    if !crate::artifacts::name::is_relative_path(relative_path) {
        return Err(SavedEvidenceError::Invalid("invalid execution output path"));
    }
    Ok(())
}

/* === Private Functions === */
fn validate_cycle_members<'a>(
    lifecycle: CycleStatus,
    ids: impl ExactSizeIterator<Item = &'a TestExecutionId>,
    evidence_through: &StorageCursor,
    revisions: &[ExecutionRevision],
) -> Result<()> {
    if !matches!(
        lifecycle,
        CycleStatus::Completed | CycleStatus::Failed | CycleStatus::Blocked | CycleStatus::Stopped
    ) || revisions.len() != ids.len()
    {
        return Err(SavedEvidenceError::Invalid(
            "cycle has not settled over exact membership",
        ));
    }
    for (revision, id) in revisions.iter().zip(ids) {
        if &revision.execution_id != id || revision.position.sequence == 0 {
            return Err(SavedEvidenceError::Invalid(
                "cycle member order or identity differs",
            ));
        }
        revision.version.check_position(&revision.position)?;
        require_saved(evidence_through, &revision.position)?;
    }
    Ok(())
}

fn artifact_digest_matches(text: Option<&str>, digest: &[u8; 32]) -> bool {
    let Some(text) = text.filter(|text| text.len() == 64) else {
        return false;
    };
    const HEX: &[u8; 16] = b"0123456789abcdef";
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .zip(digest)
        .all(|(pair, byte)| {
            pair[0] == HEX[(byte >> 4) as usize] && pair[1] == HEX[(byte & 15) as usize]
        })
}

fn require_saved(saved: &StorageCursor, dependency: &StorageCursor) -> Result<()> {
    if !saved.covers(dependency)? {
        return Err(SavedEvidenceError::Invalid(
            "dependency exceeds saved boundary",
        ));
    }
    Ok(())
}

pub(crate) fn validate_materialized_input(
    file: &FileReference,
    id: &TestExecutionId,
    context: &FileReference,
    dependencies: &StorageCursor,
) -> Result<()> {
    check_file(file, id)?;
    let prefix = format!("{}.", id.as_str());
    let token = file
        .metadata
        .id
        .as_str()
        .strip_prefix(&prefix)
        .and_then(|value| value.strip_suffix(MATERIALIZED_REPORT_INPUT_SUFFIX))
        .ok_or(SavedEvidenceError::Invalid(
            "materialized input file identity differs",
        ))?;
    if token.len() != 36
        || TestExecutionId::from_uuid(token).is_err()
        || token.bytes().any(|byte| byte.is_ascii_uppercase())
        || file.metadata.media_type != MATERIALIZED_REPORT_INPUT_MEDIA_TYPE
        || file.length == 0
    {
        return Err(SavedEvidenceError::Invalid(
            "materialized input identity or media type differs",
        ));
    }
    require_saved(dependencies, &file.position)?;
    file.position.history.check(&context.position.history)?;
    if file.position.sequence <= context.position.sequence {
        return Err(SavedEvidenceError::Invalid(
            "materialized input must follow frozen execution context",
        ));
    }
    Ok(())
}

fn check_file(file: &FileReference, id: &TestExecutionId) -> Result<()> {
    if file.position.sequence == 0
        || file.metadata.scope.as_ref().map(|scope| scope.as_str()) != Some(id.as_str())
        || file
            .metadata
            .expected_length
            .is_some_and(|length| length != file.length)
        || file
            .metadata
            .expected_sha256
            .is_some_and(|hash| hash != file.sha256)
    {
        return Err(SavedEvidenceError::Invalid(
            "file identity or integrity differs",
        ));
    }
    Ok(())
}

/* === Tests === */
