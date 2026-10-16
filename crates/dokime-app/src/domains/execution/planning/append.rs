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
pub(crate) use crate::transactions::AppendCycle as Append;

/* === Imports === */
// Standard Library Imports
// Third Party Imports

// Local Imports
use crate::definition::TestCase;
use crate::{CommandContext, Error, RuntimeHandle};
use dokime::core::{TestCycleId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    /// Appends materialized cases to an existing mutable cycle. Each case's
    /// authored definition is read from the registry at its suite revision.
    pub async fn append_cycle_cases(
        &self,
        test_cycle_id: TestCycleId,
        cases: Vec<TestCase>,
        context: CommandContext,
    ) -> Result<(), Error> {
        let sources = {
            let state = self.state_read();
            cases
                .iter()
                .filter_map(|case| state.suites.suite_for_case(case))
                .map(|entry| entry.source.clone())
                .collect::<Vec<_>>()
        };
        let admission_identifier = context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
            .map_or_else(dokime::core::CommandId::new, |key| {
                dokime::core::CommandId::from_seed(format!(
                    "cycle-append:{}:{test_cycle_id}:{key}",
                    self.state_read().identity.node.node_id
                ))
            });
        let transaction = Append {
            admission_identifier,
            settings: super::settings(self, sources)?,
            test_cycle_id: test_cycle_id.clone(),
            cases,
            occurred_at: Timestamp::now_utc(),
        };
        self.commit_with_key(context.idempotency_key, transaction)
            .await?;
        self.execution_storage
            .save_cycle(test_cycle_id.clone())
            .await?;
        self.reconcile_sampling().await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
