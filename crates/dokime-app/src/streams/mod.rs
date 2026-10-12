/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod telemetry;

/* === Re-Exports === */
pub use telemetry::{CycleTrack, ExecutionTrack, TelemetryRow};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use dokime::core::{Metadata, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AggregateOutput {
    pub key: String,
    pub metadata: Metadata,
    pub value: Value,
}

/* === Enums === */
#[harmos::streams]
pub enum Streams {
    Telemetry(TelemetryRow),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
