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
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::{runtime::Error, state::RuntimeState};
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;
use crate::document::{DocumentAddition, DocumentCardBody, DownsampleTechnique};

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

#[harmos::transaction(id = "dokime.documents.plot.settings", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePlotSettings {
    pub cycle_id: TestCycleId,
    pub execution_id: TestExecutionId,
    pub document_id: DocumentId,
    pub plot_id: Identifier,
    pub display_points: Option<u32>,
    pub downsample: DownsampleTechnique,
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

impl Transaction<RuntimeState> for UpdatePlotSettings {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), String> {
        self.validate(state).map_err(|error| error.to_string())
    }

    fn apply(&self, state: &mut RuntimeState) {
        let execution = state
            .executions
            .get_mut(&self.execution_id)
            .expect("checked execution");
        let document = execution
            .documents
            .values_mut()
            .find(|document| document.document_id == self.document_id)
            .expect("checked document");
        let plots = document
            .cards
            .iter_mut()
            .map(|card| &mut card.body)
            .chain(
                document
                    .sections
                    .iter_mut()
                    .flat_map(|section| section.cards.iter_mut().map(|card| &mut card.body)),
            )
            .filter_map(|body| match body {
                DocumentCardBody::Plot { plot } if plot.metadata.identifier == self.plot_id => {
                    Some(plot)
                }
                _ => None,
            });
        for plot in plots {
            plot.display_points = self.display_points;
            plot.downsample = match self.downsample {
                DownsampleTechnique::Unspecified => DownsampleTechnique::Auto,
                technique => technique,
            };
        }
        state
            .cycles
            .get_mut(&self.cycle_id)
            .expect("checked cycle")
            .touch_at(self.occurred_at.clone());
    }
}

impl UpdatePlotSettings {
    pub(crate) fn validate(&self, state: &RuntimeState) -> Result<(), Error> {
        if state.shutting_down {
            return Err(Error::conflict(
                "document_closed",
                "runtime is shutting down",
            ));
        }
        if self.display_points.is_some_and(|points| points > 100_000) {
            return Err(Error::Operation(
                "display_points must not exceed 100000".into(),
            ));
        }
        let execution = state
            .executions
            .get(&self.execution_id)
            .filter(|execution| execution.test_cycle_id == self.cycle_id)
            .ok_or_else(|| not_found("execution not found"))?;
        let settling = state.run_requests.values().any(|request| {
            request.summary.cycle_id == self.cycle_id
                && request.completion_at.is_some()
                && (request.test_execution_id.as_ref() == Some(&self.execution_id)
                    || request.settled_execution_ids.contains(&self.execution_id))
        });
        if execution.saved.is_some()
            || settling
            || !matches!(
                execution.status,
                dokime_domain::model::execution::result::ExecutionStatus::Pending
                    | dokime_domain::model::execution::result::ExecutionStatus::Running
            )
        {
            return Err(Error::conflict(
                "document_closed",
                "plot settings cannot change after execution settlement",
            ));
        }
        let document = execution
            .documents
            .values()
            .find(|document| document.document_id == self.document_id)
            .ok_or_else(|| not_found("execution document not found"))?;
        let exists = document.cards.iter().map(|card| &card.body)
            .chain(document.sections.iter().flat_map(|section| section.cards.iter().map(|card| &card.body)))
            .any(|body| matches!(body, DocumentCardBody::Plot { plot } if plot.metadata.identifier == self.plot_id));
        if !exists {
            return Err(not_found("document plot not found"));
        }
        Ok(())
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
