/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
// Third Party Imports

// Local Imports
use crate::bench::Heater;
use crate::domain::model::prompts::SoakCheckpoint;
use crate::domain::operations::ReadChamber;
use crate::{Error, Result};
use dokime::host::ToolConnectionAcquired;
use dokime::prelude::*;

/* === Constants === */

const RAMP_POLL: Duration = Duration::from_millis(10);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A record is a type, declared once, in the file of whatever publishes it —
// here, beside the step that files it. The struct's fields *are* the contract's
// fields, so there is no id to keep in agreement with a separate declaration
// and no detail that can be published without being declared. The template
// names it once in `records = Records` below, which is what binds the
// contract to this template and tells the host what to expect. Its explicit `description` is the
// text the host shows.
#[record(id = "{{crate}}.soak.held", label = "Hold reached", severity = info, description = "One completed hold, with what it measured.")]
#[derive(Clone, Debug)]
pub struct HoldReached {
    #[record(description = "How many samples the chamber was held for.")]
    pub samples: u64,
    #[record(unit = "°C", description = "What it was at when the hold ended.")]
    pub celsius: f64,
    #[record(description = "The unit the bench was told about, when it was told about one.")]
    pub unit: Option<String>,
}

#[parameters]
pub struct SoakParameters {
    #[parameter(
        label = "Target temperature",
        unit = "°C",
        default = 60.0,
        description = "What the chamber is driven to. Deliberately unbounded above: a target the bench cannot reach is something this procedure *discovers*, and refusing it at admission would hide the failure the run is there to find."
    )]
    pub target_c: f64,
    #[parameter(label = "Allowed band", unit = "°C", default = 1.5, min = 0.0)]
    pub band_c: f64,
    #[parameter(label = "Samples to hold", default = 5, min = 1)]
    pub hold_samples: u64,
    #[parameter(label = "Heater port", default = "/dev/ttyUSB0")]
    pub heater_port: String,
    #[parameter(label = "Seconds to wait for the unit", unit = "s", default = 0.5, min = 0.0)]
    pub connect_budget_s: f64,
    #[parameter(label = "Highest host CPU load allowed", unit = "%", default = 100.0, min = 0.0, max = 100.0)]
    pub host_cpu_limit: f64,
    #[parameter(label = "Ask the operator to sign off", default = false)]
    pub attended: bool,
}

pub struct Hold {
    unit: Option<String>,
    note: Option<String>,
}

pub struct SoakResources {
    pub heater: tokio::sync::Mutex<Heater>,
    holding: AtomicBool,
}

// The row names its home, which is the only place this stream's id is
// written: a step waits on these by type with `ctx.listen::<SoakRow>()`, a
// criterion reads them with `ctx.samples::<SoakRow>()`, and the
// `#[observer]` below adds only the cadence it publishes them on.
// `#[stream]` writes the columns as well, so this does not also derive
// `#[dokime::row]`, and every column comes with a handle a document names
// it by — rename `celsius` and the plot above stops compiling rather than
// quietly drawing nothing.
#[stream(
    id = "{{crate}}.soak.samples",
    window = 4096,
    label = "Soak Samples",
    description = "The chamber, once per interval, for the length of the attempt."
)]
pub struct SoakRow {
    #[row(unit = "°C", format = fixed(1))]
    pub celsius: f64,
    #[row(role = state)]
    pub holding: bool,
}

#[derive(serde::Serialize)]
struct Summary<'a> {
    target_c: f64,
    band_c: f64,
    hold_samples: u64,
    final_c: f64,
    unit: Option<&'a str>,
    operator_note: Option<&'a str>,
}

/* === Enums === */

#[dokime::catalog]
pub enum Required {
    ReadChamber(ReadChamber),
}

#[dokime::catalog]
pub enum Prompts {
    SoakCheckpoint(SoakCheckpoint),
}

#[dokime::catalog]
pub enum Records {
    HoldReached(HoldReached),
}

#[dokime::catalog]
pub enum Inputs {
    System(dokime_host::System),
    ToolConnectionAcquired(ToolConnectionAcquired),
}

