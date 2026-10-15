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
use crate::{CommandContext, Error, RuntimeHandle, unavailable_rejection};
use dokime::core::{TestCycleId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    /// Archives or unarchives one terminal cycle, live or retired to history.
    pub async fn set_cycle_archived(
        &self,
        test_cycle_id: TestCycleId,
        archived: bool,
        context: CommandContext,
    ) -> Result<(), Error> {
        let live = self.state_read().cycles.contains_key(&test_cycle_id);
        if !live && !self.stored_cycle_exists(&test_cycle_id).await? {
            return Err(Error::not_found("not_found", "cycle not found"));
        }
        self.commit_archive(ArchiveSubject::Cycle(test_cycle_id), archived, context)
            .await
    }

    /// Archives or unarchives one terminal execution, live or retired to history.
    pub async fn set_execution_archived(
        &self,
        test_execution_id: TestExecutionId,
        archived: bool,
        context: CommandContext,
    ) -> Result<(), Error> {
        let live = self
            .state_read()
            .executions
            .contains_key(&test_execution_id);
        if !live && !self.stored_execution_exists(&test_execution_id).await? {
            return Err(Error::not_found("not_found", "execution not found"));
        }
        self.commit_archive(
            ArchiveSubject::Execution(test_execution_id),
            archived,
            context,
        )
        .await
    }

    async fn commit_archive(
        &self,
        subject: ArchiveSubject,
        archived: bool,
        context: CommandContext,
    ) -> Result<(), Error> {
        let transaction = SetArchived { subject, archived };
        transaction
            .check(&self.state_read())
            .map_err(|message| Error::conflict("conflict", message))?;
        match context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
        {
            Some(key) => {
                self.commit_keyed(harmos::state::RequestKey::new(key), transaction)
                    .await
            }
            None => self.commit(transaction).await,
        }
        .map_err(|error| unavailable_rejection(error.to_string()))?;
        Ok(())
    }

    pub(crate) async fn stored_cycle_exists(&self, id: &TestCycleId) -> Result<bool, Error> {
        let storage = self.storage();
        let mut query =
            harmos::storage::ObjectQuery::<dokime::storage::objects::cycle::Cycle>::new();
        query.item = Some(id.as_str());
        let selection = storage.query(query).await?;
        let Some(view) = selection.view() else {
            return Ok(false);
        };
        Ok(dokime::storage::objects::cycle::read_in(
            storage,
            view,
            id,
            harmos::storage::ObjectVersion::Latest,
            selection.decoded_bytes(),
        )
        .await?
        .is_some())
    }

    pub(crate) async fn stored_execution_exists(
        &self,
        id: &TestExecutionId,
    ) -> Result<bool, Error> {
        let reads = self.config().storage_reads.execution;
        let view = self.storage().view_with_limits(reads).await?;
        let limit = view.decoded_bytes()?;
        Ok(ExecutionView::open(
            self.storage(),
            view,
            id,
            harmos::storage::ObjectVersion::Latest,
            limit,
        )
        .await?
        .is_some_and(|inspection| inspection.execution().saved.is_some()))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
