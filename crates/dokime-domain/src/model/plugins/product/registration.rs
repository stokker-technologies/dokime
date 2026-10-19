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
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use crate::model::definition::TestSuite;
use crate::model::document::resolve_document_plot_axes;
use crate::model::records::RecordContract;
use crate::model::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */
product_error!(SuiteContractError, formatter {
    Self::EmptySuiteId => formatter.write_str("suite id must not be empty"),
});

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestSuiteDefinition {
    pub suite_id: String,
    pub tags: Vec<String>,
    pub workflow: Option<String>,
    pub suite: TestSuite,
    pub record_contracts: Vec<RecordContract>,
    pub telemetry_contracts: Vec<TelemetryContract>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionProviderTarget {
    pub source: String,
    pub binding: SidecarExecutionBinding,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarExecutionBinding {
    pub workflow: String,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SuiteContractError {
    EmptySuiteId,
}

/* === Traits === */

/* === Implementations === */
impl TestSuiteDefinition {
    pub fn new(suite_id: impl Into<String>, suite: TestSuite) -> crate::Result<Self> {
        let suite_id = suite_id.into();
        if suite_id.trim().is_empty() {
            return Err(SuiteContractError::EmptySuiteId.into());
        }

        Ok(Self { suite_id, tags: Vec::new(), workflow: None, suite, record_contracts: Vec::new(), telemetry_contracts: Vec::new() })
    }
}

impl SidecarExecutionBinding {
    pub fn workflow(&self) -> &str {
        &self.workflow
    }
}

/* === Public Functions === */
pub fn resolve_suite_plot_axes(definition: &mut TestSuiteDefinition) {
    // Normalized, so the automatic `timestamp` column a contract does not
    // spell out is still a column a plot's x axis can be read against.
    let contracts = definition.telemetry_contracts.iter().map(TelemetryContract::normalized).collect::<Vec<_>>();
    for template in &mut definition.suite.test_case_templates {
        if let Some(document) = template.document.as_mut() {
            resolve_document_plot_axes(document, &contracts);
        }
    }
}

/* === Private Functions === */

/* === Tests === */
