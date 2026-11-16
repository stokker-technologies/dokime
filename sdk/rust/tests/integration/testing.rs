/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// An Author Tests a Template Without a Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use prost_types::Timestamp;

// Local Imports
use dokime::prelude::*;
use dokime::testing::{Evaluated, Harness, Logged, Prompted};
use dokime::{ExecutionProgressValue, Severity, Termination, Value, value::Kind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Parameters)]
#[harmos(package = "oven")]
struct Setpoint {
    celsius: f64,
    hold_s: u32,
}

#[derive(dokime::Row)]
#[harmos(package = "oven")]
struct Temperature {
    celsius: f64,
}

#[derive(dokime::Record)]
#[harmos(package = "oven")]
struct Soaked {
    celsius: f64,
    confirmed: bool,
}

struct Oven {
    celsius: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn plugin() -> Plugin<((), Soaked)> {
    let temperature = Stream::<Temperature>::new("oven.temperature", "Temperature");
    let observer = Observer::new(&temperature, async |ctx: &Context| Ok(Temperature { celsius: ctx.get::<Oven>()?.celsius })).every(std::time::Duration::from_millis(500));
    let soak = Template::new("oven.soak", "Soak")
        .parameters(Setpoint { celsius: 85.0, hold_s: 2 })
        .initialize(async |ctx: &Context| {
            ctx.set(Oven {
                celsius: ctx.parameters::<Setpoint>()?.celsius,
            });
            Ok(())
        })
        .step(Step::new("Soak", soak).identifier("soak"))
        .criterion(Criterion::new("Held Setpoint", held).identifier("held"))
        .cleanup(async |ctx: &Context| ctx.log(Severity::Info, format!("{:?}", ctx.termination())).await)
        .stream(temperature)
        .observer(observer);
    Plugin::new("oven", "1.0.0")
        .suite(Suite::new("oven", "Oven").template(soak))
        .prompt(Prompt::new("oven.door", "Door", "Is the Door Closed?").action("cancel", "Cancel").action("closed", "Closed"))
        .requires_host(HostOperation::PromptsCreate)
        .requires_host(HostOperation::EvidenceRecordsAppend)
        .requires_host(HostOperation::EvidenceStatistics)
        .requires_host(HostOperation::EvidenceLogsAppend)
        .requires_host(HostOperation::ExecutionsProgressGet)
        .requires_host(HostOperation::ExecutionsProgressSet)
        .requires_host(HostOperation::TimeSleep)
        .record(Record::<Soaked>::new("oven.soaked", "Soaked"))
}

async fn soak(ctx: &Context) -> Result<()> {
    let door = ctx.host().prompts.create("door".into(), "oven.door".into(), Default::default()).await?;
    let hold_s = ctx.parameters::<Setpoint>()?.hold_s;
    if hold_s == 0 {
        return Err(Error::Invalid("Soak: Hold for at Least One Second".into()));
    }
    ctx.sleep(Duration::from_secs(hold_s.into())).await?;
    ctx.progress(1, 1, "Soaked").await?;
    let soaked = Soaked {
        celsius: ctx.get::<Oven>()?.celsius,
        confirmed: door.action == "closed",
    };
    ctx.record(&Record::<Soaked>::new("oven.soaked", "Soaked"), soaked).await?;
    Ok(())
}

async fn held(ctx: &Context) -> Result<bool> {
    let statistics = ctx.statistics("oven.temperature", "celsius", "soak").await?.statistics.unwrap_or_default();
    Ok(statistics.count == 4
        && statistics.mean
            == Some(Value {
                kind: Some(Kind::Number(ctx.parameters::<Setpoint>()?.celsius)),
            }))
}

/* === Tests === */

#[tokio::test]
async fn author_soaks_an_oven_without_a_host() -> Result<()> {
    // Run the Template In-Process; the Declared Prompt Takes Its First Non-Cancel Action.
    let report = Harness::new(plugin(), "oven.soak").run().await?;
    assert!(report.passed(), "{report:?}");
    assert_eq!(report.criteria(), [Evaluated { criterion: "held".into(), passed: true }]);
    assert_eq!(
        report.prompts(),
        [Prompted {
            step: "soak".into(),
            prompt_type: "oven.door".into(),
            action: "closed".into(),
        }]
    );

    // The Sleep Grants One Window of Every Sample Due by Its End, Inside the Step.
    let samples = report.samples::<Temperature>("oven.temperature")?;
    let times = samples.iter().map(|sample| (sample.at.seconds, sample.at.nanos)).collect::<Vec<_>>();
    assert_eq!(times, [(0, 500_000_000), (1, 0), (1, 500_000_000), (2, 0)]);
    assert!(samples.iter().all(|sample| sample.step == "soak" && sample.value.celsius == 85.0));
    assert_eq!(report.now(), Timestamp { seconds: 2, nanos: 0 });

    // Records, Progress and Cleanup Logs Are Inspected as Typed Values.
    let soaked = report.records::<Soaked>("oven.soaked")?;
    assert_eq!(soaked.len(), 1);
    assert_eq!((soaked[0].celsius, soaked[0].confirmed), (85.0, true));
    assert_eq!(
        report.progress(),
        Some(&ExecutionProgressValue {
            completed: 1,
            total: 1,
            label: "Soaked".into(),
        })
    );
    assert_eq!(
        report.logs(),
        [Logged {
            step: String::new(),
            severity: Severity::Info,
            message: format!("{:?}", Some(Termination::Completed)),
        }]
    );

    // Answer the Prompt Programmatically and Change the Parameters for a Failing Run.
    let report =
        Harness::new(plugin(), "oven.soak").parameters(Setpoint { celsius: 40.0, hold_s: 0 }).answer("oven.door", "cancel").run().await?;
    assert_eq!(report.termination(), Termination::Failed);
    assert!(!report.passed());
    assert!(report.criteria().is_empty());
    assert_eq!(report.prompts()[0].action, "cancel");
    assert!(report.samples::<Temperature>("oven.temperature")?.is_empty());
    assert_eq!(report.logs()[0].message, format!("{:?}", Some(Termination::Failed)));
    Ok(())
}
