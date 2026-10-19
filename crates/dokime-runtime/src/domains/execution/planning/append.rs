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
use crate::{RuntimeHandle, runtime::Error};
use dokime_domain::model::identity::TestCycleId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn append_executions(
        &self, cycle_id: TestCycleId,
        executions: Vec<(dokime_domain::model::identity::TestExecutionId, dokime_domain::model::identity::CaseId)>,
    ) -> Result<(), Error> {
        if executions.is_empty() || executions.len() > 1000 {
            return Err(Error::conflict("invalid_append", "append requires 1 to 1000 executions"));
        }
        let sources = {
            let state = self.state_read();
            executions
                .iter()
                .filter_map(|(_, id)| state.suites.case(id))
                .filter_map(|case| state.suites.suite_for_case(case))
                .map(|suite| suite.source.clone())
                .collect::<Vec<_>>()
        };
        let transaction = crate::transactions::AppendExecutions {
            cycle_id: cycle_id.clone(),
            executions,
            settings: super::settings(self, sources)?,
            occurred_at: crate::model::time::now_utc(),
        };
        let permit = self.admit_owned_work().ok_or(Error::ShuttingDown)?;
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            for (id, _) in &transaction.executions {
                let known = runtime.state_read().executions.contains_key(id);
                if known {
                    continue;
                }
                let stored =
                    crate::runtime::storage::execution::ExecutionView::open(runtime.storage(), id, harmos::storage::ObjectVersion::Latest)
                        .await?;
                if stored.is_some() {
                    return Err(Error::conflict("execution_exists", "execution identity already belongs to saved history"));
                }
            }
            runtime.commit(transaction).await.map_err(|error| Error::Operation(error.to_string()))?;
            runtime.execution_storage.save_cycle(cycle_id).await?;
            runtime.reconcile_sampling().await;
            Ok(())
        })
        .await
        .map_err(|error| Error::Operation(error.to_string()))?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
