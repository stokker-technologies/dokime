/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Features Shared by Host Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env;
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::RetryPolicy;
use dokime::model::Section;
use dokime::templates::Context;
use dokime::templates::StatisticsReadOptions;
use dokime::{Classification, Plot, PlotKind, prelude::*};
use dokime::{Document, FieldReference, Severity, value::Kind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ExecutionPlugin;
struct ExecutionSuite;
struct CaptureTestTemplate;

#[derive(dokime::Row)]
#[dokime(package = "execution")]
struct Reading {
    value: f64,
}

#[derive(dokime::Parameters)]
#[dokime(package = "execution")]
struct Parameters {
    value: f64,
    fail_attempts: u32,
    evaluate: bool,
    initialize_failure: Option<bool>,
    initialize_question: Option<bool>,
}

#[derive(dokime::Record)]
#[dokime(package = "execution")]
struct Attempt {
    value: f64,
    attempt: u32,
}

struct Bench {
    value: f64,
    fail_attempts: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    let reading = Stream::<Reading>::new("reading", "Reading");
    let observer = Observer::new(&reading, read).every(Duration::from_secs(1));
    let plot = Plot {
        kind: PlotKind::TimeSeries as i32,
        y: vec![FieldReference::new("execution.execution.capture.reading").field("value")],
        ..Default::default()
    };
    let document = Document::new("Execution Report").section(Section::new("Measurements").plot(plot));
    let revision = env::var("DOKIME_EXECUTION_REVISION").unwrap_or_else(|_| "1".into());
    let revised = revision != "1";
    let template = Template::of::<CaptureTestTemplate>()
        .label("Execution Capture")
        .record(Record::<Attempt>::new("attempt", "Attempt"))
        .description(revision)
        .parameters(Parameters {
            value: 10.0,
            fail_attempts: 0,
            evaluate: true,
            initialize_failure: None,
            initialize_question: None,
        })
        .initialize(initialize)
        .step(Step::new("Baseline", baseline).identifier("baseline"))
        .step(
            Step::new("Measure", measure)
                .identifier("measure")
                .policy(OperationPolicy {
                    retries: Some(RetryPolicy {
                        total_attempts: 2,
                        initial: Some(prost_types::Duration { seconds: 0, nanos: 100_000_000 }),
                        maximum: None,
                        deterministic_jitter: false,
                    }),
                    ..Default::default()
                })
                .replay_safe(),
        )
        .step(Step::new("Tail", baseline).identifier("tail"))
        .criterion(Criterion::new("Own Measurements", own_measurements).identifier("own_measurements"))
        .materialize(move |mut case, parameters| {
            if parameters.get("fail_attempts").is_some_and(|value| value.kind == Some(Kind::UnsignedInteger(u32::MAX as u64))) {
                return Err(Error::Invalid("Materialize: Correct the Failure Count".into()));
            }
            if revised {
                case.steps.pop();
            }
            if parameters.get("evaluate").is_some_and(|value| value.kind == Some(Kind::Boolean(false))) {
                case.criteria.clear();
            }
            Ok(case)
        })
        .cleanup(cleanup)
        .document(document)
        .stream(reading)
        .observer(observer);
    let plugin =
        Plugin::of::<ExecutionPlugin>("1.0.0").suite(Suite::of::<ExecutionSuite>().identifier("execution.execution").template(template));
    plugin.run()
}

async fn initialize(ctx: &Context) -> Result<()> {
    let parameters = ctx.parameters::<Parameters>()?;
    ctx.set(Bench {
        value: parameters.value,
        fail_attempts: parameters.fail_attempts,
    });
    if parameters.initialize_failure == Some(true) {
        return Err(Error::Invalid("Initialize Execution: Resolve the Fixture's Requested Initialization Failure".into()));
    }
    if parameters.initialize_question == Some(true) {
        ctx.prompt("Continue Initialization?", &["Continue"]).await?;
    }
    // Resolve Host Facts Through the Active Invocation.
    let environment = ctx.environment().await?;
    if environment.backend_pid == 0 || environment.in_step {
        return Err(Error::Invalid("Initialize: Read the Host Environment Outside a Step".into()));
    }
    let _now = ctx.now().await?;

    ctx.log(Severity::Info, parameters.value.to_string()).await
}

async fn baseline(ctx: &Context) -> Result<()> {
    ctx.sleep(Duration::from_secs(1)).await
}

async fn measure(ctx: &Context) -> Result<()> {
    if ctx.attempt() == 1 {
        ctx.prompt("Continue the Measurement?", &["Continue"]).await?;
    }
    let bench = ctx.get::<Bench>()?;
    ctx.record(
        &Record::<Attempt>::new("attempt", "Attempt"),
        Attempt {
            value: bench.value,
            attempt: ctx.attempt(),
        },
    )
    .await?;
    ctx.sleep(Duration::from_secs(1)).await?;
    if ctx.attempt() <= bench.fail_attempts {
        return Err(Error::failure(Classification::Transient, "measurement_failed", "Measurement Failed", "Retry the Measurement"));
    }
    Ok(())
}

async fn read(ctx: &Context) -> Result<Reading> {
    Ok(Reading { value: ctx.get::<Bench>()?.value })
}

async fn own_measurements(ctx: &Context) -> Result<bool> {
    let stats = ctx.statistics_with("reading", "value", "measure", StatisticsReadOptions::default()).await?;
    let repeated = ctx.statistics_with("reading", "value", "measure", StatisticsReadOptions { at: stats.at.clone() }).await?;
    let mut latest = ctx.statistics_with("reading", "value", "measure", StatisticsReadOptions::default()).await?;
    let advanced = latest.at.as_ref().zip(stats.at.as_ref()).is_some_and(|(latest, first)| {
        latest.positions.iter().any(|current| {
            first.positions.iter().any(|previous| current.history == previous.history && current.sequence > previous.sequence)
        })
    });
    latest.at = stats.at.clone();
    let attempt = ctx.records(&Record::<Attempt>::new("attempt", "Attempt")).step(measure).last_or_none().await?;
    Ok(stats == repeated
        && stats == latest
        && advanced
        && stats.value.as_ref().is_some_and(|statistics| statistics.count > 0)
        && stats.value.as_ref().and_then(|statistics| statistics.min.as_ref()).and_then(|value| value.kind.as_ref())
            == Some(&Kind::Number(ctx.get::<Bench>()?.value))
        && stats.value.as_ref().map(|statistics| &statistics.max) == stats.value.as_ref().map(|statistics| &statistics.min)
        && attempt.is_some_and(|record| record.value == ctx.get::<Bench>().map(|bench| bench.value).unwrap_or_default()))
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, ctx.get::<Bench>()?.value.to_string()).await
}

/* === Tests === */
