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
use dokime::{builders::Plot, prelude::*, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[harmos(package = "execution.v1")]
struct Reading {
    value: f64,
}

#[derive(dokime::Parameters)]
#[harmos(package = "execution.v1")]
struct Parameters {
    value: f64,
    fail_attempts: u32,
    evaluate: bool,
}

#[derive(dokime::Record)]
#[harmos(package = "execution.v1")]
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
    let reading = Stream::<Reading>::new("execution.reading", "Reading");
    let observer = Observer::new(&reading, read).every_ms(1000);
    let document = v1::Document::new("Execution Report")
        .section(v1::Section::new("Measurements").plot(Plot::time_series(v1::FieldReference::new("execution.reading").field("value"))));
    let revision = std::env::var("DOKIME_EXECUTION_REVISION").unwrap_or_else(|_| "1".into());
    let revised = revision != "1";
    let template = Template::new("execution.capture", "Execution Capture")
        .description(revision)
        .parameters(Parameters {
            value: 10.0,
            fail_attempts: 0,
            evaluate: true,
        })
        .initialize(initialize)
        .step(Step::new("Baseline", baseline).identifier("baseline"))
        .step(Step::new("Measure", measure).identifier("measure").retries(1))
        .step(Step::new("Tail", baseline).identifier("tail"))
        .criterion(Criterion::new("Own Measurements", own_measurements).identifier("own_measurements"))
        .materialize(move |mut case, parameters| {
            if parameters.get("fail_attempts").is_some_and(|value| value.kind == Some(v1::value::Kind::UnsignedInteger(u32::MAX as u64))) {
                return Err(Error::Invalid("Materialize: Correct the Failure Count".into()));
            }
            if revised {
                case.steps.pop();
            }
            if parameters.get("evaluate").is_some_and(|value| value.kind == Some(v1::value::Kind::Boolean(false))) {
                case.criteria.clear();
            }
            Ok(case)
        })
        .cleanup(cleanup)
        .document(document)
        .stream(reading)
        .observer(observer);
    let plugin = Plugin::new("execution", "1.0.0")
        .suite(Suite::new("execution", "Execution").template(template))
        .record(Record::<Attempt>::new("execution.attempt", "Attempt"))
        .prompt(Prompt::new("execution.continue", "Continue", "Continue the Measurement?").action("continue", "Continue"))
        .requires_host::<dokime::protocol::context::v1::PromptsCreateRequest>()
        .requires_host::<dokime::protocol::context::v1::evidence::RecordsAppendRequest>()
        .requires_host::<dokime::protocol::context::v1::evidence::RecordsListRequest>()
        .requires_host::<dokime::protocol::context::v1::EvidenceStatisticsRequest>()
        .requires_host::<dokime::protocol::context::v1::evidence::LogsAppendRequest>()
        .requires_host::<dokime::protocol::context::v1::BackendEnvironmentRequest>()
        .requires::<dokime::clock::TimeGetRequest>()
        .requires::<dokime::clock::TimeSleepRequest>();
    let plugin = if std::env::args().any(|argument| argument == "--remote") { plugin.remote() } else { plugin };
    plugin.serve()
}

async fn initialize(ctx: &Context) -> Result<()> {
    // Resolve Host Facts Through the Active Invocation.
    let environment = ctx.host(dokime::protocol::context::v1::BackendEnvironmentRequest {}).await?;
    if environment.backend_pid == 0 || environment.in_step {
        return Err(Error::Invalid("Initialize: Read the Host Environment Outside a Step".into()));
    }
    let _now = ctx.now().await?;

    let parameters = ctx.parameters::<Parameters>()?;
    ctx.set(Bench {
        value: parameters.value,
        fail_attempts: parameters.fail_attempts,
    });
    ctx.log(v1::Severity::Info, parameters.value.to_string()).await
}

async fn baseline(ctx: &Context) -> Result<()> {
    ctx.sleep(Duration::from_secs(1)).await
}

async fn measure(ctx: &Context) -> Result<()> {
    if ctx.attempt() == 1 {
        ctx.prompt(dokime::protocol::context::v1::PromptsCreateRequest {
            prompt: "continue".into(),
            prompt_type: "execution.continue".into(),
            ..Default::default()
        })
        .await?;
    }
    let bench = ctx.get::<Bench>()?;
    let refused = ctx
        .host(dokime::protocol::context::v1::evidence::LogsAppendRequest {
            execution: "missing-execution".into(),
            message: "out-of-scope".into(),
            ..Default::default()
        })
        .await;
    let scope_refused = refused.is_err_and(|error| error.to_string().contains("dokime.context.v1.evidence.Logs/Append"));
    ctx.record(
        &Record::<Attempt>::new("execution.attempt", "Attempt"),
        Attempt {
            value: bench.value,
            attempt: ctx.attempt(),
            scope_refused,
        },
    )
    .await?;
    ctx.sleep(Duration::from_secs(1)).await?;
    if ctx.attempt() <= bench.fail_attempts {
        return Err(Error::failure(dokime::Classification::Transient, "measurement_failed", "Measurement Failed", "Retry the Measurement"));
    }
    Ok(())
}

async fn read(ctx: &Context) -> Result<Reading> {
    Ok(Reading { value: ctx.get::<Bench>()?.value })
}

async fn own_measurements(ctx: &Context) -> Result<bool> {
    let stats = ctx.statistics("execution.reading", "", "measure").await?;
    let repeated = ctx.statistics("execution.reading", "value", "measure").await?;
    let records = ctx.records("execution.attempt", "measure", 1).await?;
    let attempt = records
        .last
        .map(|record| <Attempt as dokime::encoding::Message>::decode(record.payload.as_slice()))
        .transpose()
        .map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(stats == repeated
        && stats.count > 0
        && stats.min == ctx.get::<Bench>()?.value
        && stats.max == stats.min
        && attempt.is_some_and(|record| record.scope_refused))
}

async fn cleanup(ctx: &Context) -> Result<()> {
    ctx.log(v1::Severity::Info, ctx.get::<Bench>()?.value.to_string()).await
}

/* === Tests === */
