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
use dokime::templates::Context;
use dokime::testing::{Evaluated, Harness, Logged, Prompted};
use dokime::{ExecutionProgressValue, Severity, Termination, Value, value::Kind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Parameters)]
#[dokime(package = "oven")]
struct Setpoint {
    celsius: f64,
    hold_s: u32,
}

#[derive(dokime::Row)]
#[dokime(package = "oven")]
struct Temperature {
    celsius: f64,
}

#[derive(dokime::Record)]
#[dokime(package = "oven")]
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

fn plugin() -> Plugin {
    let temperature = Stream::<Temperature>::new("temperature", "Temperature");
    let observer = Observer::new(&temperature, async |ctx: &Context| Ok(Temperature { celsius: ctx.get::<Oven>()?.celsius }))
        .every(std::time::Duration::from_millis(500));
    let soak = Template::of::<()>()
        .identifier("soak")
        .label("Soak")
        .record(Record::<Soaked>::new("soaked", "Soaked"))
        .prompt(Prompt::new("door", "Door", "Is the Door Closed?").action("cancel", "Cancel").action("closed", "Closed"))
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
    Plugin::of::<()>("1.0.0").identifier("oven").suite(Suite::of::<()>().identifier("oven.oven").label("Oven").template(soak))
}

async fn soak(ctx: &Context) -> Result<()> {
    let door = ctx.prompt_answer(Prompt::new("door", "Door", "Close the Door"), Default::default()).await?;
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
    ctx.record(&Record::<Soaked>::new("soaked", "Soaked"), soaked).await?;
    Ok(())
}

async fn held(ctx: &Context) -> Result<bool> {
    let statistics = ctx.statistics("temperature", "celsius", "soak").await?.unwrap_or_default();
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
    let report = Harness::new(plugin(), "oven.oven.soak").run().await?;
    assert!(report.passed(), "{report:?}");
    assert_eq!(
        report.criteria(),
        [Evaluated {
            criterion: "oven.oven.soak.held".into(),
            passed: true
        }]
    );
    assert_eq!(
        report.prompts(),
        [Prompted {
            step: "oven.oven.soak.soak".into(),
            prompt_type: "oven.oven.soak.door".into(),
            action: "closed".into(),
        }]
    );

    // The Sleep Grants One Window of Every Sample Due by Its End, Inside the Step.
    let samples = report.samples::<Temperature>("oven.oven.soak.temperature")?;
    let times = samples.iter().map(|sample| (sample.at.seconds, sample.at.nanos)).collect::<Vec<_>>();
    assert_eq!(times, [(0, 500_000_000), (1, 0), (1, 500_000_000), (2, 0)]);
    assert!(samples.iter().all(|sample| sample.step == "oven.oven.soak.soak" && sample.value.celsius == 85.0));
    assert_eq!(report.now(), Timestamp { seconds: 2, nanos: 0 });

    // Records, Progress and Cleanup Logs Are Inspected as Typed Values.
    let soaked = report.records::<Soaked>("oven.oven.soak.soaked")?;
    assert_eq!(soaked.len(), 1);
    assert_eq!((soaked[0].celsius, soaked[0].confirmed), (85.0, true));
    assert!(matches!(report.progress(), Some(ExecutionProgressValue { completed: 1, total: 1, .. })));
    assert_eq!(
        report.logs(),
        [Logged {
            step: String::new(),
            severity: Severity::Info,
            message: format!("{:?}", Some(Termination::Completed)),
        }]
    );

    // Answer the Prompt Programmatically and Change the Parameters for a Failing Run.
    let report = Harness::new(plugin(), "oven.oven.soak")
        .parameters(Setpoint { celsius: 40.0, hold_s: 0 })
        .answer("oven.oven.soak.door", "cancel")
        .run()
        .await?;
    assert_eq!(report.termination(), Termination::Failed);
    assert!(!report.passed());
    assert!(report.criteria().is_empty());
    assert_eq!(report.prompts()[0].action, "cancel");
    assert!(report.samples::<Temperature>("oven.oven.soak.temperature")?.is_empty());
    assert_eq!(report.logs()[0].message, format!("{:?}", Some(Termination::Failed)));
    Ok(())
}
