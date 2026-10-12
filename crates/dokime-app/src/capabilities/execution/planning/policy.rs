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
pub(crate) use crate::transactions::UpdateCyclePolicy as UpdatePolicy;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::Transaction;
// Local Imports
use crate::{CommandContext, Error, RuntimeHandle, unavailable_rejection};
use dokime::core::{TestCycleId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn update_cycle_policy(
        &self,
        test_cycle_id: TestCycleId,
        continue_on_failure: bool,
        context: CommandContext,
    ) -> std::result::Result<(), Error> {
        let transaction = UpdatePolicy {
            test_cycle_id: test_cycle_id.clone(),
            continue_on_failure,
            occurred_at: Timestamp::now_utc(),
        };
        transaction
            .check(&self.state_read())
            .map_err(|message| Error::conflict("conflict", message))?;
        match context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
        {
            Some(key) => {
                self.commit_keyed(harmos::RequestKey::new(key), transaction)
                    .await
            }
            None => self.commit(transaction).await,
        }
        .map_err(|error| unavailable_rejection(error.to_string()))?;
        self.execution_storage.save_cycle(test_cycle_id).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
