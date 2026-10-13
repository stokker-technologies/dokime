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
use std::io::{self, Write};
// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::execution::{CycleStatus, CycleView, ExecutionStatus};
use crate::records::TestRecordSet;
use crate::state::RuntimeState;
use dokime::core::{TestCycleId, TestExecutionId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.execution.cycle.copy", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CopyCycle {
    pub source_cycle_id: TestCycleId,
    pub test_cycle_id: TestCycleId,
    pub occurred_at: Timestamp,
}

// Walk the existing borrowed Serde representation without allocating a payload.
// Per-write overhead bounds container/scalar ownership as well as encoded bytes.
struct Budget {
    remaining: usize,
    depth: usize,
    quoted: bool,
    escaped: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for CopyCycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        super::ensure_runtime_active(state)?;
        if state.cycles.contains_key(&self.test_cycle_id) {
            return Err("target cycle already exists".into());
        }
        let source = state
            .cycle_view(&self.source_cycle_id)
            .ok_or("source cycle not found")?;
        if source.test_executions.is_empty() || source.test_executions.len() > 1000 {
            return Err("cycle copy requires 1 to 1000 executions".into());
        }
        let mut budget = Budget {
            remaining: 8 * 1024 * 1024,
            depth: 0,
            quoted: false,
            escaped: false,
        };
        budget.source(&source.metadata)?;
        for execution in &source.test_executions {
            budget.source(&execution.snapshot)?;
        }
        Ok(())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        self.drafts(state).collect(out);
    }

    fn apply(&self, state: &mut RuntimeState) {
        let records = self.drafts(state);
        let source = state
            .cycle_view(&self.source_cycle_id)
            .expect("checked source cycle");
        let mut cycle = CycleView {
            saved: None,
            test_cycle_id: self.test_cycle_id.clone(),
            session_id: Some(state.session_id.clone()),
            metadata: source.metadata.clone(),
            lifecycle: CycleStatus::Ready,
            created_at: self.occurred_at.clone(),
            started_at: None,
            updated_at: self.occurred_at.clone(),

            test_executions: Vec::new(),
            records: TestRecordSet::default(),
            continue_on_failure: source.continue_on_failure,
            result: None,
        };
        for (ordinal, original) in source.test_executions.iter().enumerate() {
            let id = TestExecutionId::from_seed(format!(
                "cycle:{}:source-execution:{}:ordinal:{ordinal}",
                cycle.test_cycle_id, original.test_execution_id
            ));
            let mut execution = crate::transactions::provider::scheduled_execution_record(
                &cycle.test_cycle_id,
                id,
                original
                    .snapshot
                    .clone()
                    .with_host_version(Some(env!("CARGO_PKG_VERSION").to_owned())),
            );
            execution.status = ExecutionStatus::Skipped;
            execution.documents = execution.snapshot.documents().clone();
            cycle.test_executions.push(execution);
        }
        state
            .replace_cycle(cycle)
            .expect("cycle projection has valid ownership");
        records.apply(state);
    }
}

impl Budget {
    fn source(&mut self, value: &impl Serialize) -> Result<(), String> {
        serde_json::to_writer(self, value).map_err(|error| error.to_string())
    }
}

impl Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining = self
            .remaining
            .checked_sub(bytes.len().saturating_add(64))
            .ok_or_else(|| io::Error::other("cycle copy exceeds 8 MiB source budget"))?;
        for byte in bytes {
            if self.quoted {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.quoted = false;
                }
            } else {
                match byte {
                    b'"' => self.quoted = true,
                    b'{' | b'[' => {
                        self.depth += 1;
                        if self.depth > 64 {
                            return Err(io::Error::other("cycle copy exceeds depth 64"));
                        }
                    }
                    b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl CopyCycle {
    fn drafts(&self, state: &RuntimeState) -> crate::transactions::records::draft::Drafts {
        use crate::records::{ProductRecordDraft, RecordSeverity};
        use dokime::core::{Identifier, RecordScope, Value};
        let mut records = crate::transactions::records::draft::Drafts::new(state);
        records.draft(
            ProductRecordDraft {
                record_type: Identifier::new("test.cycle.prepared").expect("built-in identifier"),
                source: "runtime".into(),
                severity: RecordSeverity::Info,
                message: Some("Cycle plan copied; ready for execution.".into()),
                details: std::collections::BTreeMap::from([
                    ("lifecycle".into(), Value::from("Ready")),
                    (
                        "source_cycle_id".into(),
                        Value::from(self.source_cycle_id.to_string()),
                    ),
                ]),
                scope: Some(RecordScope::default().with_cycle(self.test_cycle_id.clone())),
            },
            self.occurred_at.clone(),
        );
        records
    }
}

/* === Public Functions === */

/* === Private Functions === */
/* === Tests === */
