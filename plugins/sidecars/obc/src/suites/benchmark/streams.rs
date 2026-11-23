/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Many Independent Charger Module Streams

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::f64::consts::TAU;

// Third Party Imports

// Local Imports
use crate::records::charger;
use crate::suites::{self, parameters};

use crate::device::Bench;
use crate::device::{Charger, Power, Supply};
use crate::settings;
use dokime::plugin::{Criterion, Observer, Step, Stream, Template, Window};
use dokime::templates::Context;
use dokime::{Document, Error, Parameter, Result, Severity};

/* === Constants === */

pub const TEMPLATE: &str = "obc.benchmark.streams";

/// Power Modules in the Charger Under Test.
pub const MODULES: usize = 64;

// Each Module Shares the Charger's Input Equally.
const SHARE: f64 = 1.0 / MODULES as f64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Power Module's Telemetry at One Instant.
#[derive(dokime::Row)]
#[dokime(package = "obc")]
pub struct ModuleTelemetry {
    pub ac_voltage_v: f64,
    pub ac_current_a: f64,
    pub input_power_w: f64,
    pub power_factor: f64,
    pub dc_voltage_v: f64,
    pub dc_current_a: f64,
    pub output_power_w: f64,
    pub efficiency_percent: f64,
    pub heatsink_c: f64,
    pub switching_khz: f64,
}

/// One Window of Module Telemetry, One Value per Sample in Every Channel.
#[derive(dokime::Message)]
#[dokime(package = "obc")]
pub struct ModuleTelemetryColumns {
    pub ac_voltage_v: Vec<f64>,
    pub ac_current_a: Vec<f64>,
    pub input_power_w: Vec<f64>,
    pub power_factor: Vec<f64>,
    pub dc_voltage_v: Vec<f64>,
    pub dc_current_a: Vec<f64>,
    pub output_power_w: Vec<f64>,
    pub efficiency_percent: Vec<f64>,
    pub heatsink_c: Vec<f64>,
    pub switching_khz: Vec<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn stream(module: usize) -> String {
    format!("obc.module_{:02}", module + 1)
}

pub fn template() -> Template {
    let template = Template::of::<()>()
        .identifier("streams")
        .label("Module Array Capture")
        .description("Capture sixty-four independent module streams at full sample rate.")
        .tag("bench");
    let template = charger::catalog(template);
    let mut template =
        parameters(template, [Parameter::number("duration_s", "Capture Duration").unit("s").default(0.25).range(0.001..=3600.0).build()])
            .initialize(initialize)
            .step(Step::new("Energize Module Array", energize).identifier("energize"))
            .step(Step::new("Capture Module Telemetry", capture).identifier("capture"))
            .criterion(
                Criterion::new("Module Streams Captured", captured)
                    .identifier("captured")
                    .description("Passed once every declared module stream contains captured telemetry."),
            )
            .cleanup(cleanup);
    for module in 0..MODULES {
        let telemetry = Stream::<ModuleTelemetry>::new(format!("module_{:02}", module + 1), format!("Module {} Telemetry", module + 1));
        let observer = Observer::window(&telemetry, async move |ctx: &Context, window: &Window| observe(ctx, module, window).await)
            .interval_setting(settings::SAMPLE_INTERVAL);
        template = template.stream(telemetry).observer(observer);
    }
    template.document(Document::new("Stream Capture").id(format!("{TEMPLATE}.report")))
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    suites::open(ctx, Supply::default(), None).await
}

async fn energize(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    bench.start()?;
    bench.label("capture");
    Ok(())
}

async fn capture(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    let samples = bench.samples_for(ctx.parameter("duration_s")?)?;
    suites::wait(ctx, &bench, samples).await?;
    ctx.log(Severity::Info, format!("Captured {samples} Samples per Module")).await
}

async fn captured(ctx: &Context) -> Result<bool> {
    for module in 0..MODULES {
        let count = ctx.statistics(&stream(module), "ac_voltage_v", "").await?.map_or(0, |statistics| statistics.count);
        if count == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    bench.stop();
    ctx.log(Severity::Info, "Module Array De-Energized").await
}

/// One Window of a Module's Telemetry, Each Sample Read From the Charger at Its Own Model Time.
async fn observe(ctx: &Context, module: usize, window: &Window) -> Result<ModuleTelemetryColumns> {
    let mut columns = ModuleTelemetryColumns::default();
    let samples = usize::try_from(window.count).map_err(|_| Error::Invalid("Observe Module: Shorten the Window".into()))?;
    if !suites::captures(ctx, &stream(module)) {
        let zeros = vec![0.0; samples];
        for channel in columns.channels() {
            *channel = zeros.clone();
        }
        return Ok(columns);
    }
    let bench = ctx.get::<Bench>()?;
    let charger = bench.charger();
    // Sample `i` of a Window Is the Model's Sample `sequence - 1 + i`, Whatever Order Streams Answer In
    for index in 0..window.count {
        let at_s = bench.time_s(window.sequence - 1 + index);
        let reading = telemetry(&charger, module, at_s);
        for (channel, value) in columns.channels().into_iter().zip(reading.values()) {
            channel.push(value);
        }
    }
    if module == 0 {
        bench.advance_by(window.count);
    }
    Ok(columns)
}

impl ModuleTelemetry {
    fn values(&self) -> [f64; 10] {
        [
            self.ac_voltage_v,
            self.ac_current_a,
            self.input_power_w,
            self.power_factor,
            self.dc_voltage_v,
            self.dc_current_a,
            self.output_power_w,
            self.efficiency_percent,
            self.heatsink_c,
            self.switching_khz,
        ]
    }
}

impl ModuleTelemetryColumns {
    fn channels(&mut self) -> [&mut Vec<f64>; 10] {
        [
            &mut self.ac_voltage_v,
            &mut self.ac_current_a,
            &mut self.input_power_w,
            &mut self.power_factor,
            &mut self.dc_voltage_v,
            &mut self.dc_current_a,
            &mut self.output_power_w,
            &mut self.efficiency_percent,
            &mut self.heatsink_c,
            &mut self.switching_khz,
        ]
    }
}

fn telemetry(charger: &Charger, module: usize, at_s: f64) -> ModuleTelemetry {
    let reading = charger.reading(at_s);
    let running = charger.power == Power::Running;
    let offset = module as f64;
    let input_power_w = reading.input_power_w * SHARE;
    let efficiency = if running { 0.94 - 0.002 * offset } else { 0.0 };
    let output_power_w = input_power_w * efficiency;
    let ripple = (TAU * (reading.frequency_hz * 2.0) * at_s + offset).sin();
    ModuleTelemetry {
        ac_voltage_v: reading.voltage_rms_v,
        ac_current_a: reading.current_rms_a * SHARE,
        input_power_w,
        power_factor: if running { reading.phase_deg.to_radians().cos() } else { 0.0 },
        dc_voltage_v: if running { reading.dc_voltage_v + 1.5 * ripple } else { 0.0 },
        dc_current_a: if running { output_power_w / reading.dc_voltage_v } else { 0.0 },
        output_power_w,
        efficiency_percent: efficiency * 100.0,
        heatsink_c: 25.0 + offset * 0.8 + if running { 15.0 * (1.0 - (-at_s / 0.1).exp()) } else { 0.0 },
        switching_khz: if running { 100.0 + offset * 0.5 + 0.2 * ripple } else { 0.0 },
    }
}

/* === Tests === */
