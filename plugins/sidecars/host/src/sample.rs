/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::HostTelemetryStream;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostStreamSample {
    pub stream: HostTelemetryStream,
    pub timestamp: Timestamp,
    pub values: BTreeMap<String, Value>,
}

impl HostStreamSample {
    pub fn new(stream: HostTelemetryStream, values: BTreeMap<String, Value>) -> Self {
        Self::with_timestamp(stream, dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now()), values)
    }

    pub fn with_timestamp(stream: HostTelemetryStream, timestamp: Timestamp, values: BTreeMap<String, Value>) -> Self {
        Self { stream, timestamp, values }
    }
}
