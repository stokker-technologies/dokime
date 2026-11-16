/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bench: A Ten-Module Charger at Scale and Its Exact Revenue Meter Register

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::parameters;
use crate::catalog::{field, number, plot, records as records_card};
use crate::device::{Charger, Power, Supply};
use crate::prompts::{BENCH_CONFIRM, confirm};
use crate::records::Confirmation;
use crate::settings;
use crate::testbench::{self, Testbench};
use dokime::plugin::{Context, Criterion, Observer, Record, SampleWindow, Step, Stream, Suite, Template};
use dokime::{Aggregate, Document, PlotKind, Result, Section, Severity, Summary, value};

/* === Constants === */

pub const SUITE: &str = "obc.bench";
pub const TEMPLATE: &str = "obc.bench.module_array";
pub const METER: &str = "obc.bench.meter";
pub const CONFIRMATION: &str = "obc.bench.confirmation";

/// The Revenue Meter's 64-Bit Energy Register, Held While the Capture Runs.
pub const REGISTER: u64 = 1_152_921_504_606_847_975;

/// Power Modules in the Charger Under Test.
pub const MODULES: usize = 10;

/// Measured Channels per Module: Field, Label and Unit.
pub const CHANNELS: [(&str, &str, &str); 10] = [
    ("ac_voltage_v", "AC Voltage", "V"),
    ("ac_current_a", "AC Current", "A"),
    ("input_power_w", "AC Input Power", "W"),
    ("power_factor", "Power Factor", ""),
    ("dc_voltage_v", "DC Link Voltage", "V"),
    ("dc_current_a", "DC Output Current", "A"),
    ("output_power_w", "DC Output Power", "W"),
    ("efficiency_percent", "Efficiency", "%"),
    ("heatsink_c", "Heatsink Temperature", "°C"),
    ("switching_khz", "Switching Frequency", "kHz"),
];

// Each Module Shares the Charger's Input Equally.
const SHARE: f64 = 1.0 / MODULES as f64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Power Module's Telemetry at One Instant.
#[derive(dokime::Row)]
#[harmos(package = "obc")]
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
#[harmos(package = "obc")]
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

/// The Exact Integer the Revenue Meter Reports Every Sample.
#[derive(dokime::Row)]
#[harmos(package = "obc")]
pub struct MeterRegister {
    pub energy_register: u64,
}