#[dokime::catalog]
pub enum ObserverStreams {
    SoakRow(SoakRow),
}

/* === Traits === */

/* === Implementations === */

#[template(
    id = "{{crate}}.soak.hold",
    label = "Hold at Temperature",
    tags = ["thermal", "soak"],
    // The operations this template calls. A template never names the plugin it
    // runs under: it names the operation types it will call, and the plugin
    // serving this suite is checked, once, against exactly this list.
    requires = Required,
    // The question the operator check asks, named by its type the way the
    // records are: the struct in `prompts/checkpoint.rs` is the declaration.
    prompts = Prompts,
    // What this template's steps publish. The contract comes off the struct in
    // this file, so this list is the whole declaration.
    records = Records,
    // The global streams the host leases for every execution of this
    // template, named by type the way this template's own rows are. Each
    // stream's observer declared how often it measures, so nothing here
    // states a cadence and every run leasing the same stream shares one
    // running instance and its tick.
    //
    // `dokime_host::System` is the machine itself: cpu, memory and battery. The other
    // five — `dokime_host::Network`, `dokime_host::WifiLink`, `dokime_host::WifiScan`, `dokime_host::WifiLatency`,
    // `dokime_host::Bluetooth` — are what a bench adds
    // when it is correlating the machine's radios against the unit.
    subscribes = Inputs,
    // The inputs, named once. The struct above is the declaration; every stage
    // reads the decoded value through `ctx.parameters()`.
    parameters = SoakParameters, description = "Drive the chamber to the target and hold it for the requested samples.")]
impl Hold {
    fn document() -> Document {
        let mut temperature = Plot::time_series("Temperature");
        temperature.id("{{crate}}.soak.temperature");
        temperature.unit("°C");
        temperature.y(SoakRow::column_celsius());
        // The row timestamp is automatic and is never named on the axis — a
        // `time_series` plot always draws against it. `x_format` only says how
        // that axis is *read*: seconds since this attempt started, rather than
        // a wall clock.
        temperature.x_format(format::elapsed_seconds());
        temperature.y_format(format::fixed(1));
        // The band is this case's own `target_c ± band_c`, so a plot drawn for
        // one case shows the band it was held to. Both are named by their typed
        // handles, the way the axes are named by their columns.
        temperature.reference_band(
            "target ± band",
            SoakRow::column_celsius(),
            Band::around(SoakParameters::target_c(), SoakParameters::band_c()),
        );
        temperature.marker_records([HoldReached::marker()]);

        let mut signals = Section::new("Soak");
        signals.id("{{crate}}.soak.signals");
        signals.plot(temperature);

        let mut document = Document::new("default", "Hold at Temperature");
        document.section(signals);

        document
    }

    async fn initialize(_: &SoakParameters, ctx: &mut InitializationContext<'_>) -> Result<(Self, SoakResources), Error> {
        let template = Self { unit: None, note: None };

        let port = ctx.parameters().heater_port.clone();
        // Every context logs, including the ones handed out by shared
        // reference: a log line is a send onto the host's own queue, so a hook
        // that opens handles can say what it is opening.
        dokime::info!(ctx, "opening the heater on {port}");
        let heater = Heater::open(port);

        let shared = SoakResources {
            heater: tokio::sync::Mutex::new(heater.await?),
            holding: AtomicBool::new(false),
        };
        Ok((template, shared))
    }

    async fn traceability(&self, ctx: &TraceabilityContext<'_>) -> Result<Vec<TraceabilitySection>, Error> {
        let identity = ctx.resources().heater.lock().await.identity().await?;
        // What the bench is, read off the same operation that reads what it is
        // at: the reply is all a template knows about the process serving it.
        let bench = ctx.call::<ReadChamber>(ReadChamber {}).await?;

        // Device Under Test
        let mut dut = TraceabilitySection::new("dut", "Device under test");
        dut.row(TraceabilityRow::text("serial_number", "Serial number", identity.serial));
        dut.row(TraceabilityRow::text("firmware", "Firmware", identity.firmware));

        // Bench
        let mut bench_section = TraceabilitySection::new("bench", "Bench");
        bench_section.row(TraceabilityRow::text("sensor", "Sensor", bench.sensor));
        bench_section.row(TraceabilityRow::number("target_c", "Target", ctx.parameters().target_c).unit("°C"));

        Ok(vec![dut, bench_section])
    }

