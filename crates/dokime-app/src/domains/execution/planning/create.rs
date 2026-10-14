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
pub(crate) use crate::transactions::{CreateCycle as Create, CreateCycleMode as Mode};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{CommandContext, Error, RuntimeHandle, transactions::CycleDefinition};
use dokime::core::{TestCycleId, Timestamp};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn start_registered_cycle(
        &self,
        suite_id: &str,
        metadata: dokime::core::Metadata,
        continue_on_failure: bool,
        context: CommandContext,
    ) -> Result<TestCycleId, Error> {
        let (suite, cases) = {
            let state = self.state_read();
            let suite = state
                .suites
                .get(suite_id)
                .ok_or_else(|| Error::not_found("suite_not_found", "registered suite not found"))?;
            let registration_id = context
                .idempotency_key
                .as_deref()
                .filter(|key| !key.is_empty())
                .map_or_else(dokime::core::CommandId::new, |key| {
                    dokime::core::CommandId::from_seed(format!(
                        "registered-cycle:{}:{key}",
                        state.identity.node.node_id,
                    ))
                });
            let cases = state.suites.materialize_suite(suite_id, registration_id)?;
            (suite.suite.clone(), cases)
        };
        self.start_cycle(
            CycleDefinition {
                suite,
                metadata,
                cases,
                continue_on_failure,
            },
            context,
        )
        .await
    }

    pub async fn preview_cycle(
        &self,
        definition: CycleDefinition,
        context: CommandContext,
    ) -> Result<TestCycleId, Error> {
        self.commit_cycle(Mode::Preview, definition, context).await
    }

    pub async fn start_cycle(
        &self,
        definition: CycleDefinition,
        context: CommandContext,
    ) -> Result<TestCycleId, Error> {
        self.commit_cycle(Mode::Start, definition, context).await
    }

    async fn commit_cycle(
        &self,
        mode: Mode,
        definition: CycleDefinition,
        context: CommandContext,
    ) -> Result<TestCycleId, Error> {
        let request_key = context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty());
        let test_cycle_id = request_key.map_or_else(TestCycleId::new, |key| {
            TestCycleId::from_seed(format!(
                "cycle-create:{}:{key}",
                self.state_read().identity.node.node_id
            ))
        });
        let sources = {
            let state = self.state_read();
            definition
                .cases
                .iter()
                .filter_map(|case| state.suites.suite_for_case(case))
                .map(|entry| entry.source.clone())
                .collect::<Vec<_>>()
        };
        let transaction = Create {
            settings: super::settings(self, sources)?,
            mode,
            definition,
            test_cycle_id: test_cycle_id.clone(),
            occurred_at: Timestamp::now_utc(),
        };
        self.commit_with_key(request_key.map(str::to_owned), transaction)
            .await?;
        self.execution_storage
            .save_cycle(test_cycle_id.clone())
            .await?;
        self.reconcile_sampling().await;
        Ok(test_cycle_id)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
