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
use crate::transactions::labels::{LabelSubject, RenameLabel};
use crate::{CommandContext, RuntimeHandle, runtime::Error, unavailable_rejection};
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn rename_cycle(&self, test_cycle_id: TestCycleId, label: String, context: CommandContext) -> Result<String, Error> {
        let label = admitted_label(&label)?;
        if !self.state_read().cycles.contains_key(&test_cycle_id) {
            return Err(if self.stored_cycle_exists(&test_cycle_id).await? {
                Error::conflict("conflict", "cycle is in History; its label is sealed")
            } else {
                Error::not_found("not_found", "cycle not found")
            });
        }
        self.commit_label(LabelSubject::Cycle(test_cycle_id.clone()), label.clone(), context).await?;
        self.execution_storage.save_cycle_header(test_cycle_id).await.map_err(|error| unsaved(&error))?;
        Ok(label)
    }

    pub async fn rename_execution(
        &self, test_execution_id: TestExecutionId, label: String, context: CommandContext,
    ) -> Result<String, Error> {
        let label = admitted_label(&label)?;
        let Some(cycle_id) = self.state_read().executions.get(&test_execution_id).map(|execution| execution.test_cycle_id.clone()) else {
            return Err(if self.stored_execution_exists(&test_execution_id).await? {
                Error::conflict("conflict", "execution is in History; its label is sealed")
            } else {
                Error::not_found("not_found", "execution not found")
            });
        };
        self.commit_label(LabelSubject::Execution(test_execution_id.clone()), label.clone(), context).await?;
        // Every Attempt Is Unsealed, So They All Checkpoint With Their Cycle
        self.execution_storage.save_cycle(cycle_id).await.map_err(|error| unsaved(&error))?;
        Ok(label)
    }

    async fn commit_label(&self, subject: LabelSubject, label: String, context: CommandContext) -> Result<(), Error> {
        let transaction = RenameLabel { subject, label, edited_at: crate::model::time::now_utc() };
        harmos::state::Transaction::check(&transaction, &self.state_read()).map_err(|message| Error::conflict("conflict", message))?;
        match context.idempotency_key.as_deref().filter(|key| !key.is_empty()) {
            Some(key) => self.commit_keyed(harmos::state::RequestKey::new(key), transaction).await,
            None => self.commit(transaction).await,
        }
        .map_err(|error| unavailable_rejection(error.to_string()))?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn admitted_label(label: &str) -> Result<String, Error> {
    let label = label.trim();
    if label.is_empty() {
        return Err(Error::validation("invalid_label", "label must not be empty", vec![]));
    }
    Ok(label.to_owned())
}

fn unsaved(error: &crate::runtime::Error) -> Error {
    unavailable_rejection(format!("label committed but its revision was not saved: {error}"))
}

/* === Tests === */
