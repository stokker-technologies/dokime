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
use serde::{Deserialize, Serialize};
// Local Imports
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use crate::diagnostics::Diagnostic;
use dokime_domain::model::records::error::RecordErrorDetail;
use crate::records::RecordSeverity;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderCycleOutcome {
    pub status: String,
    pub dependency_imports: Vec<ProviderDependencyProbe>,
    pub provider: ProviderStatusOutcome,
    pub executions: Vec<ProviderExecutionOutcome>,
    pub failures: Vec<ProviderExecutionFailure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderDependencyProbe {
    pub module: String,
    pub status: String,
    pub version: Option<String>,
    pub error: Option<ProviderExecutionErrorDetail>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderStatusOutcome {
    pub status: String,
    pub domain_registration_count: u64,
    pub test_suite_count: u64,
    pub test_suite_identifiers: Vec<String>,
    pub error: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderExecutionOutcome {
    pub case_id: String,
    pub test_execution_id: String,
    pub template_identifier: String,
    pub label: String,
    pub module: String,
    pub status: String,
    pub phases: Vec<ProviderPhaseOutcome>,
    pub records: Vec<ProviderRecordOutcome>,
    pub telemetry: Vec<ProviderTelemetryOutcome>,
    pub criteria: Vec<ProviderCriterionOutcome>,
    pub failures: Vec<ProviderExecutionFailure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderPhaseOutcome {
    pub identifier: String,
    pub label: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderRecordOutcome {
    pub record_type: String,
    pub timestamp: Timestamp,
    pub severity: RecordSeverity,
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<RecordErrorDetail>,
    #[serde(default)]
    pub details: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderTelemetryOutcome {
    pub stream: String,
    pub label: String,
    pub samples: Vec<ProviderTelemetryRow>,
}

/// A producer-declared row and its untouched canonical Postcard representation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderTelemetryRow {
    pub values: BTreeMap<String, Value>,
    pub payload: Vec<u8>,
}

impl ProviderTelemetryRow {
    pub fn typed<T: harmos::Message + crate::telemetry::TelemetryRow>(row: T) -> Self {
        let payload = row.encode_to_vec();
        Self {
            values: row.into_row(),
            payload,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderCriterionOutcome {
    pub identifier: String,
    pub label: String,
    pub status: String,
    pub expected: Option<Value>,
    pub required: Option<bool>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExecutionFailure {
    pub stage: String,
    pub template_identifier: Option<String>,
    pub module: Option<String>,
    pub message: String,
    #[serde(default)]
    pub detail: Option<Box<Diagnostic>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExecutionErrorDetail {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub message: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ProviderCycleOutcome {
    pub fn ready() -> Self {
        Self {
            status: "passed".to_owned(),
            dependency_imports: Vec::new(),
            provider: ProviderStatusOutcome::ready(),
            executions: Vec::new(),
            failures: Vec::new(),
        }
    }

    pub fn failed(stage: impl Into<String>, message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            status: "failed".to_owned(),
            dependency_imports: Vec::new(),
            provider: ProviderStatusOutcome {
                status: "error".to_owned(),
                error: Some(Value::from(message.clone())),
                ..ProviderStatusOutcome::ready()
            },
            executions: Vec::new(),
            failures: vec![ProviderExecutionFailure::new(stage, message)],
        }
    }
}

impl ProviderExecutionFailure {
    pub fn new(stage: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            stage: stage.into(),
            template_identifier: None,
            module: None,
            message: message.into(),
            detail: None,
        }
    }

    #[must_use]
    pub fn detailed(mut self, detail: &Diagnostic) -> Self {
        self.message = crate::diagnostics::redacted(&self.message);
        self.detail = Some(Box::new(detail.redacted()));
        self
    }
}

impl ProviderStatusOutcome {
    pub fn ready() -> Self {
        Self {
            status: "ready".to_owned(),
            domain_registration_count: 0,
            test_suite_count: 0,
            test_suite_identifiers: Vec::new(),
            error: None,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
