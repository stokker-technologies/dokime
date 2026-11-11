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
use crate::domain::model::chamber::System;
use crate::domain::model::prompts::SoakCheckpoint;
use crate::domain::model::soak::Summary;
use crate::domain::{Error, Result};
use crate::operations::{EncodeSummary, OpenHeater, ReadChamber, ReleaseChamber, SetTarget};
use dokime::host::ToolConnectionAcquired;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[record(id = "{{crate}}.soak.held", label = "Hold Reached", severity = info, description = "One completed hold, with what it measured.")]
#[derive(Clone, Debug)]
pub struct HoldReached {
    #[record(description = "How many samples the chamber was held for.")]
    pub samples: u64,

    #[record(unit = "°C", description = "What it was at when the hold ended.")]
    pub temperature_c: f64,

    #[record(description = "The unit the bench was told about, when it was told about one.")]
    pub unit: Option<String>,
}

#[parameters]
pub struct Parameters {
    #[parameter(
        label = "Target Temperature",
        unit = "°C",
        default = 60.0,
        description = "What the chamber is driven to. Deliberately unbounded above: a target the bench cannot reach is something this procedure *discovers*, and refusing it at admission would hide the failure the run is there to find."
    )]
    pub target_c: f64,

    #[parameter(label = "Allowed Band", unit = "°C", default = 1.5, min = 0.0)]
    pub band_c: f64,

    #[parameter(label = "Samples to Hold", default = 5, min = 1)]
    pub hold_samples: u64,

    #[parameter(label = "Heater Port", default = "/dev/ttyUSB0")]
    pub heater_port: String,

    #[parameter(label = "Seconds to Wait for the Unit", unit = "s", default = 0.5, min = 0.0)]
    pub connect_budget_s: f64,

    #[parameter(label = "Highest Host CPU Load Allowed", unit = "%", default = 100.0, min = 0.0, max = 100.0)]
    pub host_cpu_limit_percent: f64,

    #[parameter(label = "Ask the Operator to Sign Off", default = false)]
    pub attended: bool,
}

pub struct Template {
    unit: Option<String>,
    note: Option<String>,
}

pub struct SoakResources {
    holding: AtomicBool,
}

#[stream(
    id = "{{crate}}.soak.samples",
    window = 4096,
    label = "Soak Samples",
    description = "The chamber, once per interval, for the length of the attempt."
)]
pub struct Row {
    #[row(unit = "°C", format = fixed(1))]
    pub temperature_c: f64,

    #[row(role = state)]
    pub holding: bool,
}

/* === Enums === */

#[dokime::catalog]
pub enum Required {
    ReadChamber(ReadChamber),
    OpenHeater(OpenHeater),
    SetTarget(SetTarget),
    ReleaseChamber(ReleaseChamber),
    EncodeSummary(EncodeSummary),
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
    System(System),
    ToolConnectionAcquired(ToolConnectionAcquired),
}

#[dokime::catalog]
pub enum Streams {
    Row(Row),
}

/* === Traits === */

/* === Implementations === */

#[template(
    id = "{{crate}}.soak.hold",
    label = "Hold at Temperature",
    tags = ["thermal", "soak"],
    requires = Required,
    prompts = Prompts,
    records = Records,
    subscribes = Inputs,
    parameters = Parameters, description = "Drive the chamber to the target and hold it for the requested samples.")]
impl Template {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<(Self, SoakResources), Error> {
        let template = Self { unit: None, note: None };
        let shared = SoakResources { holding: AtomicBool::new(false) };
        Ok((template, shared))
    }

