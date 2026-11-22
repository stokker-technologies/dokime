/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stop Ends Execution Waits

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error, num::NonZeroUsize, sync::Arc, time::Duration};

// Third Party Imports
use async_trait::async_trait;
use harmos_runtime::clock::{Clock, Controller};
use prost::Message;
use tokio::sync::Notify;

// Local Imports
use crate::{ControlledClock, HostSettings, PlacementSettings, SchedulerClock, clock::SchedulerSource, services};
use dokime::templates::{Context, Criterion, Record, Step, Stream, Template};
use dokime::{Plugin, Suite, Termination};
use dokime_operations::host::ClientSettings;
use dokime_protocol::host::{CaseStatus, CasesListRequest, evidence::RecordsAppendRequest};
use dokime_storage::ExecutionState;
use dokime_test_support::credentials;

/* === Constants === */

const SLEEP: Duration = Duration::from_secs(3 * 60 * 60);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
struct ManualClock {
    source: ControlledClock,
    waiting: Arc<Notify>,
}

#[derive(dokime::Row)]
#[dokime(package = "wait_fixture")]
struct Reading {
    value: u64,
}

struct Probe {
    sampled: Notify,
    looping: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait]
impl SchedulerClock for ManualClock {
    fn now_ns(&self) -> i64 {
        self.source.now_ns()
    }
    async fn wait_until(&self, at_ns: i64) -> crate::Result<()> {
        self.source.wait_until(at_ns).await
    }
    async fn reached(&self, at_ns: i64) {
        if at_ns == SLEEP.as_nanos() as i64 {
            self.waiting.notify_one();
        }
        self.source.reached(at_ns).await;
    }
    fn execution_clock(&self) -> (Clock, Option<Controller>) {
        (Clock::with_clock(SchedulerSource::new(Arc::new(self.clone()))), None)
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn wait(context: &Context) -> dokime::Result<()> {
    let probe = context.plugin::<Probe>()?;
    probe.sampled.notified().await;
    context.record(&Record::new("before", "Before"), 1_i64).await?;
    assert!(!context.stop_requested());
    if probe.looping {
        while !context.stop_requested() {
            context.sleep(SLEEP).await?;
        }
    } else {
        context.sleep(SLEEP).await?;
    }
    if context.stop_requested() {
        context.sleep(SLEEP).await?;
    }
    context
        .record(
            &Record::new("after", "After"),
            Reading {
                value: context.now().await?.seconds as u64,
            },
        )
        .await?;
    Ok(())
}

async fn evaluate(context: &Context) -> dokime::Result<bool> {
    context.record(&Record::new("evaluated", "Evaluated"), true).await?;
    Ok(true)
}

/* === Tests === */

#[test]
fn stop_ends_waits() -> Result<(), Box<dyn Error>> {
    credentials::run(Box::pin(async {
        for (action, looping) in [("stop", false), ("cancel", false), ("complete", false), ("stop", true)] {
            // Start a Real Host With a Manually Advanced Execution Clock.
            let root = credentials::tempdir()?;
            let clock = ManualClock {
                source: ControlledClock::manual(0),
                waiting: Arc::new(Notify::new()),
            };
            let settings = HostSettings {
                installation: None,
                placement: PlacementSettings {
                    reconnect_delay: Duration::from_secs(1),
                    stream_window: NonZeroUsize::new(8).unwrap(),
                    interval_window: crate::INTERVAL_WINDOW,
                    interval_lag: crate::INTERVAL_LAG,
                    message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                },
                storage: dokime_storage::storage_defaults(),
                clock: Arc::new(clock.clone()),
                capture: crate::CaptureSettings::default(),
                credentials: credentials::namespace(),
            };
            let policies = ClientSettings {
                read_timeout: crate::PLUGINS_TIMEOUT,
                mutation_timeout: crate::PLUGINS_TIMEOUT,
                watch_timeout: harmos_protocol::policy::UNBOUNDED,
                max_cases: 1000,
                page_items: 50,
                stream_buffer: 8,
                message_bytes: 4 * 1024 * 1024,
            };
            let server = services::serve("127.0.0.1:0".parse()?, root.path(), settings, policies).await?;
            let client = dokime::client::connect_with(dokime::client::ConnectionSettings {
                target: Some(format!("http://{}", server.endpoint())),
                ..Default::default()
            })
            .await?;

            // Register Native SDK Handlers, Records and a Deterministic Stream.
            let stream = Stream::<Reading>::new("readings", "Readings").publish_on(async |context: &Context, publisher| {
                publisher.publish(Reading { value: 42 }).await?;
                context.plugin::<Probe>()?.sampled.notify_one();
                Ok(())
            });
            let template = Template::of::<()>()
                .identifier("wait")
                .label("Wait")
                .record(Record::<i64>::new("before", "Before"))
                .record(Record::<Reading>::new("after", "After"))
                .record(Record::<bool>::new("later_result", "Later Result"))
                .record(Record::<bool>::new("evaluated", "Evaluated"))
                .record(Record::<i64>::new("cleanup", "Cleanup"))
                .stream(stream)
                .step(Step::new("Wait", wait))
                .step(
                    Step::new("Later", async |context: &Context| {
                        context.record(&Record::new("later_result", "Later Result"), true).await?;
                        Ok(())
                    })
                    .identifier("later"),
                )
                .criterion(Criterion::new("Evaluate", evaluate))
                .cleanup(async |context: &Context| {
                    context.record(&Record::new("cleanup", "Cleanup"), context.termination().unwrap() as i64).await?;
                    Ok(())
                });
            let plugin = Plugin::of::<()>("1.0.0")
                .identifier("wait_fixture")
                .state(Probe { sampled: Notify::new(), looping })
                .suite(Suite::of::<()>().identifier("wait_fixture.suite").label("Waits").template(template));
            let registered = client.plugins().register(plugin).await?;
            let host = &server.host;
            let cases = client.cases().materialize::<()>().await?;
            let cycle = client.cycle("Waits").await?;
            let execution = cycle.execution(&cases[0]).await?;
            let execution = execution.identifier().to_owned();

            // Wait Until the Actual Host Sleep Is Pending, Then Stop, Cancel or Advance It.
            let run = host.start_execution(&execution).await?;
            clock.waiting.notified().await;
            match action {
                "stop" => host.stop_execution(&execution)?,
                "cancel" => host.cancel(&execution)?,
                _ => clock.wait_until(SLEEP.as_nanos() as i64).await?,
            }
            let result = run.wait().await?;
            let snapshot = &result.executions[0];
            let termination = match action {
                "stop" => Termination::Stopped,
                "cancel" => Termination::Cancelled,
                _ => Termination::Completed,
            };
            assert_eq!(snapshot.state, ExecutionState::Sealed);
            assert_eq!(snapshot.execution.termination, termination as i32);
            assert_eq!(clock.now_ns(), if action == "complete" { SLEEP.as_nanos() as i64 } else { 0 });
            assert_eq!(snapshot.records("wait_fixture.suite.wait.before").len(), 1);
            assert_eq!(snapshot.records("wait_fixture.suite.wait.after").len(), usize::from(action != "cancel"));
            assert_eq!(snapshot.records("wait_fixture.suite.wait.evaluated").len(), usize::from(action != "cancel"));
            assert_eq!(snapshot.records("wait_fixture.suite.wait.cleanup").len(), 1);
            assert_eq!(snapshot.records("wait_fixture.suite.wait.later_result").len(), usize::from(action == "complete"));
            if action != "cancel" {
                let after = RecordsAppendRequest::decode(snapshot.records("wait_fixture.suite.wait.after")[0].payload.as_slice())?;
                let after = Reading::decode(after.payload.as_slice())?;
                assert_eq!(after.value, if action == "complete" { SLEEP.as_secs() } else { 0 });
            }
            assert_eq!(snapshot.evidence.iter().any(|row| row.kind == "criterion:wait_fixture.suite.wait.evaluate"), action != "cancel");
            assert!(!snapshot.samples.is_empty());
            assert!(host.streams_closed(&execution)?);
            let cases = dokime_operations::host::cases::list::handle(
                &host.assembly.as_ref().unwrap().context,
                CasesListRequest {
                    parent: snapshot.execution.cycle.clone(),
                    ..Default::default()
                },
            )
            .await?;
            assert_eq!(
                cases.cases[0].status,
                match action {
                    "stop" => CaseStatus::Stopped,
                    "cancel" => CaseStatus::Cancelled,
                    _ => CaseStatus::Passed,
                } as i32
            );

            // Release Only the Listener and Host This Test Owns.
            registered.unregister().await?;
            client.close().await?;
            server.close().await?;
        }
        Ok(())
    }))
}
