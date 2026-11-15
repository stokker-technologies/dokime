/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Measurement Rows and Their Shared Document Sections

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::catalog::{field, plot};
use crate::device::Reading;
use dokime::{Document, PlotKind, Section};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Charger Sample: AC Input Waveform, RMS Values and DC Output.
#[derive(dokime::Row)]
#[harmos(package = "obc")]
pub struct Measurement {
    pub sample_index: i64,
    pub elapsed_s: f64,
    pub voltage_v: f64,
    pub current_a: f64,
    pub voltage_rms_v: f64,
    pub current_rms_a: f64,
    pub frequency_hz: f64,
    pub phase_deg: f64,
    pub input_power_w: f64,
    pub dc_voltage_v: f64,
    pub dc_current_a: f64,
    pub phase: String,
}

/// One Point of the Bench's Thermal Channel Survey: a Heatsink Sensor's Rise Above Ambient.
#[derive(dokime::Row)]
#[harmos(package = "obc")]
pub struct Survey {
    pub sweep: f64,
    pub module: f64,
    pub sensor: f64,
    pub rise_k: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Measurement {
    pub fn new(index: u64, reading: Reading, phase: &str) -> Self {
        Self {
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
}

/* === Public Functions === */

/// AC Waveforms Against Model Time and the Battery Charging Outputs.
pub fn document(stream: &str, id: &str, title: &str) -> Document {
    let mut waveforms = Section::new("AC Waveforms").id(format!("{id}.waveforms"));
    for (name, label, unit) in [("voltage_v", "Voltage", "V"), ("current_a", "Current", "A")] {
        let mut chart = plot(format!("{stream}.{name}"), label, PlotKind::Xy, unit);
        chart.x = Some(field(stream, "elapsed_s"));
        chart.y.push(field(stream, name));
        waveforms = waveforms.plot(chart);
    }
    Document::new(title).id(format!("{id}.report")).section(waveforms).section(charging(stream, id))
}

/// Battery Voltage, Charging Current and AC Input Power Over Time.
pub fn charging(stream: &str, id: &str) -> Section {
    let mut section = Section::new("Battery Charging").id(format!("{id}.charging"));
    for (name, label, unit) in
        [("dc_voltage_v", "Battery Voltage", "V"), ("dc_current_a", "Charging Current", "A"), ("input_power_w", "AC Input Power", "W")]
    {
        let mut chart = plot(format!("{stream}.{name}"), label, PlotKind::TimeSeries, unit);
        chart.y.push(field(stream, name));
        section = section.plot(chart);
    }
    section
}

/* === Private Functions === */

/* === Tests === */
