/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod pulse;

pub use pulse::{DEFAULT_SAMPLE_COUNT, DEFAULT_TOLERANCE, Pulse, PulseRow, SAMPLE_INTERVAL_MS, pulse_value};

pub const ID: &str = "obc.checks";

pub struct ObcChecks;

#[dokime::catalog]
pub enum Templates {
    Pulse(pulse::Pulse),
}

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
