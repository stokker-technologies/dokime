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
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::RetryPolicy;
use dokime::encoding;
use dokime::model::Section;
use dokime::plugin::HostEvidenceStatisticsOptions;
use dokime::{Classification, Plot, PlotKind, prelude::*};
use dokime::{Document, FieldReference, Severity, value::Kind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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
}

#[derive(dokime::Record)]
#[dokime(package = "execution")]
struct Attempt {
    value: f64,
    attempt: u32,
    scope_refused: bool,
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
    let observer = Observer::new(&reading, read).every(std::time::Duration::from_secs(1));
    let plot = Plot {
        kind: PlotKind::TimeSeries as i32,
        y: vec![FieldReference::new("execution.reading").field("value")],
        ..Default::default()
    };
    let document = Document::new("Execution Report").section(Section::new("Measurements").plot(plot));
    let revision = std::env::var("DOKIME_EXECUTION_REVISION").unwrap_or_else(|_| "1".into());
    let revised = revision != "1";
    let template = Template::new("capture", "Execution Capture")
        .record(Record::<Attempt>::new("attempt", "Attempt"))
        .description(revision)
        .parameters(Parameters {
            value: 10.0,
            fail_attempts: 0,
            evaluate: true,
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
    let plugin = Plugin::new("execution", "1.0.0").suite(Suite::new("execution", "Execution").template(template));
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

async fn initialize(ctx: &Context) -> Result<()> {
    // Resolve Host Facts Through the Active Invocation.
    let environment = ctx.host().backend.environment().await?;
    if environment.backend_pid == 0 || environment.in_step {
        return Err(Error::Invalid("Initialize: Read the Host Environment Outside a Step".into()));
    }
    let _now = ctx.now().await?;

    let parameters = ctx.parameters::<Parameters>()?;
    ctx.set(Bench {
        value: parameters.value,
        fail_attempts: parameters.fail_attempts,
    });
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
    let refused =
        ctx.host().evidence.logs.append("missing-execution".into(), Severity::Info, "out-of-scope".into(), ctx.now().await?).await;
    let scope_refused = refused.is_err_and(|error| error.to_string().contains("Append"));
    ctx.record(
        &Record::<Attempt>::new("attempt", "Attempt"),
        Attempt {
            value: bench.value,
            attempt: ctx.attempt(),
            scope_refused,
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
    let stats = ctx.statistics("reading", "", "measure").await?;
    let repeated = ctx
        .host()
        .evidence
        .statistics_with(
            ctx.resolve("reading")?,
            "value".into(),
            HostEvidenceStatisticsOptions {
                step: Some(ctx.resolve("measure")?),
                at: stats.resolved_at.clone(),
                ..Default::default()
            },
        )
        .await?;
    let mut latest = ctx.statistics("reading", "value", "measure").await?;
    let advanced = latest.resolved_at.as_ref().zip(stats.resolved_at.as_ref()).is_some_and(|(latest, first)| {
        latest.positions.iter().any(|current| {
            first.positions.iter().any(|previous| current.history == previous.history && current.sequence > previous.sequence)
        })
    });
    latest.resolved_at = stats.resolved_at.clone();
    let records = ctx.records(&Record::<Attempt>::new("attempt", "Attempt")).step(measure).list(1).await?;
    let attempt = records
        .last
        .map(|record| <Attempt as encoding::Message>::decode(record.payload.as_slice()))
        .transpose()
        .map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(stats == repeated
        && stats == latest
        && advanced
        && stats.statistics.as_ref().is_some_and(|statistics| statistics.count > 0)
        && stats.statistics.as_ref().and_then(|statistics| statistics.min.as_ref()).and_then(|value| value.kind.as_ref())
            == Some(&Kind::Number(ctx.get::<Bench>()?.value))
        && stats.statistics.as_ref().map(|statistics| &statistics.max) == stats.statistics.as_ref().map(|statistics| &statistics.min)
        && attempt.is_some_and(|record| record.scope_refused))
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(Severity::Info, ctx.get::<Bench>()?.value.to_string()).await
}

/* === Tests === */
