/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Thermal Suite Shared by Every SDK System Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::RetryPolicy;
use dokime::model::Section;
use dokime::operations;
use dokime::prelude::*;
use dokime::templates::Context;
use dokime::{Card, Document, EvidenceCard, FieldReference, Metadata, Plot, PlotKind, Severity, card, value};
use dokime::{SettingDeclaration, SettingValue, setting_value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ThermalPlugin;
struct ThermalSuite;
struct CycleTestTemplate;
struct LoadTestTemplate;

#[derive(dokime::Message)]
#[dokime(package = "thermal")]
struct Echo {
    value: Vec<u8>,
}

#[derive(dokime::Parameters)]
#[dokime(package = "thermal")]
struct LoadParameters {
    burst: bool,
}

#[derive(dokime::Row)]
#[dokime(package = "thermal")]
struct Temperature {
    celsius: f64,
}

#[derive(dokime::Parameters)]
#[dokime(package = "thermal")]
struct Parameters {
    limit: f64,
}

#[derive(dokime::Record)]
#[dokime(package = "thermal")]
struct Door {
    closed: bool,
}

#[derive(dokime::Row)]
#[dokime(package = "thermal")]
struct Counter {
    value: u64,
}

// One Window of the Counter, One Value per Sample.
#[derive(dokime::Message)]
#[dokime(package = "thermal")]
struct CounterColumns {
    value: Vec<u64>,
}

struct Bench {
    limit: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let temperature = Stream::<Temperature>::new("temperature", "Temperature [°C]");
    let observer = Observer::new(&temperature, read_temperature).every(Duration::from_secs(1));
    let counter = Stream::<Counter>::new("counter", "Exact Counter");
    // A 1 kHz Stream Answers Each Host Window at Once.
    let counter_observer = Observer::window(&counter, read_counter).every(Duration::from_millis(1));
    let door = Record::<Door>::new("door", "Door Check");
    let plot = Plot {
        metadata: Some(Metadata {
            identifier: "thermal.temperature.plot".into(),
            label: "Temperature".into(),
            ..Default::default()
        }),
        kind: PlotKind::TimeSeries as i32,
        y: vec![FieldReference::new("thermal.cycle.temperature").field("celsius")],
        unit: "°C".into(),
        ..Default::default()
    };
    let template = Template::of::<CycleTestTemplate>()
        .label("Thermal Cycle")
        .record(door)
        .parameters(Parameters { limit: 85.0 })
        .initialize(initialize)
        .step(
            Step::new("Ramp to Target", ramp)
                .identifier("ramp")
                .policy(OperationPolicy {
                    retries: Some(RetryPolicy {
                        total_attempts: 3,
                        initial: Some(prost_types::Duration { seconds: 0, nanos: 100_000_000 }),
                        maximum: None,
                        deterministic_jitter: false,
                    }),
                    ..Default::default()
                })
                .replay_safe(),
        )
        .step(Step::new("Hold", hold).identifier("hold"))
        .criterion(Criterion::new("Temperature Within Limit", within_limit).identifier("within_limit"))
        .cleanup(cleanup)
        .stream(temperature)
        .observer(observer)
        .stream(counter)
        .observer(counter_observer)
        .document(Document::new("Thermal Report").id("thermal.report").section(
            Section::new("Evidence").id("thermal.evidence").plot(plot).card(Card {
                metadata: Some(Metadata {
                    identifier: "thermal.records".into(),
                    label: "Records".into(),
                    ..Default::default()
                }),
                body: Some(card::Body::Evidence(EvidenceCard::Records as i32)),
                ..Default::default()
            }),
        ));
    let mut load = Template::of::<LoadTestTemplate>()
        .label("Thermal Load")
        .parameters(LoadParameters { burst: false })
        .initialize(async |ctx: &Context| ctx.progress(0, 1, "Hold").await)
        .step(Step::new("Hold", load_hold).identifier("hold"))
        .materialize(|mut case, parameters| {
            let burst = matches!(parameters.get("burst").and_then(|value| value.kind.as_ref()), Some(value::Kind::Boolean(true)));
            case.streams.retain(|stream| stream.ends_with(".burst") == burst);
            Ok(case)
        });
    for index in 0..128 {
        let stream = Stream::<Counter>::new(format!("channel_{index:03}"), format!("Channel {index:03}"));
        let observer = Observer::window(&stream, read_counter).every(Duration::from_millis(1));
        load = load.stream(stream).observer(observer);
    }
    let burst = Stream::<Counter>::new("burst", "Single-Stream Ceiling");
    let observer = Observer::window(&burst, read_counter).every(Duration::from_millis(1)).interval_setting("thermal.load.interval");
    load = load.stream(burst).observer(observer);
    let plugin = Plugin::of::<ThermalPlugin>("1.0.0")
        .operation(Operation::new("thermal.echo", Visibility::Public, echo))
        .setting(SettingDeclaration {
            minimum: None,
            maximum: None,
            name: "thermal.load.interval".into(),
            title: "Load Interval".into(),
            description: String::new(),
            default: Some(SettingValue {
                value: Some(setting_value::Value::Duration(prost_types::Duration { seconds: 0, nanos: 1_000_000 })),
            }),
        })
        .suite(Suite::of::<ThermalSuite>().template(template).template(load));
    plugin.run()
}

async fn echo(_: &operations::Context, request: Echo) -> Result<Echo> {
    Ok(request)
}

async fn load_hold(ctx: &Context) -> Result<()> {
    ctx.cancelled().await;
    Ok(())
}

async fn initialize(ctx: &Context) -> Result<()> {
    let parameters = ctx.parameters::<Parameters>()?;
    ctx.set(Bench { limit: parameters.limit });
    ctx.log(Severity::Info, "Initialized").await
}

async fn ramp(ctx: &Context) -> Result<()> {
    let _bench = ctx.get::<Bench>()?;
    ctx.progress(1, 2, "Ramp to Target").await
}

async fn hold(ctx: &Context) -> Result<()> {
    let answer = ctx.prompt("Is the Door Closed?", &["Yes", "No"]).await?;
    ctx.record(&Record::<Door>::new("door", "Door Check"), Door { closed: answer == "Yes" }).await?;
    ctx.sleep(Duration::from_millis(2500)).await?;
    ctx.progress(2, 2, "Hold").await
}

async fn read_counter(_: &Context, window: &Window) -> Result<CounterColumns> {
    let samples = usize::try_from(window.count).map_err(|_| Error::Invalid("Read Counter: Shorten the Window".into()))?;
    Ok(CounterColumns {
        value: vec![1_152_921_504_606_847_975; samples],
    })
}

async fn read_temperature(ctx: &Context) -> Result<Temperature> {
    Ok(Temperature { celsius: ctx.get::<Bench>()?.limit - 1.0 })
}

async fn within_limit(ctx: &Context) -> Result<bool> {
    let stats =
        ctx.statistics("temperature", "celsius", "hold").await?.ok_or_else(|| Error::Invalid("Read Statistics: Missing Payload".into()))?;
    let closed = ctx.records(&Record::<Door>::new("door", "Door Check")).step(hold).last_or_none().await?.is_some_and(|door| door.closed);
    Ok(stats.count > 0
        && matches!(stats.max.as_ref().and_then(|value| value.kind.as_ref()), Some(value::Kind::Number(value)) if *value <= ctx.get::<Bench>()?.limit)
        && closed)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, "Cleaned Up").await
}

/* === Tests === */
