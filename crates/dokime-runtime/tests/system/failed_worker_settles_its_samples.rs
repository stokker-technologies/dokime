/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Failed Host Worker Drains Its Samples Before It Seals

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// Third Party Imports
use tonic::transport::Endpoint;

// Local Imports
use dokime::Client;
use dokime::EvidenceScope;
use dokime::Metadata;
use dokime::Termination;
use dokime::client::CasesCreateOptions;
use dokime::client::CyclesCreateOptions;
use dokime::client::ExecutionBinding;
use dokime::client::ExecutionMode;
use dokime::client::ExecutionState;
use dokime::client::ExecutionsCommandsCreateOptions;
use dokime::client::ExecutionsCreateOptions;
use dokime::client::ExecutionsExecutionCommandAction;
use dokime::client::ExecutionsStartOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::client::RegisteredSelection;
use dokime::client::SelectionPlan;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_runtime::ControlledClock;
use dokime_runtime::PluginProcess;
use dokime_runtime::Result as RuntimeResult;
use dokime_runtime::SchedulerClock;
use dokime_runtime::services;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use harmos_runtime::clock::{Clock as RuntimeClock, Controller};

/* === Constants === */

const ROWS: u64 = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Controlled Time Whose Armed Reading Fails the Host Worker That Takes It.
struct FailingClock {
    clock: ControlledClock,
    readings: AtomicU32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait::async_trait]
impl SchedulerClock for FailingClock {
    fn execution_clock(&self) -> (RuntimeClock, Option<Controller>) {
        self.clock.execution_clock()
    }
    fn now_ns(&self) -> i64 {
        if self.readings.try_update(Ordering::SeqCst, Ordering::SeqCst, |left| left.checked_sub(1)) == Ok(1) {
            panic!("Story Clock: Fail the Worker That Takes the Armed Reading");
        }
        self.clock.now_ns()
    }
    async fn wait_until(&self, at_ns: i64) -> RuntimeResult<()> {
        self.clock.wait_until(at_ns).await
    }
    async fn reached(&self, at_ns: i64) {
        self.clock.reached(at_ns).await;
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn run(client: &Client, cycle: &str, case: &str, execution: &str, mode: ExecutionMode) -> Result<(), Box<dyn Error>> {
    client
        .executions
        .create_with(
            cycle,
            vec![ExecutionBinding {
                execution_id: execution.into(),
                case_id: case.into(),
            }],
            ExecutionsCreateOptions {
                request_id: Some(uuid::Uuid::now_v7().to_string()),
                ..Default::default()
            },
        )
        .await?;
    client
        .executions
        .start_with(
            execution,
            mode,
            ExecutionsStartOptions {
                request_id: Some(uuid::Uuid::now_v7().to_string()),
                ..Default::default()
            },
        )
        .await?;
    Ok(())
}

async fn sealed(client: &Client, execution: &str) -> Result<(Termination, u64), Box<dyn Error>> {
    // Runtime Frames Wake the Read; a Failed Worker Ends Progress Watches With Its Failure.
    let mut runtime = client.backend.overview.watch().await?;
    let detail = loop {
        let detail = client.executions.get(execution).await?.execution.ok_or("Read Execution: Missing Detail")?;
        if detail.summary.as_ref().unwrap().state == ExecutionState::Sealed as i32 {
            break detail;
        }
        runtime.next().await?.ok_or("Await Sealed Execution: Runtime Watch Closed")?;
    };
    let termination = match detail.outcome.unwrap().attribution {
        Some(Attribution::NotEvaluated(outcome)) => Termination::try_from(outcome.termination)?,
        attribution => return Err(format!("Read Execution Outcome: Expected Not Evaluated, Found {attribution:?}").into()),
    };
    let scope = EvidenceScope {
        execution_id: Some(execution.into()),
        ..Default::default()
    };
    let count = client.evidence.statistics(scope, "execution.boundary", "value").await?.statistics.unwrap().count;
    Ok((termination, count))
}

/* === Tests === */

#[test]
fn failed_worker_settles_its_samples() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Serve a Host Whose Clock Can Fail One Worker Mid-Run
            let root = super::support::tempdir()?;
            let clock = Arc::new(FailingClock {
                clock: ControlledClock::new(0),
                readings: AtomicU32::new(0),
            });
            let mut settings = super::support::settings();
            settings.clock = clock.clone();
            let mut plugin = PluginProcess::start(&super::support::boundary_fixture()?, true, &settings.placement).await?;
            let server = services::serve("127.0.0.1:0".parse()?, root.path(), settings, super::support::service_settings()).await?;
            let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let entry = client
                .plugins
                .create(
                    "boundary",
                    PluginPlacement {
                        kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            let cycle = client
                .cycles
                .create_with(
                    Default::default(),
                    Metadata {
                        label: "Failed Worker".into(),
                        ..Default::default()
                    },
                    Default::default(),
                    String::new(),
                    CyclesCreateOptions {
                        request_id: Some(uuid::Uuid::now_v7().to_string()),
                        ..Default::default()
                    },
                )
                .await?
                .cycle
                .unwrap();
            let case = client
                .cases
                .create_with(
                    SelectionPlan {
                        cycle_id: cycle.cycle_id.clone(),
                        selections: vec![RegisteredSelection {
                            suite_id: "execution".into(),
                            template_id: "execution.boundary".into(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    false,
                    CasesCreateOptions {
                        request_id: Some(uuid::Uuid::now_v7().to_string()),
                        ..Default::default()
                    },
                )
                .await?
                .cases[0]
                .case_id
                .clone();

            // Fail the Worker After Its Step Published Every Row, With the Binding Still Open
            let failed = "019a0000-0000-7000-8000-00000000f001";
            run(&client, &cycle.cycle_id, &case, failed, ExecutionMode::Interactive).await?;
            // Readings: the Command's Receipt, Then the Step's Start and Completion.
            clock.readings.store(3, Ordering::SeqCst);
            client
                .executions
                .commands
                .create_with(
                    failed,
                    "capture",
                    ExecutionsExecutionCommandAction::Next,
                    ExecutionsCommandsCreateOptions {
                        request_id: Some(failed.into()),
                        ..Default::default()
                    },
                )
                .await?;
            let (termination, count) = sealed(&client, failed).await?;
            assert_eq!(termination, Termination::Interrupted);
            assert_eq!(count, ROWS, "Seal the Failed Worker's Evidence With Every Accepted Sample");

            // The Same Plugin Runs the Next Execution
            let next = "019a0000-0000-7000-8000-00000000f002";
            run(&client, &cycle.cycle_id, &case, next, ExecutionMode::Automatic).await?;
            assert_eq!(sealed(&client, next).await?, (Termination::Completed, ROWS));

            server.close().await?;
            plugin.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
