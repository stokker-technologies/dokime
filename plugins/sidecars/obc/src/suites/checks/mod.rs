/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod cpu_load;
pub mod pulse;

/* === Re-Exports === */

pub use pulse::{DEFAULT_SAMPLE_COUNT, DEFAULT_TOLERANCE, Pulse, PulseRow, SAMPLE_INTERVAL_MS, pulse_value};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

// Third-Party Imports
pub const ID: &str = "obc.checks";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ObcChecks;

/* === Enums === */

#[dokime::catalog]
pub enum Templates {
    Pulse(pulse::Pulse),
    CpuLoad(cpu_load::CpuLoad),
}

/* === Traits === */

/* === Implementations === */

#[dokime::suite(
    id = ID,
    label = "OBC Checks",
    tags = ["example", "sidecar"],
    // The binding that makes this suite run *inside the sidecar*: the host
    // dispatches the sidecar that registered this suite and drives it over
    // `dokime.execution.{start,drain,cancel}`. The workflow the sidecar sees on
    // the start request defaults to the suite id.
    templates = Templates, description = "Bench checks run inside the OBC plugin before a Charger is connected.")]
// There is no `records()` and no `telemetry()` here: the one stream this suite
// streams is declared by the `#[dokime::observer]` that publishes it, and the two
// records its template publishes are declared beside the steps that publish
// them. The suite publishes what its templates declared.
impl ObcChecks {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
