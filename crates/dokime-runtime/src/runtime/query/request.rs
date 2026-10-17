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
use serde::{Deserialize, Serialize};
// Local Imports
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::DurationMillis;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuntimeCursor(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeViewQuery {
    #[serde(default)]
    pub scope: Option<RecordScope>,
    pub topic: RuntimeViewTopic,
    pub window: RuntimeWindow,
    pub resolution: RuntimeResolution,
    #[serde(default)]
    pub cursor: Option<RuntimeCursor>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RuntimeWindow {
    Full,
    Since { cursor: RuntimeCursor },
    TimeRange { start: Timestamp, end: Timestamp },
    Tail { duration: DurationMillis },
    LastSamples { count: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "topic")]
pub enum RuntimeViewTopic {
    Overview,
    InstalledPlugins,
    Suites,
    Cycle,
    Execution,
    ExecutionFiles,
    RecordDefinitions,
    Records,
    TelemetryDefinitions { stream: Option<Identifier> },
    Telemetry { stream: Option<Identifier> },
    Document { document_id: DocumentId },
    Documents,
    Artifacts,
    Resources,
    Prompts,
    Health,
    Traceability,
    Notes,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RuntimeResolution {
    Raw,
    Samples { count: usize },
    Pixels { width: u32 },
    Summary,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
