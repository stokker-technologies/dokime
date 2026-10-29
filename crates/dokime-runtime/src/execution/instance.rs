/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::document::DocumentRegistry;
use dokime_domain::model::execution::result::TestExecution;

#[derive(Clone, Debug)]
pub struct TestExecutionInstance {
    pub execution: TestExecution,
    pub documents: DocumentRegistry,
}

impl TestExecutionInstance {
    pub fn new(execution: TestExecution) -> Self {
        let mut documents = DocumentRegistry::default();
        let updated_at = execution.finished_at.clone().or_else(|| execution.started_at.clone()).unwrap_or_else(crate::model::time::now_utc);
        for definition in execution.documents.values().cloned() {
            documents.open_instance(definition, updated_at.clone());
        }

        Self { execution, documents }
    }
}
