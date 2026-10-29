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
use crate::model::time::DurationMillis;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySamplingRequest {
    pub request_id: String,
    pub source_kind: TelemetrySourceKind,
    pub source_id: String,
    pub stream: String,

    #[serde(default)]
    pub interval: Option<DurationMillis>,

    #[serde(default)]
    pub min_interval: Option<DurationMillis>,

    #[serde(default)]
    pub start_after: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TelemetrySampleKey {
    pub source_kind: TelemetrySourceKind,
    pub source_id: String,
    pub stream: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetryCaptureLease {
    pub lease_id: String,
    pub job_id: String,
    pub request: TelemetrySamplingRequest,
    pub scope: SamplingScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SamplingScope {
    #[serde(default)]
    pub test_cycle_id: Option<String>,

    #[serde(default)]
    pub test_case_id: Option<String>,

    #[serde(default)]
    pub test_execution_id: Option<String>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySourceKind {
    InstalledPlugin,
    PythonCallback,
    Wasm,
    External,
}

/* === Traits === */

/* === Implementations === */

impl TelemetrySamplingRequest {
    pub fn sample_key(&self) -> TelemetrySampleKey {
        TelemetrySampleKey {
            source_kind: self.source_kind.clone(),
            source_id: self.source_id.clone(),
            stream: self.stream.clone(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
