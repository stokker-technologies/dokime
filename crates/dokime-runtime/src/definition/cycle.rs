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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::execution::result::CycleView;
use crate::execution::TestExecutionInstance;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
pub struct TestCycleInstance {
    pub cycle: CycleView,
    executions: BTreeMap<TestExecutionId, TestExecutionInstance>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TestCycleInstance {
    pub fn new(cycle: CycleView) -> Self {
        let executions = cycle
            .test_executions
            .iter()
            .cloned()
            .map(|execution| {
                (
                    execution.test_execution_id.clone(),
                    TestExecutionInstance::new(execution),
                )
            })
            .collect();

        Self { cycle, executions }
    }

    pub fn executions(&self) -> impl Iterator<Item = &TestExecutionInstance> {
        self.executions.values()
    }

    pub fn get_execution(&self, execution_id: &TestExecutionId) -> Option<&TestExecutionInstance> {
        self.executions.get(execution_id)
    }

    pub fn cycle_id(&self) -> &dokime_domain::model::identity::TestCycleId {
        &self.cycle.test_cycle_id
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
