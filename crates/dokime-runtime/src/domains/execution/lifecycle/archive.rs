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

// Local Imports
use crate::runtime::storage::execution::ExecutionView;
use crate::transactions::{ArchiveSubject, SetArchived};
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
    pub async fn set_cycle_archived(&self, test_cycle_id: TestCycleId, archived: bool, context: CommandContext) -> Result<(), Error> {
        let live = self.state_read().cycles.contains_key(&test_cycle_id);
        if !live && !self.stored_cycle_exists(&test_cycle_id).await? {
            return Err(Error::not_found("not_found", "cycle not found"));
        }
        self.commit_archive(ArchiveSubject::Cycle(test_cycle_id), archived, context).await
    }

    pub async fn set_execution_archived(
        &self, test_execution_id: TestExecutionId, archived: bool, context: CommandContext,
    ) -> Result<(), Error> {
        let live = self.state_read().executions.contains_key(&test_execution_id);
        if !live && !self.stored_execution_exists(&test_execution_id).await? {
            return Err(Error::not_found("not_found", "execution not found"));
        }
        self.commit_archive(ArchiveSubject::Execution(test_execution_id), archived, context).await
    }

    async fn commit_archive(&self, subject: ArchiveSubject, archived: bool, context: CommandContext) -> Result<(), Error> {
        let _admission = self.evidence_admission.lock().await;
        let transaction = SetArchived { subject, archived };
        transaction.check(&self.state_read()).map_err(|message| Error::conflict("conflict", message))?;
        match context.idempotency_key.as_deref().filter(|key| !key.is_empty()) {
            Some(key) => self.commit_keyed(harmos::state::RequestKey::new(key), transaction).await,
            None => self.commit(transaction).await,
        }
        .map_err(|error| unavailable_rejection(error.to_string()))?;
        let (cycles, executions) = {
            let state = self.state_read();
            (
                state.archived_cycles.iter().map(ToString::to_string).collect(),
                state.archived_executions.iter().map(ToString::to_string).collect(),
            )
        };
        self.local_storage.history_archives(cycles, executions).await?;
        Ok(())
    }

    pub(crate) async fn stored_cycle_exists(&self, id: &TestCycleId) -> Result<bool, Error> {
        Ok(crate::storage::objects::cycle::read(&self.storage().clone(), id, harmos::storage::ObjectVersion::Latest).await?.is_some())
    }

    pub(crate) async fn stored_execution_exists(&self, id: &TestExecutionId) -> Result<bool, Error> {
        Ok(ExecutionView::open(self.storage(), id, harmos::storage::ObjectVersion::Latest)
            .await?
            .is_some_and(|inspection| inspection.execution().saved.is_some()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