    #[step(
        label = "Wait for the Unit",
        description = "Waits for somebody else to say the unit is on the bench.\n`ctx.listen::<T>()` is the one way a step waits, and it answers a standard stream — so `.timeout(…)`, `.next()` and the rest are the adapters Rust already has, imported once from `dokime::prelude`. [`ToolConnectionAcquired`] is the host's own record type, so this step names no id and reads no detail map: the unit's identity is a field of the fact it waited for.\nAn execution's record inbox starts where the run started, so a unit that came online before this step reached its await still satisfies it.\nA bench with no gateway beside it is told about no unit, so a deadline here is a note rather than a failure. Make it a `?` the day another plugin is responsible for saying so."
    )]
    async fn connected(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Open Heater and Read Identity
        let operation = OpenHeater {
            port: ctx.parameters().heater_port.clone(),
        };
        let identity = ctx.operation(operation).await?;

        // Read Chamber Temperature
        let operation = ReadChamber {};
        let reading = ctx.operation(operation).await?;

        // === Device Under Test === //
        let mut section = TraceabilitySection::new("Device Under Test");

        let serial_number = TraceabilityRow::text("Serial Number", &identity.serial_number);
        section.row(serial_number);

        let firmware_version = TraceabilityRow::text("Firmware Version", &identity.firmware_version);
        section.row(firmware_version);

        ctx.traceability().section(section);

        // === Test Bench === //
        // Keep the edit key stable when the section label changes.
        let mut section = TraceabilitySection::new("Test Bench");
        section.id("bench");

        let sensor = TraceabilityRow::text("Sensor", &reading.sensor);
        section.row(sensor);

        let target_c = TraceabilityRow::number("Target", ctx.parameters().target_c).id("target_c");
        let target_c = target_c.unit("°C");
        section.row(target_c);

        ctx.traceability().section(section);

        // Wait for the Unit
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
        // Set the Target
        let operation = SetTarget { temperature_c: ctx.parameters().target_c };
        ctx.operation(operation).await?;
        dokime::info!(ctx, "heater set to {:.1} °C", ctx.parameters().target_c);

        // Update Target Traceability
        let target_c = TraceabilityRow::number("Target", ctx.parameters().target_c).id("target_c");
        let target_c = target_c.unit("°C");
        ctx.traceability().row("bench", target_c);

        // Wait for the Band
        let parameters = ctx.parameters();
        let target_c = parameters.target_c;
        let band_c = parameters.band_c;
        let mut measurements = ctx.listen::<Row>();
        while let Some(reading) = measurements.next().await {
            if (reading.temperature_c - target_c).abs() <= band_c {
                dokime::info!(ctx, "target reached at {:.2} °C", reading.temperature_c);
                return Ok(());
            }
        }
        Err(Error::Unmeasured("temperature_c"))
    }

    #[step(
        label = "Hold",
        description = "Holds, writing the chamber log as it goes.\nThe step waits on its own observer by the type it publishes — an exact number of rows rather than a length of time, so what the run measured does not depend on how quickly it got going — and appends one line per row through an artifact writer, so the evidence reaches the host while the hold is still running rather than all at once at the end. The row carries the measurement, so the step reads the chamber once, in the observer, rather than twice. What it measured is published as one `HoldReached`, which is a struct rather than a map of details nothing checks until admission."
    )]
    async fn hold(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // === Open the Chamber Log === //
        let mut log = ctx.artifacts().create("heater.log")?;
        let mut held = ctx.listen::<Row>();
        ctx.resources().holding.store(true, Ordering::Release);

        // === Count the Hold === //
        let wanted = ctx.parameters().hold_samples;
        let mut temperature_c = ctx.parameters().target_c;
        let mut sample = 0;
        while sample < wanted {
            let Some(row) = held.next().await else { break };
            sample += 1;
            temperature_c = row.temperature_c;
            writeln!(log, "{sample},{temperature_c:.2}")?;
        }
        if held.missed() > 0 {
            dokime::warn!(ctx, "the hold missed {} rows it measured", held.missed());
        }
        log.finish()?;
        dokime::info!(ctx, "held for {sample} samples, ending at {temperature_c:.2} °C");

        // === Publish Measurements === //
        let summary = Summary {
            target_c: ctx.parameters().target_c,
            band_c: ctx.parameters().band_c,
            hold_samples: ctx.parameters().hold_samples,
            final_c: temperature_c,
            unit: self.unit.clone(),
            operator_note: self.note.clone(),
        };

        // Encode Summary
        let operation = EncodeSummary { summary };
        let summary = ctx.operation(operation).await?.into_owned();
        ctx.artifacts().write("summary.json", summary.json)?;
        let record = HoldReached {
            samples: wanted,
            temperature_c,
            unit: self.unit.clone(),
        };
        ctx.publish(record).await;

        Ok(())
    }

    #[step(label = "Operator Check", description = "Asks the operator to sign off, on a bench that has one.")]
    async fn check(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        // Skip an Unattended Bench
        if !ctx.parameters().attended {
            ctx.info("unattended bench: no operator was asked");
            return Ok(());
        }

        // Ask the Operator
        let prompt = SoakCheckpoint::default();
        let answer = ctx.ask(prompt).await?;
        dokime::info!(ctx, "operator note: {}", answer.note);
        self.note = Some(answer.note);
        if !answer.accepted {
            ctx.warn("the operator did not sign off on this soak");
        }

        Ok(())
    }

    #[criterion(
        id = "{{crate}}.soak.in_band",
        label = "Temperature Stayed in Band",
        description = "Did the chamber stay in band for the whole hold?\nA criterion reads captured evidence and nothing else: the handles are released before it runs, so what it did not measure during the steps it cannot go and look at now."
    )]
    fn in_band(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let rows = ctx.samples::<Row>();
        let parameters = ctx.parameters();

        // Check / Number of Rows
        let number_ok = rows.iter().any(|row| row.holding);

        // Check / Temperature in Band
        let temperature_ok = rows.iter().filter(|row| row.holding).all(|row| {
            (row.temperature_c - parameters.target_c).abs().partial_cmp(&parameters.band_c) != Some(std::cmp::Ordering::Greater)
        });

        Ok(number_ok && temperature_ok)
    }

    #[criterion(
        id = "{{crate}}.soak.host_quiet",
        label = "Host Stayed Under Its CPU Limit",
        description = "Was the machine quiet enough for the measurement to mean anything?\nHost rows are evidence like any other stream, which is what makes this a criterion rather than a note. The limit is a parameter because what counts as quiet is the bench's business: the starter ships it at 100%, which admits anything the host can report, so tighten it to what your own measurement needs rather than to whatever the machine happened to be doing. A host row whose load the runtime could not read is indeterminate rather than a pass — a criterion that cannot read its evidence has decided nothing — and a run that captured no host rows at all has nothing to contradict."
    )]
    fn host_quiet(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        let rows = ctx.samples::<System>();
        let limit = ctx.parameters().host_cpu_limit_percent;

        // Check / Host CPU Load
        let load_ok = match rows.iter().find(|row| row.cpu_load_percent.is_none_or(|load| load > limit)) {
            Some(row) => {
                row.cpu_load_percent.ok_or(Error::Unmeasured("cpu_load_percent"))?;
                false
            }
            None => true,
        };

        Ok(load_ok)
    }

    #[observer(interval_ms = 100, streams = Streams)]
    async fn observe(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        // Read Chamber Temperature
        let operation = ReadChamber {};
        let reading = ctx.operation(operation).await?;

        let row = Row {
            temperature_c: reading.temperature_c,
            holding: ctx.resources().holding.load(Ordering::Acquire),
        };
        ctx.publish(row).await;

        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        // Release Chamber
        let operation = ReleaseChamber {};
        let result = ctx.operation(operation).await;
        if let Err(error) = &result {
            dokime::warn!(ctx, "could not release chamber: {error}");
        }
        dokime::info!(ctx, "chamber cleanup finished after {:?}", ctx.reason());
        result?;

        Ok(())
    }

    fn document() -> Document {
        let mut temperature = Plot::time_series("Temperature");
        temperature.id("{{crate}}.soak.temperature");
        temperature.unit("°C");
        let temperature_c = Row::column_temperature_c();
        temperature.y(temperature_c);
        let elapsed_seconds = format::elapsed_seconds();
        temperature.x_format(elapsed_seconds);
        let fixed = format::fixed(1);
        temperature.y_format(fixed);
        let target_c = Parameters::target_c();
        let band_c = Parameters::band_c();
        temperature.reference_tolerance("Target ± Band", target_c, band_c);
        let marker = HoldReached::marker();
        temperature.marker_records([marker]);

        let mut signals = Section::new("Soak");
        signals.id("{{crate}}.soak.signals");
        signals.plot(temperature);

        let mut document = Document::new("Hold at Temperature");
        document.section(signals);

        document
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
