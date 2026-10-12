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
// Local Imports
use crate::transactions::CopyCycle;
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
    pub async fn copy_cycle(
        &self,
        source_cycle_id: TestCycleId,
        context: CommandContext,
    ) -> Result<TestCycleId, Error> {
        let transaction = CopyCycle {
            source_cycle_id,
            test_cycle_id: TestCycleId::new(),
            occurred_at: Timestamp::now_utc(),
        };
        {
            let state = self.state_read();
            if !state.cycles.contains_key(&transaction.source_cycle_id) {
                return Err(Error::not_found(
                    "cycle_not_found",
                    "source cycle not found",
                ));
            }
            transaction
                .check(&state)
                .map_err(|message| Error::conflict("copy_refused", message))?;
        }
        let id = transaction.test_cycle_id.clone();
        // This mutation deliberately offers no retry-key contract. The public
        // API documents that an ambiguous transport result is unsafe to replay.
        if context.idempotency_key.is_some() {
            return Err(Error::validation(
                "unsupported_idempotency",
                "cycle copy does not support idempotency keys",
                vec![],
            ));
        }
        self.execution_storage
            .save_cycle(transaction.source_cycle_id.clone())
            .await?;
        self.commit(transaction)
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?;
        self.execution_storage.save_cycle(id.clone()).await?;
        Ok(id)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
