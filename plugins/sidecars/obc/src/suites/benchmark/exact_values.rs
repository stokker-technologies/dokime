/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Unsigned Meter Register and Statistics

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::device::{Bench, Supply};
use crate::records::charger;
use crate::{settings, suites};
use dokime::plugin::{Criterion, Observer, Prompt, Record, Step, Stream, Template, Window};
use dokime::templates::Context;
use dokime::{Aggregate, Document, Error, FieldReference, Parameter, PromptArchetype, Result, Summary, Value, model, value};

/* === Constants === */

pub const TEMPLATE: &str = "obc.benchmark.exact_values";
pub const METER: &str = "obc.benchmark.exact_values.meter";
pub const CONFIRMATION: &str = "obc.benchmark.exact_values.confirmation_record";
pub const CONFIRM: &str = "obc.benchmark.exact_values.confirm";
pub const REGISTER: u64 = 1_152_921_504_606_847_975;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Exact Integer the Revenue Meter Reports Every Sample.
#[derive(dokime::Row)]
#[dokime(package = "obc")]
pub struct MeterRegister {
    pub energy_register: u64,
}

/// One Window of Meter Register Readings.
#[derive(dokime::Message)]
#[dokime(package = "obc")]
pub struct MeterRegisterColumns {
    pub energy_register: Vec<u64>,
}

#[derive(dokime::Record)]
#[dokime(package = "obc")]
pub struct Confirmation {
    pub accepted: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn contracts() -> Record<Confirmation> {
    Record::new("confirmation_record", "Meter Readings Confirmed")
}
pub fn prompt() -> Prompt {
    Prompt::new("confirm", "Confirm Meter Readings", "Does the Meter Register Look Plausible?")
        .archetype(PromptArchetype::Confirmation)
        .action("yes", "Yes")
        .action("no", "No")
}
pub fn template() -> Template {
    let meter = Stream::<MeterRegister>::new("meter", "Revenue Meter Register");
    let observer = Observer::window(&meter, read_register).interval_setting(settings::SAMPLE_INTERVAL);
    charger::catalog(
        Template::of::<()>()
            .identifier("exact_values")
            .label("Exact Values")
            .record(contracts())
            .prompt(prompt())
            .parameter(Parameter::number("duration_s", "Capture Duration").unit("s").default(0.25).range(0.001..=100.0))
            .initialize(initialize)
            .step(Step::new("Capture Meter Register", capture).identifier("capture"))
            .step(Step::new("Confirm Meter Register", confirmation).identifier("confirmation"))
            .criterion(Criterion::new("Exact Meter Register", exact).identifier("exact_register"))
            .cleanup(cleanup)
            .stream(meter)
            .observer(observer)
            .document(document()),
    )
}
pub fn document() -> Document {
    let register = || FieldReference::new(METER).field("energy_register");
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
    let metering = model::Section::new("Metering")
        .id("obc.benchmark.exact_values.metering")
        .summary("Exact Statistics", summary)
        .card(dokime::Card::records("obc.benchmark.exact_values.records"));
    Document::new("Exact Values").id(format!("{TEMPLATE}.report")).section(metering)
}
async fn initialize(ctx: &Context) -> Result<()> {
    suites::open(ctx, Supply::default(), None).await
}
async fn capture(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    let samples = bench.samples_for(ctx.parameter("duration_s")?)?;
    suites::wait(ctx, &bench, samples).await
}
async fn confirmation(ctx: &Context) -> Result<()> {
    let answer = ctx.prompt(prompt(), &[]).await?;
    ctx.record(&contracts(), Confirmation { accepted: answer == "Yes" }).await?;
    Ok(())
}
async fn exact(ctx: &Context) -> Result<bool> {
    let Some(statistics) = ctx.statistics("meter", "energy_register", "").await? else {
        return Ok(false);
    };
    let exact = |value: Option<Value>| matches!(value.and_then(|value| value.kind), Some(value::Kind::UnsignedInteger(REGISTER)));
    let confirmed = charger::last(ctx, &contracts()).await?.is_some_and(|record| record.accepted);
    Ok(statistics.count > 0 && exact(statistics.min) && exact(statistics.max) && confirmed)
}
async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.get::<Bench>()?.stop();
    Ok(())
}
async fn read_register(ctx: &Context, window: &Window) -> Result<MeterRegisterColumns> {
    if suites::captures(ctx, METER) {
        ctx.get::<Bench>()?.advance_by(window.count);
    }
    let samples = usize::try_from(window.count).map_err(|_| Error::Invalid("Read Register: Shorten the Window".into()))?;
    Ok(MeterRegisterColumns { energy_register: vec![REGISTER; samples] })
}

/* === Private Functions === */

/* === Tests === */
