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
use harmos::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::{Error, state::RuntimeState};
use dokime::core::{DocumentId, RecordScope, Timestamp};
use dokime::document::DocumentAddition;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::transaction(id = "dokime.documents.execution.add", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddExecutionDocumentParts {
    pub scope: RecordScope,
    pub additions: Vec<DocumentAddition>,
    pub occurred_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for AddExecutionDocumentParts {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        add_execution_document_parts(&mut state.clone(), self.clone())
            .map_err(|error| error.to_string())
    }

    fn apply(&self, state: &mut RuntimeState) {
        add_execution_document_parts(state, self.clone())
            .expect("Harmos checked the document additions before applying it");
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn add_execution_document_parts(
    state: &mut RuntimeState,
    update: AddExecutionDocumentParts,
) -> Result<(), Error> {
    let test_cycle_id = update
        .scope
        .test_cycle_id
        .clone()
        .ok_or_else(|| not_found("document addition scope missing cycle"))?;
    let test_execution_id = update
        .scope
        .test_execution_id
        .clone()
        .ok_or_else(|| not_found("document addition scope missing execution"))?;
    let execution = state
        .executions
        .get_mut(&test_execution_id)
        .filter(|execution| execution.test_cycle_id == test_cycle_id)
        .ok_or_else(|| not_found("execution not found"))?;

    let default_id = DocumentId::from_identifier(crate::documents::DEFAULT_DOCUMENT_ID)
        .map_err(|error| not_found(error.to_string()))?;
    let document = execution
        .documents
        .get_mut(&default_id)
        .ok_or_else(|| not_found("execution has no default document"))?;
    for addition in update.additions {
        let _ = document.add(addition);
    }

    state
        .cycles
        .get_mut(&test_cycle_id)
        .expect("the cycle was found above")
        .touch_at(update.occurred_at);

    Ok(())
}

fn not_found(message: impl Into<String>) -> Error {
    Error::not_found("not_found", message)
}

/* === Tests === */
