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

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeSet;

// Third Party Imports
use harmos::{
    DefinitionId, Identified, MemorySize,
    storage::{
        FileReference, ObjectRetention, ObjectRevision, ObjectVersion, StorageCursor, StoredObject,
    },
};

// Local Imports
use super::{annotations, metadata};
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use crate::execution::saved::{CycleSeal, ExecutionRevision};
use crate::execution::{CycleStatus, CycleView, ResultStatus, ResultSummary, TestCycle};
use crate::storage::{Error, Manager, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Cycle {
    saved: Option<Seal>,
    cycle_id: String,
    session_id: Option<String>,
    metadata: Option<metadata::Row>,
    lifecycle: String,
    created_at: String,
    started_at: Option<String>,
    updated_at: String,
    execution_ids: Vec<String>,
    records: Option<annotations::Set>,
    continue_on_failure: bool,
    result: Option<Summary>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Summary {
    status: String,
    passed: u32,
    failed: u32,
    blocked: u32,
    skipped: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Seal {
    owner_node_id: String,
    evidence_through: Option<StorageCursor>,
    executions: Vec<super::execution::Revision>,
    report: Option<FileReference>,
    dependencies_saved: Option<StorageCursor>,
    sealed_at: String,
}

struct Input<'a> {
    saved: &'a Option<CycleSeal>,
    test_cycle_id: &'a TestCycleId,
    session_id: &'a Option<SessionId>,
    metadata: &'a dokime_domain::model::identity::Metadata,
    lifecycle: CycleStatus,
    created_at: &'a Timestamp,
    started_at: &'a Option<Timestamp>,
    updated_at: &'a Timestamp,
    execution_ids: &'a [TestExecutionId],
    records: &'a crate::records::TestRecordSet,
    continue_on_failure: bool,
    result: &'a Option<ResultSummary>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> From<&'a TestCycle> for Input<'a> {
    fn from(source: &'a TestCycle) -> Self {
        Self {
            saved: &source.saved,
            test_cycle_id: &source.test_cycle_id,
            session_id: &source.session_id,
            metadata: &source.metadata,
            created_at: &source.created_at,
            started_at: &source.started_at,
            updated_at: &source.updated_at,
            records: &source.records,
            result: &source.result,
            lifecycle: source.lifecycle,
            execution_ids: &source.execution_ids,
            continue_on_failure: source.continue_on_failure,
        }
    }
}
impl<'a> Input<'a> {
    fn view(source: &'a CycleView, ids: &'a [TestExecutionId]) -> Result<Self> {
        if !source.test_executions.is_empty() {
            return Err(Error::Invalid(
                "captured grouping contains joined executions",
            ));
        }
        Ok(Self::grouping(source, ids))
    }

    fn joined_view(source: &'a CycleView) -> Result<Self> {
        let [execution] = source.test_executions.as_slice() else {
            return Err(Error::Invalid(
                "captured report requires one joined execution",
            ));
        };
        if execution.test_cycle_id != source.test_cycle_id {
            return Err(Error::Invalid("captured report execution grouping differs"));
        }
        Ok(Self::grouping(
            source,
            std::slice::from_ref(&execution.test_execution_id),
        ))
    }

    fn grouping(source: &'a CycleView, ids: &'a [TestExecutionId]) -> Self {
        Self {
            saved: &source.saved,
            test_cycle_id: &source.test_cycle_id,
            session_id: &source.session_id,
            metadata: &source.metadata,
            created_at: &source.created_at,
            started_at: &source.started_at,
            updated_at: &source.updated_at,
            records: &source.records,
            result: &source.result,
            lifecycle: source.lifecycle,
            execution_ids: ids,
            continue_on_failure: source.continue_on_failure,
        }
    }
}

impl Identified for Cycle {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("dokime.object.cycle"), 1)
    }
}
impl StoredObject for Cycle {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::UntilReleased;
    fn key(&self) -> &str {
        &self.cycle_id
    }
}
impl Cycle {
    pub(crate) fn check_report_source<'a>(
        &self,
        owner: &str,
        finalized_at: Option<&str>,
        mut members: impl Iterator<Item = Result<&'a str>>,
    ) -> Result<()> {
        if self.saved.is_some()
            || self.cycle_id != owner
            || finalized_at.is_some_and(|at| self.updated_at != at)
        {
            return Err(Error::Invalid(
                "report header differs from frozen cycle source",
            ));
        }
        for member in &self.execution_ids {
            if members.next().transpose()? != Some(member.as_str()) {
                return Err(Error::Invalid(
                    "report header membership differs from sources",
                ));
            }
        }
        if members.next().is_some() {
            return Err(Error::Invalid(
                "report header membership differs from sources",
            ));
        }
        Ok(())
    }

    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        Ok(<Self as harmos::Message>::decoding_memory(bytes)?)
    }

    pub fn encoding_memory(source: &TestCycle) -> Result<usize> {
        Self::encoding_input(&Input::from(source))
    }

    pub fn membership_len(&self) -> usize {
        self.execution_ids.len()
    }

    pub fn encoding_view_memory(source: &CycleView, ids: &[TestExecutionId]) -> Result<usize> {
        Self::encoding_input(&Input::view(source, ids)?)
    }

    pub fn from_view(source: &CycleView, ids: &[TestExecutionId]) -> Result<Self> {
        Self::from_input(&Input::view(source, ids)?)
    }

    pub fn encoding_joined_view_memory(source: &CycleView) -> Result<usize> {
        Self::encoding_input(&Input::joined_view(source)?)
    }

    pub fn from_joined_view(source: &CycleView) -> Result<Self> {
        Self::from_input(&Input::joined_view(source)?)
    }

    fn encoding_input(source: &Input<'_>) -> Result<usize> {
        source.created_at.to_datetime()?;
        source.updated_at.to_datetime()?;
        if let Some(started) = &source.started_at {
            started.to_datetime()?;
        }
        let fields = [
            source
                .saved
                .as_ref()
                .map(Seal::encoding_memory)
                .transpose()?
                .unwrap_or(0),
            source.test_cycle_id.as_str().len(),
            source.session_id.as_ref().map_or(0, |id| id.as_str().len()),
            metadata::Row::encoding_memory(source.metadata)?,
            lifecycle_text(&source.lifecycle).len(),
            source.created_at.as_str().len(),
            source
                .started_at
                .as_ref()
                .map_or(0, |time| time.as_str().len()),
            source.updated_at.as_str().len(),
            annotations::Set::encoding_memory(source.records)?,
            source.result.as_ref().map_or(0, Summary::encoding_memory),
            harmos::btree_map_memory::<&TestExecutionId, ()>(source.execution_ids.len()),
            source
                .execution_ids
                .len()
                .saturating_mul(std::mem::size_of::<String>()),
        ];
        if fields.contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for id in source.execution_ids {
            bytes = bytes.saturating_add(id.as_str().len());
        }
        Ok(bytes)
    }
}