/// One Window of Meter Register Readings.
#[derive(dokime::Message)]
#[harmos(package = "obc")]
pub struct MeterRegisterColumns {
    pub energy_register: Vec<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> Record<Confirmation> {
    Record::new(CONFIRMATION, "Bench Readings Confirmed")
        .description("The operator's confirmation of the module readings and meter register.")
}

pub fn stream(module: usize) -> String {
    format!("obc.bench.module_{:02}", module + 1)
}

pub fn suite() -> Suite {
    let template = Template::new(TEMPLATE, "Module Array Capture")
        .description("Capture all ten power modules at full sample rate with the exact revenue meter register, then confirm the readings.")
        .tag("bench");
    let mut template = parameters(template, [number("duration_s", "Capture Duration", "s", 0.25, Some(0.001), Some(3600.0))])
        .initialize(initialize)
        .step(Step::new("Energize Module Array", energize).identifier("energize"))
        .step(Step::new("Capture Module Telemetry", capture).identifier("capture"))
        .step(Step::new("Confirm Bench Readings", confirmation).identifier("confirmation"))
        .criterion(
            Criterion::new("Exact Meter Register", exact)
                .identifier("exact_register")
                .description("Passed once every saved register value is the exact 64-bit reading and the operator confirmed it."),
        )
        .cleanup(cleanup);
    for module in 0..MODULES {
        let telemetry = Stream::<ModuleTelemetry>::new(stream(module), format!("Module {} Telemetry", module + 1));
        let observer = Observer::window(&telemetry, async move |ctx: &Context, window: &SampleWindow| observe(ctx, module, window).await)
            .interval_setting(settings::SAMPLE_INTERVAL);
        template = template.stream(telemetry).observer(observer);
    }
    let meter =
        Stream::<MeterRegister>::new(METER, "Revenue Meter Register").description("The bench meter's exact 64-bit energy register.");
    let register = Observer::window(&meter, read_register).interval_setting(settings::SAMPLE_INTERVAL);
    let template = template.stream(meter).observer(register).document(document());
    Suite::new(SUITE, "Bench")
        .description("Scale and exactness workloads on a realistic charger bench: many channels, long histories, exact integers.")
        .tag("bench")
        .template(template)
}

/// Exact Register Statistics, Then Ten Channels for Each of the Ten Modules: One Hundred Plots.
pub fn document() -> Document {
    let register = || field(METER, "energy_register");
    let mut summary = Summary::new();
    for (name, kind, rank) in [
        ("Count", Aggregate::Count, 0.0),
        ("Sum", Aggregate::Sum, 0.0),
        ("Mean", Aggregate::Mean, 0.0),
        ("Median", Aggregate::Median, 0.0),
        ("Min", Aggregate::Min, 0.0),
        ("Max", Aggregate::Max, 0.0),
        ("First", Aggregate::First, 0.0),
        ("Last", Aggregate::Last, 0.0),
        ("P05", Aggregate::Percentile, 5.0),
        ("P95", Aggregate::Percentile, 95.0),
        ("P99", Aggregate::Percentile, 99.0),
    ] {
        let value = if kind == Aggregate::Percentile { register().percentile(rank) } else { register().aggregate(kind) };
        summary = summary.tile(name, value);
    }
    let metering =
        Section::new("Metering").id("obc.bench.metering").summary("Exact Statistics", summary).card(records_card("obc.bench.records"));
    let mut document = Document::new("Module Array").id("obc.bench.module_array.report").section(metering);
    for module in 0..MODULES {
        let stream = stream(module);
        let mut section = Section::new(format!("Module {}", module + 1)).id(stream.clone());
        for (name, label, unit) in CHANNELS {
            let mut chart = plot(format!("{stream}.{name}"), label, PlotKind::TimeSeries, unit);
            chart.y = vec![field(&stream, name)];
            section = section.plot(chart);
        }
        document = document.section(section);
    }
    document
}

/* === Private Functions === */

async fn initialize(ctx: &Context) -> Result<()> {
    Testbench::open(ctx, Supply::default(), None).await
}

async fn energize(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Testbench>()?;
    let mut state = bench.state();
    let now_s = state.now_s();
    state.charger.start(now_s)?;
    state.label = "capture";
    Ok(())
}

async fn capture(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Testbench>()?;
    let samples = bench.samples_for(ctx.parameter("duration_s")?)?;
    bench.wait(ctx, samples).await?;
    ctx.log(Severity::Info, format!("Captured {samples} Samples per Module")).await
}

async fn confirmation(ctx: &Context) -> Result<()> {
    let accepted = confirm(ctx, BENCH_CONFIRM).await?;
    ctx.record(&contracts(), Confirmation { accepted }).await?;
    Ok(())
}

async fn exact(ctx: &Context) -> Result<bool> {
    let Some(statistics) = ctx.statistics(METER, "energy_register", "").await?.statistics else {
        return Ok(false);
    };
    let exact = |value: Option<dokime::Value>| matches!(value.and_then(|value| value.kind), Some(value::Kind::UnsignedInteger(REGISTER)));
    let confirmed = crate::records::last(ctx, &contracts(), "").await?.is_some_and(|record| record.accepted);
    Ok(statistics.count > 0 && exact(statistics.min) && exact(statistics.max) && confirmed)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Testbench>()?;
    bench.state().charger.stop();
    ctx.log(Severity::Info, "Module Array De-Energized").await
}

/// One Window of a Module's Telemetry, Each Sample Read From the Charger at Its Own Model Time.
async fn observe(ctx: &Context, module: usize, window: &SampleWindow) -> Result<ModuleTelemetryColumns> {
    let mut columns = ModuleTelemetryColumns::default();
    let samples = usize::try_from(window.count).map_err(|_| dokime::Error::Invalid("Observe Module: Shorten the Window".into()))?;
    if !testbench::captures(ctx, &stream(module)) {
        let zeros = vec![0.0; samples];
        for channel in columns.channels() {
            *channel = zeros.clone();
        }
        return Ok(columns);
    }
    let bench = ctx.get::<Testbench>()?;
    let state = bench.state();
    // Sample `i` of a Window Is the Model's Sample `sequence - 1 + i`, Whatever Order Streams Answer In
    for index in 0..window.count {
        let at_s = state.time_s(window.sequence - 1 + index);
        let reading = telemetry(&state.charger, module, at_s);
        for (channel, value) in columns.channels().into_iter().zip(reading.values()) {
            channel.push(value);
        }
    }
    Ok(columns)
}

async fn read_register(ctx: &Context, window: &SampleWindow) -> Result<MeterRegisterColumns> {
    if testbench::captures(ctx, METER) {
        // The Meter Advances the Shared Model Clock by Every Sample of Its Window
        ctx.get::<Testbench>()?.advance_by(window.count);
    }
    let samples = usize::try_from(window.count).map_err(|_| dokime::Error::Invalid("Read Register: Shorten the Window".into()))?;
    Ok(MeterRegisterColumns { energy_register: vec![REGISTER; samples] })
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
    let ripple = (std::f64::consts::TAU * (reading.frequency_hz * 2.0) * at_s + offset).sin();
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
