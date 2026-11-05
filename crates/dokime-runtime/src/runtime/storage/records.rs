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

// Local Imports
use super::{context_records::ContextRecord, execution::ExecutionView};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::CaseId;
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
    local: crate::storage::sqlite::State,
    after: u64,
    context: bool,
    view: ExecutionView,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RecordReader {
    pub(crate) fn open(view: &ExecutionView, local: &crate::storage::sqlite::State) -> Result<Self> {
        let mut reader = Self::selected(view, local)?;
        reader.context = true;
        Ok(reader)
    }

    pub(crate) fn selected(view: &ExecutionView, local: &crate::storage::sqlite::State) -> Result<Self> {
        if !view.saved().covers(view.evidence_through()?)? {
            return Err(invalid("record evidence exceeds the original saved view"));
        }
        Ok(Self {
            local: local.clone(),
            after: 0,
            context: false,
            view: view.clone(),
        })
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
        let execution = self.view.execution();
        let at = self.view.evidence_through()?.clone();
        let rows = if self.context {
            self.local.context_records(at, execution.test_execution_id.to_string(), execution.test_cycle_id.to_string(), self.after).await?
        } else {
            let scope = crate::evidence_runtime::ScopeSelection {
                test_execution_id: Some(execution.test_execution_id.clone()),
                ..Default::default()
            };
            self.local.scoped_records(at, scope, None, self.after, 0, u64::MAX).await?
        };
        let Some(last) = rows.last() else { return Ok(None) };
        self.after = last.position;
        let selected = harmos::storage::ReadBatch::new(rows).try_map(|rows| -> Result<_> {
            let mut selected = Vec::new();
            selected.try_reserve_exact(rows.len()).map_err(|_| invalid("record projection allocation failed"))?;
            for row in rows {
                let mut record = row.record;
                apply_execution_scope(
                    &mut record,
                    &execution.records,
                    &execution.test_cycle_id,
                    &execution.test_case_id,
                    &execution.test_execution_id,
                )?;
                selected.push(PositionedRecord {
                    position: row.position,
                    captured: ContextRecord { epoch: row.epoch, record },
                });
            }
            Ok(selected)
        })?;
        Ok(Some(selected))
    }
}

/* === Public Functions === */

/* === Private Functions === */

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
