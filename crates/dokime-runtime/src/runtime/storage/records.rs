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

// Third Party Imports

use harmos::storage::{RecordRead, StoredRecord, query::RecordSelection};

// Local Imports
use super::{context_records::ContextRecord, execution::ExecutionView};
use crate::storage::records;
use crate::storage::records::{RecordRow, ScopeRow};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::state::records::set::TestRecordSet;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(super) type PositionedRecord = super::paging::Positioned<ContextRecord>;

/* === Structs === */

pub struct RecordReader {
    rows: harmos::storage::RecordReader<RecordRow>,
    view: ExecutionView,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RecordReader {
    pub(crate) async fn open(view: &ExecutionView) -> Result<Self> {
        let through = view.evidence_through()?;
        if !view.saved().covers(through)? {
            return Err(invalid("record evidence exceeds the original saved view"));
        }
        let rows = view
            .session()
            .read_records::<RecordRow>(RecordRead {
                at: through.clone(),
                selection: RecordSelection {
                    schema: records::schema(),
                    scope: None,
                    item: None,
                    source_start: None,
                    source_end: None,
                },
                after: 0,
            })
            .await?;
        Ok(Self { rows, view: view.clone() })
    }

    pub async fn next(&mut self) -> Result<Option<harmos::storage::ReadBatch<Vec<ContextRecord>>>> {
        let Some(rows) = self.next_positioned().await? else {
            return Ok(None);
        };
        let selected = rows.try_map(|rows| -> Result<_> {
            let mut selected = Vec::new();
            selected.try_reserve_exact(rows.len()).map_err(|_| invalid("record projection allocation failed"))?;
            selected.extend(rows.into_iter().map(|row| row.captured));
            Ok(selected)
        })?;
        Ok(Some(selected))
    }

    pub(super) async fn next_positioned(&mut self) -> Result<Option<harmos::storage::ReadBatch<Vec<PositionedRecord>>>> {
        loop {
            let Some(rows) = self.rows.next().await? else {
                return Ok(None);
            };
            let selected = rows.try_map(|rows| self.select(rows))?;
            if !selected.is_empty() {
                return Ok(Some(selected));
            }
        }
    }

    // One Page's Rows Owned By The Execution, Or Its Shared Context When Requested.
    fn select(&self, rows: Vec<StoredRecord<RecordRow>>) -> Result<Vec<PositionedRecord>> {
        let execution = self.view.execution();

        let mut selected = Vec::with_capacity(rows.len());
        for row in rows {
            let native = owned(&row, execution);
            let position = row.position.sequence;
            let mut decoded = decode(row)?;
            if !native && execution.records.captured_scope(&decoded.record).is_none() {
                continue;
            }
            apply_execution_scope(
                &mut decoded.record,
                &execution.records,
                &execution.test_cycle_id,
                &execution.test_case_id,
                &execution.test_execution_id,
            )?;
            selected.push(PositionedRecord { position, captured: decoded });
        }
        Ok(selected)
    }
}

/* === Public Functions === */

/* === Private Functions === */

// Report Context Includes The Execution's Own Records, Its Cycle's Unowned
// Records And Records Without Any Test Owner.
fn owned(row: &StoredRecord<RecordRow>, execution: &TestExecution) -> bool {
    let scope = row.payload.scope();
    let field = |select: fn(&ScopeRow) -> &Option<String>| scope.and_then(|scope| select(scope).as_deref());
    let owner = field(|scope| &scope.test_execution_id);
    if owner == Some(execution.test_execution_id.as_str()) {
        return true;
    }
    if owner.is_some() {
        return false;
    }
    field(|scope| &scope.test_cycle_id) == Some(execution.test_cycle_id.as_str())
        || [
            field(|scope| &scope.test_cycle_id),
            field(|scope| &scope.test_case_id),
            field(|scope| &scope.step_id),
            field(|scope| &scope.criterion_id),
        ]
        .iter()
        .all(Option::is_none)
}

fn decode(row: StoredRecord<RecordRow>) -> Result<ContextRecord> {
    let epoch = row.scope.as_ref().ok_or_else(|| invalid("captured record has no process epoch"))?;
    if epoch.as_str().bytes().any(|byte| byte.is_ascii_uppercase()) || row.source.is_some() || row.source_time.is_some() {
        return Err(invalid("captured record envelope differs from its declared schema"));
    }
    let epoch = SessionId::from_uuid(epoch.as_str()).map_err(crate::storage::Error::from)?;
    let record = row.payload.entry()?;
    if row.item.as_str() != record.record_id.as_str() {
        return Err(invalid("captured record key differs from payload identity"));
    }
    Ok(ContextRecord { epoch, record })
}

fn apply_execution_scope(
    record: &mut RecordEntry, captures: &TestRecordSet, cycle: &TestCycleId, case: &CaseId, execution: &TestExecutionId,
) -> Result<()> {
    if let Some(scope) = captures.captured_scope(record).cloned() {
        if scope.test_execution_id.as_ref() != Some(execution)
            || scope.test_cycle_id.as_ref().is_some_and(|id| id != cycle)
            || scope.test_case_id.as_ref().is_some_and(|id| id != case)
        {
            return Err(invalid("frozen record subscription belongs to another execution"));
        }
        record.scope = Some(scope);
    }
    let Some(scope) = record.scope.as_ref() else {
        return Ok(());
    };
    if scope.test_execution_id.as_ref() != Some(execution) {
        return Ok(());
    }
    if scope.test_cycle_id.as_ref().is_some_and(|id| id != cycle) || scope.test_case_id.as_ref().is_some_and(|id| id != case) {
        return Err(invalid("captured record belongs to another cycle or case"));
    }
    Ok(())
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */
