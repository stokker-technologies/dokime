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
use crate::{HostTelemetryConfig, HostTelemetryStream};
use dokime_domain::model::time::DurationMillis;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostTelemetryTaskSpec {
    pub stream: HostTelemetryStream,
    pub interval: DurationMillis,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl HostTelemetryTaskSpec {
    pub fn from_config(config: &HostTelemetryConfig) -> Vec<Self> {
        config.task_streams().into_iter().map(|stream| Self { stream, interval: config.interval_for(stream) }).collect()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
