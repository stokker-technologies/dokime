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

// Third Party Imports

// Local Imports
use dokime::{prelude::*, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[harmos(package = "thermal.v1")]
struct Temperature {
    celsius: f64,
}

#[derive(dokime::Parameters)]
#[harmos(package = "thermal.v1")]
struct Parameters {
    limit: f64,
}

#[derive(dokime::Record)]
#[harmos(package = "thermal.v1")]
struct Door {
    closed: bool,
}

#[derive(dokime::Row)]
#[harmos(package = "thermal.v1")]
struct Counter {
    value: u64,
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
    let temperature = Stream::<Temperature>::new("thermal.temperature", "Temperature [°C]");
    let observer = Observer::new(&temperature, read_temperature).every_ms(1000);
    let counter = Stream::<Counter>::new("thermal.counter", "Exact Counter");
    let counter_observer = Observer::new(&counter, read_counter).every_ms(1);
    let mut summary = v1::Summary::new();
    for (name, kind, percentile) in [
        ("Count", v1::Aggregate::Count, 0.0),
        ("Sum", v1::Aggregate::Sum, 0.0),
        ("Mean", v1::Aggregate::Mean, 0.0),
        ("Median", v1::Aggregate::Median, 0.0),
        ("Min", v1::Aggregate::Min, 0.0),
        ("Max", v1::Aggregate::Max, 0.0),
        ("First", v1::Aggregate::First, 0.0),
        ("Last", v1::Aggregate::Last, 0.0),
        ("P05", v1::Aggregate::Percentile, 5.0),
        ("P95", v1::Aggregate::Percentile, 95.0),
        ("P99", v1::Aggregate::Percentile, 99.0),
    ] {
        let input = v1::FieldReference::new("thermal.counter").field("value");
        let value = v1::DataExpression {
            kind: Some(v1::data_expression::Kind::Aggregate(Box::new(v1::AggregateExpression {
                input: Some(Box::new(input)),
                kind: kind as i32,
                percentile,
            }))),
            ..Default::default()
        };
        summary = summary.tile(name, value);
    }
    let door = Record::<Door>::new("thermal.door", "Door Check");
    let prompt = Prompt::new("thermal.door", "Door Check", "Is the Door Closed?")
        .archetype(v1::PromptArchetype::Confirmation)
        .action("yes", "Yes")
        .action("no", "No");
    let mut plots = v1::Section::new("Plots");
    for index in 0..100 {
        plots = plots.card(v1::Card {
            metadata: Some(v1::Metadata {
                identifier: format!("plot-card-{index}"),
                ..Default::default()
            }),
            body: Some(v1::card::Body::Plot(v1::Plot {
                metadata: Some(v1::Metadata {
                    identifier: format!("plot-{index}"),
                    ..Default::default()
                }),
                kind: v1::PlotKind::TimeSeries as i32,
                y: vec![v1::FieldReference::new("thermal.counter").field("value")],
                ..Default::default()
            })),
            ..Default::default()
        });
    }
    let template = Template::new("thermal.cycle", "Thermal Cycle")
        .parameters(Parameters { limit: 85.0 })
        .initialize(initialize)
        .step(Step::new("Ramp to Target", ramp).identifier("ramp").retries(2))
        .step(Step::new("Hold", hold).identifier("hold"))
        .criterion(Criterion::new("Temperature Within Limit", within_limit).identifier("within_limit"))
        .cleanup(cleanup)
        .stream(temperature)
        .observer(observer)
        .stream(counter)
        .observer(counter_observer)
        .document(
            v1::Document::new("Thermal Report")
                .id("thermal.report")
                .section(v1::Section::new("Evidence").id("thermal.evidence").summary("Exact Statistics", summary).card(v1::Card {
                    metadata: Some(v1::Metadata {
                        identifier: "thermal.records".into(),
                        label: "Records".into(),
                        ..Default::default()
                    }),
                    body: Some(v1::card::Body::Evidence(v1::EvidenceCard::Records as i32)),
                    ..Default::default()
                }))
                .section(plots),
        );
    let plugin = Plugin::new("thermal", "1.0.0")
        .suite(Suite::new("thermal", "Thermal").template(template))
        .record(door)
        .prompt(prompt)
        .requires_host::<dokime::protocol::plugin::host::v1::PromptsCreateRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::evidence::RecordsAppendRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::evidence::RecordsListRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::EvidenceStatisticsRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::executions::ProgressGetRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::executions::ProgressSetRequest>()
        .requires_host::<dokime::protocol::plugin::host::v1::evidence::LogsAppendRequest>()
        .requires::<dokime::clock::TimeSleepRequest>();
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

async fn initialize(ctx: &Context) -> Result<()> {
    let parameters = ctx.parameters::<Parameters>()?;
    ctx.set(Bench { limit: parameters.limit });
    ctx.log(v1::Severity::Info, "Initialized").await
}

async fn ramp(ctx: &Context) -> Result<()> {
    let _bench = ctx.get::<Bench>()?;
    ctx.progress(1, 2, "Ramp to Target").await
}

async fn hold(ctx: &Context) -> Result<()> {
    let answer = ctx
        .prompt(dokime::protocol::plugin::host::v1::PromptsCreateRequest {
            prompt: "door".into(),
            prompt_type: "thermal.door".into(),
            ..Default::default()
        })
        .await?;
    ctx.record(
        &Record::<Door>::new("thermal.door", "Door Check"),
        Door {
            closed: answer.action == "yes" && answer.accepted,
        },
    )
    .await?;
    ctx.sleep(std::time::Duration::from_millis(2500)).await?;
    ctx.progress(2, 2, "Hold").await
}

async fn read_counter(_: &Context) -> Result<Counter> {
    Ok(Counter { value: 1_152_921_504_606_847_975 })
}

async fn read_temperature(ctx: &Context) -> Result<Temperature> {
    Ok(Temperature { celsius: ctx.get::<Bench>()?.limit - 1.0 })
}

async fn within_limit(ctx: &Context) -> Result<bool> {
    let stats = ctx.statistics("thermal.temperature", "celsius", "hold").await?;
    let records = ctx.records("thermal.door", "hold", 0).await?;
    let closed = records
        .last
        .map(|record| <Door as dokime::encoding::Message>::decode(record.payload.as_slice()))
        .transpose()
        .map_err(|error| Error::Invalid(error.to_string()))?
        .is_some_and(|door| door.closed);
    Ok(stats.count > 0
        && matches!(stats.max.as_ref().and_then(|value| value.kind.as_ref()), Some(v1::value::Kind::Number(value)) if *value <= ctx.get::<Bench>()?.limit)
        && closed)
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(v1::Severity::Info, "Cleaned Up").await
}

/* === Tests === */