impl TryFrom<&TestCycle> for Cycle {
    type Error = Error;
    fn try_from(source: &TestCycle) -> Result<Self> {
        Self::from_input(&Input::from(source))
    }
}

impl Cycle {
    fn from_input(source: &Input<'_>) -> Result<Self> {
        if Self::encoding_input(source)? == usize::MAX {
            return Err(Error::Invalid("cycle encoding memory overflows"));
        }
        if let Some(saved) = &source.saved {
            saved.validate_membership(
                source.test_cycle_id,
                source.lifecycle,
                source.execution_ids.iter(),
            )?;
            saved.sealed_at.to_datetime()?;
        }
        let mut seen = BTreeSet::new();
        for id in source.execution_ids {
            if !seen.insert(id) {
                return Err(Error::Invalid("duplicate cycle execution membership"));
            }
        }

        let mut execution_ids = Vec::new();
        execution_ids
            .try_reserve_exact(source.execution_ids.len())
            .map_err(|_| Error::Invalid("cycle membership encoding allocation failed"))?;
        if execution_ids.capacity() > source.execution_ids.len() {
            return Err(Error::Invalid("cycle membership slots exceed admission"));
        }
        for id in source.execution_ids {
            execution_ids.push(id.as_str().to_owned());
        }
        let metadata = metadata::Row::try_from(source.metadata)?;
        let records = annotations::Set::try_from(source.records)?;
        Ok(Self {
            saved: source.saved.as_ref().map(Seal::try_from).transpose()?,
            cycle_id: source.test_cycle_id.as_str().to_owned(),
            session_id: source.session_id.as_ref().map(|id| id.as_str().to_owned()),
            metadata: Some(metadata),
            lifecycle: lifecycle_text(&source.lifecycle).to_owned(),
            created_at: source.created_at.as_str().to_owned(),
            started_at: source
                .started_at
                .as_ref()
                .map(|time| time.as_str().to_owned()),
            updated_at: source.updated_at.as_str().to_owned(),
            execution_ids,
            records: Some(records),
            continue_on_failure: source.continue_on_failure,
            result: source.result.as_ref().map(Summary::from),
        })
    }
}
impl TryFrom<Cycle> for TestCycle {
    type Error = Error;

