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
use crate::{CommandContext, RuntimeHandle, runtime::Error};
use dokime_domain::model::identity::TestCycleId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn append_cycle_cases(
        &self, test_cycle_id: TestCycleId, cases: Vec<TestCase>, context: CommandContext,
    ) -> Result<Vec<dokime_domain::model::identity::TestExecutionId>, Error> {
        let sources = {
            let state = self.state_read();
            cases.iter().filter_map(|case| state.suites.suite_for_case(case)).map(|entry| entry.source.clone()).collect::<Vec<_>>()
        };
        let admission_identifier = context.idempotency_key.as_deref().filter(|key| !key.is_empty()).map_or_else(
            crate::model::identity::new_id::<dokime_domain::model::identity::CommandId>,
            |key| {
                dokime_domain::model::identity::CommandId::from_seed(format!(
                    "cycle-append:{}:{test_cycle_id}:{key}",
                    self.state_read().identity.node.node_id
                ))
            },
        );
        let transaction = Append {
            admission_identifier,
            settings: super::settings(self, sources)?,
            test_cycle_id: test_cycle_id.clone(),
            cases,
            occurred_at: crate::model::time::now_utc(),
        };
        let execution_ids = transaction.execution_ids();
        self.commit_with_key(context.idempotency_key, transaction).await?;
        self.execution_storage.save_cycle(test_cycle_id.clone()).await?;
        self.reconcile_sampling().await;
        Ok(execution_ids)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
