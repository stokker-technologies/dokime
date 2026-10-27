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
use super::value::Value;
use crate::document::capture::Capture;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::traceability::TraceabilitySnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CycleReportInput {
    pub session_id: SessionId,
    pub cycle: CycleView,

    #[serde(default)]
    pub documents: BTreeMap<DocumentId, Capture>,

    #[serde(default)]
    pub plots: BTreeMap<String, Value>,

    #[serde(default)]
    pub case_documents: Vec<ReportCaseDocument>,

    pub records: crate::storage::ReadRows<RecordEntry>,
    pub telemetry: crate::storage::ReadRows<TelemetrySample>,
    pub traceability: Vec<TraceabilitySnapshot>,

    #[serde(default)]
    pub artifact_links: Vec<ReportArtifactLink>,

    #[serde(default)]
    pub appearance: Option<Value>,

    pub generated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportCaseDocument {
    pub test_case_id: CaseId,
    pub test_execution_id: Option<TestExecutionId>,

    #[serde(default)]
    pub documents: BTreeMap<DocumentId, Capture>,

    #[serde(default)]
    pub plots: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportArtifactLink {
    pub artifact_id: String,
    pub label: String,
    pub href: String,
    pub path: String,

    #[serde(default)]
    pub media_type: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub size_bytes: Option<u64>,

    #[serde(default)]
    pub group_id: Option<String>,

    #[serde(default)]
    pub group_label: Option<String>,

    #[serde(default)]
    pub scope: Option<RecordScope>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/* === Private Functions === */
/* === Tests === */
