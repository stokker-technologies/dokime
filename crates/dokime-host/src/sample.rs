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
use crate::{HOST_SOURCE_ID, HostTelemetryStream};
use dokime_contract::core::{Timestamp, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostStreamSample {
    pub stream: HostTelemetryStream,
    pub timestamp: Timestamp,
    pub values: BTreeMap<String, Value>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl HostStreamSample {
    pub fn new(stream: HostTelemetryStream, values: BTreeMap<String, Value>) -> Self {
        Self::with_timestamp(stream, dokime_contract::core::Timestamp::from_system_time(std::time::SystemTime::now()), values)
    }

    pub fn with_timestamp(
        stream: HostTelemetryStream,
        timestamp: Timestamp,
        values: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            stream,
            timestamp,
            values,
        }
    }
}

/* === Public Functions === */
pub(crate) fn default_source() -> &'static str {
    HOST_SOURCE_ID
}

/* === Private Functions === */

/* === Tests === */
