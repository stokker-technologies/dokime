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
use crate::domain::model::charger::Reading;
use dokime::expressions::telemetry;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::row]
#[derive(Clone)]
pub struct Row {
    #[row(role = counter)]
    pub sample_index: i64,

    #[row(label = "Elapsed", unit = "s")]
    pub elapsed_s: f64,

    #[row(label = "Voltage", unit = "V")]
    pub voltage_v: f64,

    #[row(label = "Current", unit = "A")]
    pub current_a: f64,

    #[row(label = "RMS Voltage", unit = "V")]
    pub voltage_rms_v: f64,

    #[row(label = "RMS Current", unit = "A")]
    pub current_rms_a: f64,

    #[row(label = "Frequency", unit = "Hz")]
    pub frequency_hz: f64,

    #[row(label = "Current Phase", unit = "deg")]
    pub phase_deg: f64,

    #[row(label = "AC Input Power", unit = "W")]
    pub input_power_w: f64,

    #[row(label = "Battery Voltage", unit = "V")]
    pub dc_voltage_v: f64,

    #[row(label = "Charging Current", unit = "A")]
    pub dc_current_a: f64,

    #[row(role = state)]
    pub phase: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn row(index: u64, reading: Reading, phase: &str) -> Row {
    Row {
        sample_index: index as i64,
        elapsed_s: reading.elapsed_s,
        voltage_v: reading.voltage_v,
        current_a: reading.current_a,
        voltage_rms_v: reading.voltage_rms_v,
        current_rms_a: reading.current_rms_a,
        frequency_hz: reading.frequency_hz,
        phase_deg: reading.phase_deg,
        input_power_w: reading.input_power_w,
        dc_voltage_v: reading.dc_voltage_v,
        dc_current_a: reading.dc_current_a,
        phase: phase.into(),
    }
}

pub(crate) fn document(stream: &str, title: &str) -> Document {
    let mut voltage = Plot::xy("Voltage");
    voltage.id(format!("{stream}.voltage"));
    voltage.unit("V");
    voltage.x(telemetry(stream, "elapsed_s"));
    voltage.y(telemetry(stream, "voltage_v"));

    let mut current = Plot::xy("Current");
    current.id(format!("{stream}.current"));
    current.unit("A");
    current.x(telemetry(stream, "elapsed_s"));
    current.y(telemetry(stream, "current_a"));

    let mut waveform = Section::new("AC Waveforms");
    waveform.plot(voltage);
    waveform.plot(current);

    let mut document = Document::new("default", title);
    document.section(waveform);

    let mut charging = Section::new("Battery Charging");
    for (column, label, unit) in [
        (Row::column_dc_voltage_v(), "Battery Voltage", "V"),
        (Row::column_dc_current_a(), "Charging Current", "A"),
        (Row::column_input_power_w(), "AC Input Power", "W"),
    ] {
        let mut plot = Plot::xy(label);
        plot.id(format!("{stream}.{}", column.field()));
        plot.unit(unit);
        plot.x(Row::column_elapsed_s().on(stream));
        plot.y(column.on(stream));
        charging.plot(plot);
    }
    document.section(charging);

    document
}

/* === Private Functions === */

/* === Tests === */
