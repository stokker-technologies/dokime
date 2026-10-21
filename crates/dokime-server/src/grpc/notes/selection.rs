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
use sha2::{Digest, Sha256};
use tonic::Status;

// Local Imports
use dokime::domain::model::execution::RecordScope;
use dokime::domain::model::execution::notes::NoteOwner;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::runtime::query::RuntimeStateSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Page {
    pub owner: Option<NoteOwner>,
    pub context: bool,
    pub limit: usize,
    pub after: String,
    pub cursor: Cursor,
    scope: Option<RecordScope>,
    case_executions: std::collections::BTreeSet<dokime::domain::model::identity::TestExecutionId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Page {
    pub fn new(
        selection: Option<dokime::domain::model::notes::wire::AnnotationSelection>, limit: u32, after_id: String,
        given_cursor: Option<Cursor>, snapshot: &RuntimeStateSnapshot, kind: &str,
    ) -> Result<Self, Status> {
        let mut selection =
            selection.unwrap_or(dokime::domain::model::notes::wire::AnnotationSelection { owner: None, include_context: false });
        let owner = selection.owner.take().map(|owner| super::wire::owner(Some(owner))).transpose()?;
        selection.owner = owner.as_ref().map(super::wire::encode_owner);
        if owner.is_none() {
            selection.include_context = false;
        }
        let stream = format!(
            "{kind}:{}",
            Sha256::digest(dokime::harmos::Message::encode_to_vec(&selection)).iter().map(|byte| format!("{byte:02x}")).collect::<String>()
        );
        let cursor = Cursor { epoch: snapshot.state().process_epoch().to_string(), stream, position: snapshot.revision() };
        if limit > 100 {
            return Err(Status::invalid_argument("page limit exceeds 100"));
        }
        if after_id.len() > 64 {
            return Err(Status::invalid_argument("invalid after_id"));
        }
        if !after_id.is_empty() && given_cursor.is_none() {
            return Err(Status::invalid_argument("subsequent page requires cursor"));
        }
        let after = if after_id.is_empty() {
            String::new()
        } else {
            let id = uuid::Uuid::parse_str(&after_id).map_err(|_| Status::invalid_argument("invalid after_id UUID"))?;
            if id.is_nil() {
                return Err(Status::invalid_argument("after_id must not be nil"));
            }
            after_id
        };
        if let Some(given) = given_cursor {
            if given.epoch != cursor.epoch || given.stream != cursor.stream {
                return Err(Status::failed_precondition("annotation epoch/selection changed; restart pagination"));
            }
            if given.position != cursor.position {
                return Err(Status::aborted("runtime revision changed; restart pagination"));
            }
        }
        let scope =
            owner.as_ref().map(|owner| snapshot.state().note_owner_scope(owner)).transpose().map_err(Status::failed_precondition)?;
        let mut case_executions = std::collections::BTreeSet::new();
        if let Some(NoteOwner::Case { cycle_id, case_id }) = &owner {
            let cycle = &snapshot.state().cycles[cycle_id];
            let executions = snapshot
                .state()
                .executions_for_cycle(cycle)
                .take(dokime::domain::model::execution::notes::NOTE_SCAN_LIMIT + 1)
                .collect::<Vec<_>>();
            if executions.len() > dokime::domain::model::execution::notes::NOTE_SCAN_LIMIT {
                return Err(Status::resource_exhausted("annotation owner scan budget exceeded"));
            }
            case_executions
                .extend(executions.iter().filter(|item| &item.test_case_id == case_id).map(|item| item.test_execution_id.clone()));
        }
        Ok(Self {
            owner,
            context: selection.include_context,
            limit: if limit == 0 { 50 } else { limit as usize },
            after,
            cursor,
            scope,
            case_executions,
        })
    }
    pub fn matches(&self, actual: &RecordScope) -> bool {
        let Some(selected) = &self.scope else {
            return true;
        };
        if actual.session_id.as_ref().is_some_and(|id| Some(id) != selected.session_id.as_ref()) {
            return false;
        }
        if actual.test_cycle_id.is_none() {
            return self.context
                && actual.test_case_id.is_none()
                && actual.test_execution_id.is_none()
                && actual.template_id.is_none()
                && actual.suite_id.is_none()
                && actual.step_id.is_none()
                && actual.criterion_id.is_none();
        }
        if actual.test_cycle_id != selected.test_cycle_id {
            return false;
        }
        // Native health facts can identify an execution without repeating its
        // case. Resolve that relation through current ownership, while preserving
        // the original scope in the returned fact.
        if let Some(execution) = &actual.test_execution_id {
            if actual.test_case_id.as_ref().is_some_and(|case| selected.test_case_id.as_ref().is_some_and(|selected| case != selected)) {
                return false;
            }
            if let Some(selected) = &selected.test_execution_id {
                return execution == selected;
            }
            if selected.test_case_id.is_some() {
                return self.case_executions.contains(execution);
            }
            return true;
        }
        if let Some(case) = &selected.test_case_id {
            match &actual.test_case_id {
                Some(id) if id != case => return false,
                None => return self.context,
                _ => {}
            }
        }
        selected.test_execution_id.is_none() || self.context
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