    fn try_from(row: Cycle) -> Result<Self> {
        if row.cycle_id.bytes().any(|byte| byte.is_ascii_uppercase())
            || row
                .execution_ids
                .iter()
                .any(|id| id.bytes().any(|byte| byte.is_ascii_uppercase()))
        {
            return Err(Error::Invalid("noncanonical cycle or execution identity"));
        }
        let mut membership: Vec<_> = row.execution_ids.iter().map(String::as_str).collect();
        membership.sort_unstable();
        if membership.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::Invalid("duplicate cycle execution membership"));
        }
        drop(membership);

        let lifecycle = match row.lifecycle.as_str() {
            "ready" => CycleStatus::Ready,
            "running" => CycleStatus::Running,
            "paused" => CycleStatus::Paused,
            "stopping" => CycleStatus::Stopping,
            "completed" => CycleStatus::Completed,
            "failed" => CycleStatus::Failed,
            "blocked" => CycleStatus::Blocked,
            "stopped" => CycleStatus::Stopped,
            _ => return Err(Error::Invalid("invalid cycle lifecycle")),
        };
        let created_at = Timestamp::new(row.created_at);
        let updated_at = Timestamp::new(row.updated_at);
        let started_at = row.started_at.map(Timestamp::new);
        created_at.to_datetime()?;
        updated_at.to_datetime()?;
        if let Some(started) = &started_at {
            started.to_datetime()?;
        }
        let mut execution_ids = Vec::with_capacity(row.execution_ids.len());
        for id in row.execution_ids {
            execution_ids.push(TestExecutionId::from_uuid(id)?);
        }
        let restored = Self {
            saved: row.saved.map(TryInto::try_into).transpose()?,
            test_cycle_id: TestCycleId::from_uuid(row.cycle_id)?,
            session_id: row.session_id.map(SessionId::from_uuid).transpose()?,
            metadata: row
                .metadata
                .ok_or(Error::Invalid("cycle metadata is missing"))?
                .try_into()?,
            lifecycle,
            created_at,
            started_at,
            updated_at,
            execution_ids,
            records: row
                .records
                .ok_or(Error::Invalid("cycle record ownership is missing"))?
                .restore_owned()?,
            continue_on_failure: row.continue_on_failure,
            result: row.result.map(TryInto::try_into).transpose()?,
        };
        if let Some(saved) = &restored.saved {
            saved.validate(&restored)?;
            saved.sealed_at.to_datetime()?;
        }
        Ok(restored)
    }
}

impl Summary {
    pub(super) fn encoding_memory(value: &ResultSummary) -> usize {
        std::mem::size_of::<Self>().saturating_add(result_status_text(&value.status).len())
    }
}

impl From<&ResultSummary> for Summary {
    fn from(value: &ResultSummary) -> Self {
        Self {
            status: result_status_text(&value.status).to_owned(),
            passed: value.passed,
            failed: value.failed,
            blocked: value.blocked,
            skipped: value.skipped,
        }
    }
}

fn result_status_text(value: &ResultStatus) -> &'static str {
    match value {
        ResultStatus::Pending => "pending",
        ResultStatus::Passed => "passed",
        ResultStatus::Failed => "failed",
        ResultStatus::Blocked => "blocked",
        ResultStatus::Cancelled => "cancelled",
    }
}

impl TryFrom<Summary> for ResultSummary {
    type Error = Error;

    fn try_from(row: Summary) -> Result<Self> {
        let status = match row.status.as_str() {
            "pending" => ResultStatus::Pending,
            "passed" => ResultStatus::Passed,
            "failed" => ResultStatus::Failed,
            "blocked" => ResultStatus::Blocked,
            "cancelled" => ResultStatus::Cancelled,
            _ => return Err(Error::Invalid("invalid cycle result status")),
        };
        Ok(Self {
            status,
            passed: row.passed,
            failed: row.failed,
            blocked: row.blocked,
            skipped: row.skipped,
        })
    }
}

impl Seal {
    fn encoding_memory(value: &CycleSeal) -> Result<usize> {
        value.sealed_at.to_datetime()?;
        let fields = [
            value.owner_node_id.as_str().len(),
            value.evidence_through.memory_size(),
            value.report.memory_size(),
            value.dependencies_saved.memory_size(),
            value.sealed_at.as_str().len(),
        ];
        if fields.contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for revision in &value.executions {
            bytes = bytes.saturating_add(super::execution::Revision::encoding_memory(revision)?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&CycleSeal> for Seal {
    type Error = Error;
    fn try_from(value: &CycleSeal) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("cycle seal encoding memory overflows"));
        }
        let mut executions = Vec::new();
        executions
            .try_reserve_exact(value.executions.len())
            .map_err(|_| Error::Invalid("cycle seal members encoding allocation failed"))?;
        for revision in &value.executions {
            executions.push(super::execution::Revision::try_from(revision)?);
        }
        Ok(Self {
            owner_node_id: value.owner_node_id.as_str().to_owned(),
            evidence_through: Some(value.evidence_through.clone()),
            executions,
            report: Some(value.report.clone()),
            dependencies_saved: Some(value.dependencies_saved.clone()),
            sealed_at: value.sealed_at.as_str().to_owned(),
        })
    }
}

impl TryFrom<Seal> for CycleSeal {
    type Error = Error;

