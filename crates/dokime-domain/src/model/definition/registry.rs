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
use crate::model::plugins::TestSuiteDefinition;
use crate::model::records::RecordContract;
use crate::model::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SuiteSet {
    #[serde(default)]
    pub suites: Vec<TestSuiteDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SuiteSnapshot {
    pub suite_id: String,
    pub source: String,
    pub tags: Vec<String>,
    pub workflow: Option<String>,
    pub suite: TestSuite,
    pub record_contracts: Vec<RecordContract>,
    pub telemetry_contracts: Vec<TelemetryContract>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SuiteSet {
    pub fn new(suites: impl IntoIterator<Item = TestSuiteDefinition>) -> Self {
        Self { suites: suites.into_iter().collect() }
    }

    pub fn suite(mut self, suite: TestSuiteDefinition) -> Self {
        self.suites.push(suite);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
