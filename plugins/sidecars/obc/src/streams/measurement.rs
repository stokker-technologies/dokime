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
use crate::device::Bench;
use crate::device::Reading;
use crate::{settings, suites};
use dokime::builders::Plot;
use dokime::plugin::{Observer, Stream};
use dokime::templates::Context;
use dokime::{Document, FieldReference, Result, model};

/* === Constants === */

// Every Charger Procedure Shares the One Plugin Measurement Stream.
pub const SAMPLES: &str = "obc.measurement";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Charger Sample: AC Input Waveform, RMS Values and DC Output.
#[derive(dokime::Row)]
#[dokime(package = "obc")]
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

pub fn catalog(template: dokime::templates::Template) -> dokime::templates::Template {
    let stream = Stream::<Measurement>::new(SAMPLES, "Charger Measurements");
    let observer = Observer::new(&stream, measure).interval_setting(settings::SAMPLE_INTERVAL);
    template.stream(stream).observer(observer)
}

async fn measure(ctx: &Context) -> Result<Measurement> {
    if !suites::captures(ctx, SAMPLES) {
        return Ok(Measurement::default());
    }
    let (index, reading, phase) = ctx.get::<Bench>()?.sample();
    Ok(Measurement::new(index, reading, phase))
}

/// AC Waveforms Against Model Time and the Battery Charging Outputs.
pub fn document(id: &str, title: &str) -> Document {
    let mut waveforms = model::Section::new("AC Waveforms").id(format!("{id}.waveforms"));
    for (name, label, unit) in [("voltage_v", "Voltage", "V"), ("current_a", "Current", "A")] {
        let chart = Plot::xy(FieldReference::new(SAMPLES).field(name))
            .id(format!("{id}.{name}"))
            .label(label)
            .unit(unit)
            .x(FieldReference::new(SAMPLES).field("elapsed_s"));
        waveforms = waveforms.plot(chart);
    }
    Document::new(title).id(format!("{id}.report")).section(waveforms).section(charging(id))
}

/// Battery Voltage, Charging Current and AC Input Power Over Time.
pub fn charging(id: &str) -> model::Section {
    let mut section = model::Section::new("Battery Charging").id(format!("{id}.charging"));
    for (name, label, unit) in
        [("dc_voltage_v", "Battery Voltage", "V"), ("dc_current_a", "Charging Current", "A"), ("input_power_w", "AC Input Power", "W")]
    {
        let chart = Plot::time_series(FieldReference::new(SAMPLES).field(name)).id(format!("{id}.{name}")).label(label).unit(unit);
        section = section.plot(chart);
    }
    section
}

/* === Private Functions === */

/* === Tests === */
