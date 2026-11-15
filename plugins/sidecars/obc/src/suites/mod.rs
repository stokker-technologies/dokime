/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Suites: One Folder Entry per Procedure Family

/* === Modules === */

pub mod bench;
pub mod checks;
pub mod protection;
pub mod recovery;
pub mod startup;
pub mod steady;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::catalog::number;
use crate::device::Supply;
use dokime::plugin::{Context, Template};
use dokime::{Parameter, Result};

/* === Constants === */

// Bench Bounds for Simulated Durations and Model Sample Intervals.
const SHORTEST_S: f64 = 0.0001;
const FINEST_STEP_S: f64 = 0.0001;
const COARSEST_STEP_S: f64 = 1.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare Parameters in Order on a Template.
pub fn parameters(mut template: Template, parameters: impl IntoIterator<Item = Parameter>) -> Template {
    for parameter in parameters {
        template = template.parameter(parameter);
    }
    template
}

/// The AC Supply Every Charger Procedure Applies, With Nominal Defaults.
pub fn supply(voltage: Option<Parameter>, load: Option<Parameter>) -> Vec<Parameter> {
    vec![
        voltage.unwrap_or_else(|| number("ac_voltage_v", "AC RMS Voltage", "V", 230.0, Some(0.0), Some(300.0))),
        load.unwrap_or_else(|| number("load_ohm", "Load Impedance Magnitude", "ohm", 100.0, Some(0.1), None)),
        number("frequency_hz", "Supply Frequency", "Hz", 50.0, Some(1.0), Some(1000.0)),
        number("phase_deg", "Current Phase Lag", "deg", 15.0, Some(-89.0), Some(89.0)),
        number("current_limit_a", "RMS Current Limit", "A", 5.0, Some(0.001), None),
    ]
}

/// The Observation Window and Model Sample Interval of a Charger Procedure.
pub fn timing(duration_label: &str, duration_s: f64) -> [Parameter; 2] {
    [
        number("duration_s", duration_label, "s", duration_s, Some(SHORTEST_S), None),
        number("sample_interval_s", "Simulation Sample Interval", "s", 0.001, Some(FINEST_STEP_S), Some(COARSEST_STEP_S)),
    ]
}

/// Read the Supply Parameters of the Frozen Case.
pub fn read_supply(ctx: &Context) -> Result<Supply> {
    Ok(Supply {
        voltage_rms_v: ctx.parameter("ac_voltage_v")?,
        load_ohm: ctx.parameter("load_ohm")?,
        frequency_hz: ctx.parameter("frequency_hz")?,
        phase_deg: ctx.parameter("phase_deg")?,
        current_limit_a: ctx.parameter("current_limit_a")?,
    })
}

/// The Measured Sample Count of a Stream, From Saved Evidence.
pub async fn sample_count(ctx: &Context, stream: &str, step: &str) -> Result<u64> {
    let statistics = ctx.statistics(stream, "voltage_v", step).await?;
    Ok(statistics.statistics.map_or(0, |statistics| statistics.count))
}

/* === Private Functions === */

/* === Tests === */