    fn try_from(row: Seal) -> Result<Self> {
        let mut executions = Vec::with_capacity(row.executions.len());
        for revision in row.executions {
            executions.push(revision.try_into()?);
        }
        Ok(Self {
            owner_node_id: NodeId::from_uuid(row.owner_node_id)?,
            evidence_through: row
                .evidence_through
                .ok_or(Error::Invalid("cycle seal evidence frontier is missing"))?,
            executions,
            report: row
                .report
                .ok_or(Error::Invalid("cycle seal report is missing"))?,
            dependencies_saved: row
                .dependencies_saved
                .ok_or(Error::Invalid("cycle seal saved frontier is missing"))?,
            sealed_at: Timestamp::new(row.sealed_at),
        })
    }
}

impl Seal {
    fn projection_memory(&self) -> Result<usize> {
        let evidence = self
            .evidence_through
            .as_ref()
            .ok_or(Error::Invalid("cycle seal evidence frontier is missing"))?;
        let report = self
            .report
            .as_ref()
            .ok_or(Error::Invalid("cycle seal report is missing"))?;
        let saved = self
            .dependencies_saved
            .as_ref()
            .ok_or(Error::Invalid("cycle seal saved frontier is missing"))?;
        if [
            evidence.memory_size(),
            report.memory_size(),
            saved.memory_size(),
        ]
        .contains(&usize::MAX)
        {
            return Ok(usize::MAX);
        }
        let fields = [
            self.owner_node_id.capacity(),
            self.sealed_at.capacity(),
            evidence
                .memory_size()
                .saturating_sub(std::mem::size_of_val(evidence)),
            report
                .memory_size()
                .saturating_sub(std::mem::size_of_val(report)),
            saved
                .memory_size()
                .saturating_sub(std::mem::size_of_val(saved)),
            self.executions
                .len()
                .saturating_mul(std::mem::size_of::<ExecutionRevision>()),
        ];
        let mut bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<CycleSeal>(), usize::saturating_add);
        for execution in &self.executions {
            let memory = execution.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(memory.saturating_sub(std::mem::size_of::<ExecutionRevision>()));
        }
        Ok(bytes)
    }
}

impl Cycle {
    pub fn restore(&self) -> Result<TestCycle> {
        self.clone().try_into()
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let metadata = self
            .metadata
            .as_ref()
            .ok_or(Error::Invalid("cycle metadata is missing"))?
            .projection_memory()?;
        let records = self
            .records
            .as_ref()
            .ok_or(Error::Invalid("cycle record ownership is missing"))?
            .projection_memory();
        let seal = self
            .saved
            .as_ref()
            .map(Seal::projection_memory)
            .transpose()?
            .unwrap_or(0);
        if [metadata, records, seal].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            self.cycle_id.capacity(),
            self.session_id.as_ref().map_or(0, String::capacity),
            self.created_at.capacity(),
            self.updated_at.capacity(),
            self.started_at.as_ref().map_or(0, String::capacity),
            metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>()),
            records.saturating_sub(std::mem::size_of::<crate::records::TestRecordSet>()),
            seal.saturating_sub(std::mem::size_of::<CycleSeal>()),
            self.execution_ids
                .len()
                .saturating_mul(std::mem::size_of::<TestExecutionId>()),
            self.execution_ids
                .len()
                .saturating_mul(std::mem::size_of::<&str>()),
        ];
        let bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<TestCycle>(), usize::saturating_add);
        Ok(self
            .execution_ids
            .iter()
            .fold(bytes, |bytes, id| bytes.saturating_add(id.capacity())))
    }
}

/* === Public Functions === */
pub async fn read(
    storage: &Manager,
    id: &TestCycleId,
    version: ObjectVersion,
) -> Result<Option<ObjectRevision<TestCycle>>> {
    let Some(stored) = storage.read_object::<Cycle>(id.as_str(), version).await? else {
        return Ok(None);
    };
    Ok(Some(ObjectRevision {
        value: stored.value.try_into()?,
        position: stored.position,
        version: stored.version,
    }))
}

/* === Private Functions === */
fn lifecycle_text(value: &CycleStatus) -> &'static str {
    match value {
        CycleStatus::Ready => "ready",
        CycleStatus::Running => "running",
        CycleStatus::Paused => "paused",
        CycleStatus::Stopping => "stopping",
        CycleStatus::Completed => "completed",
        CycleStatus::Failed => "failed",
        CycleStatus::Blocked => "blocked",
        CycleStatus::Stopped => "stopped",
    }
}

/* === Tests === */