    #[step(
        label = "Wait for the Unit",
        description = "Waits for somebody else to say the unit is on the bench.\n`ctx.listen::<T>()` is the one way a step waits, and it answers a standard stream — so `.timeout(…)`, `.next()` and the rest are the adapters Rust already has, imported once from `dokime::prelude`. [`ToolConnectionAcquired`] is the host's own record type, so this step names no id and reads no detail map: the unit's identity is a field of the fact it waited for.\nAn execution's record inbox starts where the run started, so a unit that came online before this step reached its await still satisfies it.\nA bench with no gateway beside it is told about no unit, so a deadline here is a note rather than a failure. Make it a `?` the day another plugin is responsible for saying so."
    )]
    async fn connected(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let budget = Duration::from_secs_f64(ctx.parameters().connect_budget_s);
        match ctx.listen::<ToolConnectionAcquired>().timeout(budget).next().await {
            Some(Ok(connected)) => {
                dokime::info!(ctx, "unit {} is on the bench", connected.resource_id);
                self.unit = Some(connected.resource_id);
            }
            Some(Err(waited)) => {
                dokime::warn!(ctx, "{waited}; testing whatever is on the bench");
            }
            None => ctx.warn("nothing is listening for units on this bench"),
        }

        Ok(())
    }

    #[step(label = "Reach Target", description = "Drives the chamber until it is inside the band.")]
    async fn reach(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let SoakParameters { target_c, band_c, .. } = *ctx.parameters();

        // === Set The Target === //

        ctx.resources().heater.lock().await.set_target(target_c).await?;
        dokime::info!(ctx, "heater set to {target_c:.1} °C");

        // === Wait For The Band === //

        loop {
            // The bench's own operation rather than this attempt's handle on
            // the sensor: what the chamber is at is the plugin's to answer, and
            // the host routes the call under the operation's declared policy.
            // `ctx.resources()` below stays where it is the point — the heater
            // is this attempt's alone.
            let celsius = ctx.call::<ReadChamber>(ReadChamber {}).await?.celsius;
            if (celsius - target_c).abs() <= band_c {
                dokime::info!(ctx, "target reached at {celsius:.2} °C");
                return Ok(());
            }
            tokio::time::sleep(RAMP_POLL).await;
        }
    }

    #[step(
        label = "Hold",
        description = "Holds, writing the chamber log as it goes.\nThe step waits on its own observer by the type it publishes — an exact number of rows rather than a length of time, so what the run measured does not depend on how quickly it got going — and appends one line per row through an artifact writer, so the evidence reaches the host while the hold is still running rather than all at once at the end. The row carries the measurement, so the step reads the chamber once, in the observer, rather than twice. What it measured is published as one `HoldReached`, which is a struct rather than a map of details nothing checks until admission."
    )]
    async fn hold(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // === Open The Chamber Log === //

        let mut log = ctx.artifacts().create("heater.log")?;
        // Subscribed before the chamber is marked as holding, so the very next
        // row the observer measures is one this step counts. A stream is a
        // stream of measurements, so a subscription sees what is measured after
        // it is taken and nothing before it.
        let mut held = ctx.listen::<SoakRow>();
        ctx.resources().holding.store(true, Ordering::Release);

        // === Count The Hold === //

        let wanted = ctx.parameters().hold_samples;
        let mut celsius = ctx.parameters().target_c;
        let mut sample = 0;
        while sample < wanted {
            let Some(row) = held.next().await else { break };
            sample += 1;
            celsius = row.celsius;
            writeln!(log, "{sample},{celsius:.2}")?;
        }
        // A subscription that fell behind lost rows, and this hold counted
        // fewer than it measured. Worth saying so rather than reporting a
        // shorter hold as if it were the one that was asked for.
        if held.missed() > 0 {
            dokime::warn!(ctx, "the hold missed {} rows it measured", held.missed());
        }
        log.finish()?;
        dokime::info!(ctx, "held for {sample} samples, ending at {celsius:.2} °C");

        // === Publish What It Measured === //

        let summary = serde_json::to_vec_pretty(&self.summary(ctx.parameters(), celsius))?;
        ctx.artifacts().write("summary.json", summary)?;
        // One verb publishes both kinds of evidence, and the declaration decides
        // which this is: `HoldReached` is a record, `SoakRow` is a measurement,
        // and neither call site repeats where it goes.
        ctx.publish(HoldReached {
            samples: wanted,
            celsius,
            unit: self.unit.clone(),
        })
        .await;

        Ok(())
    }

    #[step(label = "Operator Check", description = "Asks the operator to sign off, on a bench that has one.")]
    async fn check(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // === Skip An Unattended Bench === //

        if !ctx.parameters().attended {
            ctx.info("unattended bench: no operator was asked");
            return Ok(());
        }

        // === Ask The Operator === //

        let answer = ctx.ask(SoakCheckpoint::default()).await?;
        dokime::info!(ctx, "operator note: {}", answer.note);
        self.note = Some(answer.note);
        if !answer.accepted {
            ctx.warn("the operator did not sign off on this soak");
        }

        Ok(())
    }

    #[observer(interval_ms = 100, streams = ObserverStreams)]
    async fn observe(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        let row = SoakRow {
            celsius: ctx.call::<ReadChamber>(ReadChamber {}).await?.celsius,
            holding: ctx.resources().holding.load(Ordering::Acquire),
        };
        ctx.publish(row).await;

        Ok(())
    }

    #[criterion(
        id = "{{crate}}.soak.in_band",
        label = "Temperature stayed in band",
        description = "Did the chamber stay in band for the whole hold?\nA criterion reads captured evidence and nothing else: the handles are released before it runs, so what it did not measure during the steps it cannot go and look at now."
    )]
    fn in_band(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let SoakParameters { target_c, band_c, .. } = *ctx.parameters();
        let mut held = 0;
        for row in ctx.samples::<SoakRow>() {
            // The ramp is not the hold, and the observer said which is which.
            if !row.holding {
                continue;
            }
            if (row.celsius - target_c).abs() > band_c {
                return Ok(false);
            }
            held += 1;
        }

        Ok(held > 0)
    }

    #[criterion(
        id = "{{crate}}.soak.host_quiet",
        label = "Host stayed under its CPU limit",
        description = "Was the machine quiet enough for the measurement to mean anything?\nHost rows are evidence like any other stream, which is what makes this a criterion rather than a note. The limit is a parameter because what counts as quiet is the bench's business: the starter ships it at 100%, which admits anything the host can report, so tighten it to what your own measurement needs rather than to whatever the machine happened to be doing. A host row whose load the runtime could not read is indeterminate rather than a pass — a criterion that cannot read its evidence has decided nothing — and a run that captured no host rows at all has nothing to contradict."
    )]
    fn host_quiet(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let limit = ctx.parameters().host_cpu_limit;
        for row in ctx.samples::<dokime_host::System>() {
            let Some(load) = row.cpu_load_percent else {
                return Err(Error::Unmeasured("cpu_load_percent"));
            };
            if load > limit {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        ctx.resources().heater.lock().await.off().await?;
        dokime::info!(ctx, "chamber released after {:?}", ctx.reason());

        Ok(())
    }
}

// A private helper, not a stage or a marker: it lives on a second inherent
// impl, beside the declared one instead of inside it, the way any other
// method the macro does not read has to.
impl Hold {
    fn summary<'a>(&'a self, parameters: &SoakParameters, celsius: f64) -> Summary<'a> {
        Summary {
            target_c: parameters.target_c,
            band_c: parameters.band_c,
            hold_samples: parameters.hold_samples,
            final_c: celsius,
            unit: self.unit.as_deref(),
            operator_note: self.note.as_deref(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
