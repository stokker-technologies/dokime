/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Streams: One Collection per Native Measurement, Each Feeding the Streams Its Fields Fill

/* === Modules === */

pub mod bluetooth;
pub mod cpu;
pub mod network;
pub mod system;
pub mod wifi;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::telemetry::{PROBES, Probe};
use dokime::Result;
use dokime::plugin::{Rows, Stream};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare One Collection Outcome Stream.
pub fn probe(identifier: &'static str) -> Stream<Probe> {
    let label = PROBES.iter().find(|(stream, _)| *stream == identifier).map_or(identifier, |(_, label)| *label);
    Stream::new(identifier, label)
}

/// Publish a Collection's Outcome; a Reading Without a Status Was Collected Successfully.
pub fn outcome(rows: &mut Rows, stream: &str, status: Option<String>, error: Option<String>) -> Result<()> {
    let row = Probe {
        outcome: status.unwrap_or_else(|| "ok".into()),
        error: error.unwrap_or_default(),
    };
    rows.publish(stream, row)
}

/* === Private Functions === */

/* === Tests === */
