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

// Local Imports

/* === Constants === */
/// The shortest observation a bench parameter accepts, in model seconds.
pub const SHORTEST_S: f64 = 0.0001;

/// The finest simulation step a bench parameter accepts, in model seconds.
pub const FINEST_STEP_S: f64 = 0.0001;

/// The coarsest simulation step a bench parameter accepts, in model seconds.
pub const COARSEST_STEP_S: f64 = 1.0;

/// The most rows one observation may produce.
pub const MOST_ROWS: f64 = 100_000.0;

/* === Macros === */
/// Gives a template's parameters, which all carry the bench supply fields, the
/// device supply they describe and the row count their observation produces.
macro_rules! bench {
    ($parameters:ty) => {
        impl $parameters {
            pub fn supply(&self) -> obc_runtime::device::Supply {
                obc_runtime::device::Supply {
                    voltage_rms_v: self.ac_voltage_v,
                    load_ohm: self.load_ohm,
                    frequency_hz: self.frequency_hz,
                    phase_deg: self.phase_deg,
                    current_limit_a: self.current_limit_a,
                }
            }

            pub fn sample_count(&self) -> Result<u64, $crate::error::Error> {
                let most = $crate::suites::common::bench::MOST_ROWS;
                let rows = (self.duration_s / self.sample_interval_s).ceil();
                if !rows.is_finite() || !(1.0..=most).contains(&rows) {
                    return Err($crate::error::Error::Observation(format!(
                        "{} s at a {} s step is {rows} rows, and this bench runs between 1 and {most}",
                        self.duration_s, self.sample_interval_s,
                    )));
                }
                Ok(rows as u64)
            }
        }
    };
}

pub(crate) use bench;

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
