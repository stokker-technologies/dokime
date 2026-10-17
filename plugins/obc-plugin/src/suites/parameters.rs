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
use crate::error::Error;
use dokime::definition::ParameterDefault;
use obc_runtime::device;

/* === Constants === */
const SHORTEST_S: f64 = 0.0001;

const FINEST_STEP_S: f64 = 0.0001;

const COARSEST_STEP_S: f64 = 1.0;

const MOST_ROWS: f64 = 100_000.0;

pub const CONNECTION_BUDGET_S: f64 = 30.0;

/* === Macros === */
macro_rules! bench {
    ($($inputs:ty),+ $(,)?) => {
        $(impl $inputs {
                        pub fn supply(&self) -> device::Supply {
                device::Supply {
                    voltage_rms_v: self.ac_voltage_v,
                    load_ohm: self.load_ohm,
                    frequency_hz: self.frequency_hz,
                    phase_deg: self.phase_deg,
                    current_limit_a: self.current_limit_a,
                }
            }

                                                                                    pub fn sample_count(&self) -> Result<u64, Error> {
                let rows = (self.duration_s / self.sample_interval_s).ceil();
                if !rows.is_finite() || !(1.0..=MOST_ROWS).contains(&rows) {
                    return Err(Error::Observation(format!(
                        "{} s at a {} s step is {rows} rows, and this bench runs between 1 and {MOST_ROWS}",
                        self.duration_s, self.sample_interval_s,
                    )));
                }
                Ok(rows as u64)
            }
        })+
    };
}

bench!(Supply, Startup, Steady, Protection, Recovery);

/* === Type Aliases === */

/* === Structs === */
#[dokime::parameters]
pub struct Supply {
    #[parameter(label = "AC Voltage (RMS)", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,
    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,
    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,
    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,
    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,
    #[parameter(label = "Observation Duration", unit = "s", default = 0.2, min = SHORTEST_S)]
    pub duration_s: f64,
    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
}

#[dokime::parameters]
pub struct Startup {
    #[parameter(label = "AC Voltage (RMS)", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,
    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,
    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,
    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,
    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,
    #[parameter(label = "Observation Duration", unit = "s", default = 0.2, min = SHORTEST_S)]
    pub duration_s: f64,
    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
    #[parameter(label = "Attach measurement summary", default = true)]
    pub emit_artifact: bool,
    #[parameter(label = "Attach reference.png from image directory", default = false)]
    pub attach_reference: bool,
    #[parameter(label = "Pace samples in real time", default = true)]
    pub realtime: bool,
    #[parameter(
        label = "Observe until stopped",
        default = false,
        description = "Whether a host stop is the expected end of this run, named once more on the template's own `graceful_stop`."
    )]
    pub continuous: bool,
    #[parameter(
        label = "Wait for the unit to report itself connected",
        default = false
    )]
    pub await_connection: bool,
    #[parameter(
        label = "Connection wait budget",
        unit = "s",
        default = CONNECTION_BUDGET_S,
        min = 0.0,
    )]
    pub connection_budget_s: f64,
}

#[dokime::parameters]
pub struct Steady {
    #[parameter(
        label = "AC Voltage (RMS)",
        unit = "V",
        expansion = "cartesian",
        default = ParameterDefault::space([207.0, 230.0, 253.0]),
    )]
    pub ac_voltage_v: f64,
    #[parameter(
        label = "Load Impedance Magnitude",
        unit = "ohm",
        expansion = "cartesian",
        default = ParameterDefault::linspace(80.0, 120.0, 3),
    )]
    pub load_ohm: f64,
    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,
    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,
    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,
    #[parameter(label = "Observation Duration", unit = "s", default = 0.05, min = SHORTEST_S)]
    pub duration_s: f64,
    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
}

#[dokime::parameters]
pub struct Protection {
    #[parameter(label = "AC Voltage (RMS)", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,
    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,
    #[parameter(
        label = "Load Steps",
        unit = "ohm",
        expansion = "cartesian",
        min = 0.1,
        min_items = 1,
        max_items = 16,
        default = ParameterDefault::literal(dokime::core::Value::Array(::std::vec![
            dokime::core::Value::Number(100.0),
            dokime::core::Value::Number(50.0),
            dokime::core::Value::Number(10.0),
        ])), description = "The impedances the procedure applies in order, starting from the one the Charger was energized at. Declared in ohms like the load it replaces.")]
    pub load_steps: Vec<f64>,
    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,
    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,
    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,
    #[parameter(label = "Observation Duration", unit = "s", default = 0.2, min = SHORTEST_S)]
    pub duration_s: f64,
    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
}

#[dokime::parameters]
pub struct Recovery {
    #[parameter(label = "AC Voltage (RMS)", unit = "V", default = 230.0)]
    pub ac_voltage_v: f64,
    #[parameter(label = "Load Impedance Magnitude", unit = "ohm", default = 100.0)]
    pub load_ohm: f64,
    #[parameter(
        label = "Outage Durations",
        unit = "s",
        expansion = zip("outage"),
        min = 0.0,
        min_items = 1,
        max_items = 16,
        default = ParameterDefault::literal(dokime::core::Value::Array(::std::vec![
            dokime::core::Value::Number(0.05),
            dokime::core::Value::Number(0.1),
            dokime::core::Value::Number(0.2),
        ])), description = "How long the Charger stays without power on each cycle, in model seconds.")]
    pub outage_durations_s: Vec<f64>,
    #[parameter(
        label = "Recovery Limits",
        unit = "s",
        expansion = zip("outage"),
        min = SHORTEST_S,
        min_items = 1,
        max_items = 16,
        default = ParameterDefault::literal(dokime::core::Value::Array(::std::vec![
            dokime::core::Value::Number(0.02),
            dokime::core::Value::Number(0.05),
            dokime::core::Value::Number(0.1),
        ])), description = "How long the restart after each outage is allowed to take, in model seconds, read in step with [`Recovery::outage_durations_s`].")]
    pub recovery_limits_s: Vec<f64>,
    #[parameter(label = "Supply Frequency", unit = "Hz", default = 50.0)]
    pub frequency_hz: f64,
    #[parameter(label = "Current Phase Lag", unit = "deg", default = 15.0)]
    pub phase_deg: f64,
    #[parameter(label = "RMS Current Limit", unit = "A", default = 5.0)]
    pub current_limit_a: f64,
    #[parameter(label = "Observation Duration", unit = "s", default = 0.2, min = SHORTEST_S)]
    pub duration_s: f64,
    #[parameter(
        label = "Simulation Sample Interval",
        unit = "s",
        default = 0.001,
        min = FINEST_STEP_S,
        max = COARSEST_STEP_S,
    )]
    pub sample_interval_s: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
