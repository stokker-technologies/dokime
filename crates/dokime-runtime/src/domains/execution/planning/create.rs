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

pub(crate) use crate::transactions::CreateCycle as Create;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{CommandContext, RuntimeHandle, runtime::Error, transactions::CycleDefinition};
use dokime_domain::model::identity::TestCycleId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn create_cycle(&self, definition: CycleDefinition, context: CommandContext) -> Result<TestCycleId, Error> {
        let request_key = context.idempotency_key.as_deref().filter(|key| !key.is_empty());
        let test_cycle_id = request_key.map_or_else(crate::model::identity::new_id::<TestCycleId>, |key| {
            TestCycleId::from_seed(format!("cycle-create:{}:{key}", self.state_read().identity.node.node_id))
        });
        let transaction = Create {
            definition,
            test_cycle_id: test_cycle_id.clone(),
            occurred_at: crate::model::time::now_utc(),
        };
        self.commit_with_key(request_key.map(str::to_owned), transaction).await?;
        self.execution_storage.save_cycle(test_cycle_id.clone()).await?;
        self.reconcile_sampling().await;
        Ok(test_cycle_id)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
