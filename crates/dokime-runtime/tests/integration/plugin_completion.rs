/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Changes After Execution Completion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};
use dokime_runtime::domains::{ActionDispatcher, ActionDispatcherError, ActionDispatcherFuture, OptionProviderOption, StatusResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct AnalysisProbe(std::sync::atomic::AtomicUsize);

struct CompletionExecutor {
    executor: Arc<BlockingPublishingExecutor>,
    context: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<ExecutionFixtureContext>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for CompletionExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            if let Some(sender) = self.context.lock().unwrap().take() {
                let _ = sender.send(context.clone());
            }
            let mut outcome = self.executor.run_cycle(context, cycle, target).await?;
            for execution in &mut outcome.executions {
                execution.records.clear();
                execution.telemetry.clear();
            }
            Ok(outcome)
        })
    }
}

impl ActionDispatcher for AnalysisProbe {
    fn status<'a>(&'a self) -> ActionDispatcherFuture<'a, StatusResponse> {
        Box::pin(async { Ok(StatusResponse::ready()) })
    }

    fn query_options<'a>(&'a self, entrypoint: &'a str) -> ActionDispatcherFuture<'a, Vec<OptionProviderOption>> {
        Box::pin(async move { Err(ActionDispatcherError::UnknownEntrypoint(entrypoint.into())) })
    }

    fn analyze<'a>(
        &'a self, _: &'a str, request: dokime_runtime::analysis::Request,
    ) -> ActionDispatcherFuture<'a, dokime_runtime::analysis::Response> {
        Box::pin(async move {
            assert_eq!(self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst), 0, "completed analyses never call a live plugin again");
            let samples = request.samples.unwrap();
            Ok(dokime_runtime::analysis::Response {
                outputs: vec![dokime_runtime::analysis::Values {
                    key: "scaled".into(),
                    unit: "s".into(),
                    axis_unit: "s".into(),
                    axis: samples.axis,
                    values: samples.values.into_iter().map(|value| value * 3.0).collect(),
                }],
            })
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn plugin_sealing() {
    tokio::time::timeout(crate::service_support::budgets::settings::STORY_BOUND, async {
        use crate::service_support::{bundles, reports, waits};
        use dokime_domain::model::execution::result::CyclePhase;

        for fail in [false, true] {
            let root = tempdir().unwrap();
            let executor = Arc::new(BlockingPublishingExecutor::new());
            let analysis = Arc::new(AnalysisProbe(std::sync::atomic::AtomicUsize::new(0)));
            let definition = dokime_runtime::analysis::Definition {
                id: "fixture_analysis.scale".into(),
                version: 1,
                label: "Scale".into(),
                description: "Scale Retained Evidence".into(),
                parameters: vec![],
                outputs: vec![dokime_runtime::analysis::Output {
                    key: "scaled".into(),
                    label: "Scaled".into(),
                    unit: Some("s".into()),
                    axis: Some(dokime_runtime::analysis::Axis {
                        key: "time".into(),
                        unit: Some("s".into()),
                    }),
                }],
                limits: dokime_runtime::analysis::Limits { max_samples: 64, max_output_values: 64 },
            };
            let mut configuration = config(root.path()).with_plugins([bundles::host_plugin()]);
            configuration.installed_plugins.push(dokime_runtime::InstalledPlugin {
                source: "fixture_evidence".into(),
                version: "1.0.0".into(),
                enabled: true,
                telemetry_contracts: vec![minimal_tracking_contract()],
                ..Default::default()
            });
            configuration.installed_plugins.push(dokime_runtime::InstalledPlugin {
                source: "fixture_analysis".into(),
                version: "1.0.0".into(),
                description: "Frozen Analysis Fixture".into(),
                enabled: true,
                analyses: vec![definition],
                ..Default::default()
            });
            configuration = configuration.with_action_dispatcher(identifier("fixture_analysis"), analysis.clone());
            let (context_sender, context_receiver) = tokio::sync::oneshot::channel();
            let fixture = CompletionExecutor {
                executor: executor.clone(),
                context: std::sync::Mutex::new(Some(context_sender)),
            };
            let (mut owner, mut runtime) = started_with_fixture(configuration, Arc::new(fixture)).await;
            let mut plot = dokime_runtime::document::Plot::xy("Scaled Evidence");
            plot.id("fixture.analysis");
            plot.stream("tracking");
            plot.x = Some("elapsed_s".into());
            plot.y = vec!["elapsed_s".into()];
            plot.analysis = Some(dokime_runtime::analysis::Selection {
                id: "fixture_analysis.scale".into(),
                version: 1,
                parameters: Default::default(),
            });
            let mut section = dokime_runtime::document::DocumentSection::new("Analysis");
            section.plot(plot);
            let installed = runtime.install_plugin(std::fs::read(bundles::plugin_obc_bundle()).unwrap(), |_| Ok(())).await.unwrap();
            let package = installed.metadata.as_ref().unwrap().identity.name.clone();
            runtime.plugin_bundle_settled(&package).await;
            assert!(runtime.watch_state().latest().state().installed_plugins.contains_key("plugin_obc"));
            let registered = runtime
                .test_suites()
                .await
                .into_iter()
                .find(|suite| {
                    suite.source == "plugin_obc"
                        && suite
                            .suite
                            .test_case_templates
                            .iter()
                            .any(|template| template.metadata.identifier.as_str() == "obc.checks.pulse")
                })
                .expect("the installed fixture contributes its suite");
            let case = materialize_cases(&registered.suite_id, &registered.suite)
                .into_iter()
                .find(|case| case.metadata.identifier.as_str() == "obc.checks.pulse")
                .unwrap();
            let cycle = start_cases(&runtime, metadata("plugin.sealing", "Plugin Sealing"), vec![case]).await;
            let execution = runtime.watch_state().latest().state().cycles[&cycle].execution_ids[0].clone();
            // Hold Before The Worker Reads Any Report Context, Then Fail Rendering On Demand.
            let job = dokime_runtime::testing::hold_report_job(&execution);
            let rendering = dokime_runtime::testing::hold_report(&execution, fail);
            let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
            assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
            executor.wait_until_entered().await;
            let scope = runtime.watch_state().latest().state().executions[&execution].capture_scope();
            let context = context_receiver.await.unwrap();
            context.add_document_parts(scope.clone(), vec![dokime_runtime::document::DocumentAddition::section(section)]).await.unwrap();
            drop(context);
            let samples = [1.0, 2.0]
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    let row = provider_tracking_row(value, true);
                    let mut sample = TelemetrySample::new(
                        dokime_runtime::model::identity::new_id(),
                        identifier("tracking"),
                        index as u64,
                        dokime_runtime::model::time::now_utc(),
                        "fixture_evidence",
                    );
                    sample.scope = Some(scope.clone());
                    sample.values = row.values;
                    sample.storage_payload = Some(row.payload);
                    sample
                })
                .collect();
            runtime
                .publish_telemetry_batch(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier("tracking"), samples))
                .await
                .unwrap();
            executor.release();
            job.entered().await;
            waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
            assert_eq!(analysis.0.load(std::sync::atomic::Ordering::SeqCst), 1);
            let frozen = runtime.execution_snapshot(&execution).await.unwrap().context().unwrap().clone();
            assert!(!frozen.execution_documents.is_empty());

            let waiting = dokime_runtime::testing::hold_report_wait(&execution);
            let sealing = runtime.clone();
            let selected = cycle.clone();
            let seal = tokio::spawn(async move { sealing.seal_cycle(selected, CommandContext::default()).await });
            waiting.entered().await;
            waiting.release();
            assert_eq!(cycle_snapshot(&runtime, &cycle).await.phase, CyclePhase::Sealing);

            // Replacement And Removal Finish While The Report Worker Is Still Held.
            let replacement = runtime.install_plugin(bundles::plugin_obc_version("99.0.0"), |_| Ok(())).await.unwrap();
            runtime.plugin_bundle_settled(&package).await;
            let mut applied = runtime.plugin_store().get(replacement.installation.install_id).await.unwrap();
            runtime.plugin_process_state(&mut applied);
            assert!(applied.active, "{:?}", applied.installation.last_boot);
            runtime.remove_plugin(replacement.installation.install_id).await.unwrap();
            runtime.plugin_bundle_settled(&package).await;
            assert!(!runtime.watch_state().latest().state().installed_plugins.contains_key("plugin_obc"));
            assert!(runtime.plugin_store().list().await.unwrap().is_empty());
            assert!(!seal.is_finished());

            job.release();
            rendering.entered().await;
            rendering.release();
            assert_eq!(seal.await.unwrap().is_err(), fail);
            if fail {
                assert_eq!(cycle_snapshot(&runtime, &cycle).await.phase, CyclePhase::SealFailed);
                owner.shutdown().await.unwrap();
                drop(runtime);
                drop(owner);
                (owner, runtime) = booted(root.path()).await;
                runtime.seal_cycle(cycle.clone(), CommandContext::default()).await.unwrap();
            }
            let completed = cycle_snapshot(&runtime, &cycle).await;
            assert_eq!(completed.phase, CyclePhase::Sealed);
            assert_eq!(completed.test_executions[0].status, ExecutionStatus::Passed);
            let inspection = runtime.execution_snapshot(&execution).await.unwrap();
            assert_eq!(inspection.context().unwrap(), &frozen);
            let mut report = runtime.open_execution_report(&execution).await.unwrap();
            let bytes = reports::opened_bytes(&mut report).await;
            let payload = reports::parse(std::str::from_utf8(&bytes).unwrap());
            assert_eq!(
                payload["case_runs"][0]["plots"]["fixture.analysis"]["computed"]["series"][0]["output"]["outputs"][0]["values"],
                serde_json::json!([3.0, 6.0])
            );
            assert_eq!(analysis.0.load(std::sync::atomic::Ordering::SeqCst), 1);
            drop(report);
            drop(inspection);
            owner.shutdown().await.unwrap();
            drop(runtime);
            drop(owner);

            // History Reads And Published Reports Survive A Plugin-Free Restart.
            let (mut owner, runtime) = booted(root.path()).await;
            assert!(runtime.test_suites().await.iter().all(|suite| suite.source != "plugin_obc"));
            let inspection = runtime.execution_snapshot(&execution).await.unwrap();
            assert_eq!(inspection.context().unwrap(), &frozen);
            assert_eq!(inspection.execution().status, ExecutionStatus::Passed);
            let mut report = runtime.open_cycle_report(&cycle).await.unwrap();
            assert!(!reports::opened_bytes(&mut report).await.is_empty());
            drop(report);
            drop(inspection);
            owner.shutdown().await.unwrap();
        }
    })
    .await
    .expect("plugin sealing exceeded its hang guard");
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn plugin_removed_case() {
    tokio::time::timeout(crate::service_support::budgets::settings::STORY_BOUND, async {
        use crate::service_support::{bundles, waits};

        let root = tempdir().unwrap();
        let executor = Arc::new(BlockingPublishingExecutor::new());
        let mut configuration = config(root.path()).with_plugins([bundles::host_plugin()]);
        configuration.installed_plugins.push(dokime_runtime::InstalledPlugin {
            source: "fixture_evidence".into(),
            version: "1.0.0".into(),
            enabled: true,
            telemetry_contracts: vec![minimal_tracking_contract()],
            ..Default::default()
        });
        let fixture = CompletionExecutor {
            executor: executor.clone(),
            context: std::sync::Mutex::new(None),
        };
        let (mut owner, runtime) = started_with_fixture(configuration, Arc::new(fixture)).await;
        let installed = runtime.install_plugin(std::fs::read(bundles::plugin_obc_bundle()).unwrap(), |_| Ok(())).await.unwrap();
        let package = installed.metadata.as_ref().unwrap().identity.name.clone();
        runtime.plugin_bundle_settled(&package).await;
        let mut admitted = runtime.plugin_store().get(installed.installation.install_id).await.unwrap();
        runtime.plugin_process_state(&mut admitted);
        assert!(admitted.active, "{:?}", admitted.installation.last_boot);
        let registration = runtime
            .test_suites()
            .await
            .into_iter()
            .find(|entry| {
                entry.source == "plugin_obc"
                    && entry.suite.test_case_templates.iter().any(|template| template.metadata.identifier.as_str() == "obc.checks.pulse")
            })
            .expect("the installed fixture contributes its suite");
        let case = materialize_cases(&registration.suite_id, &registration.suite)
            .into_iter()
            .find(|case| case.metadata.identifier.as_str() == "obc.checks.pulse")
            .unwrap();
        let cycle = runtime
            .create_cycle(
                CycleDefinition {
                    metadata: metadata("removed.plugin", "Removed Plugin"),
                    continue_automatically: false,
                },
                CommandContext::default(),
            )
            .await
            .unwrap();
        runtime.append_fixture_cases(cycle.clone(), vec![case.clone(), case], CommandContext::default()).await.unwrap();
        let members = cycle_snapshot(&runtime, &cycle).await.test_executions;
        let execution = &members[0].test_execution_id;
        let report = dokime_runtime::testing::hold_report_job(execution);
        let receipt = runtime.start_execution(execution.clone(), Default::default(), CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        executor.wait_until_entered().await;
        executor.release();
        report.entered().await;
        waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
        runtime.remove_plugin(installed.installation.install_id).await.unwrap();
        runtime.plugin_bundle_settled(&package).await;
        assert!(runtime.execution_snapshot(execution).await.unwrap().execution().saved.is_none());
        let refused = runtime.start_execution(members[1].test_execution_id.clone(), Default::default(), CommandContext::default()).await;
        assert_eq!(refused.status, CommandStatus::Rejected);
        let rejection = refused.rejection.unwrap();
        assert_eq!(rejection.code, "execution_plugin_unavailable");
        assert!(!rejection.message.is_empty());
        report.release();
        runtime.wait_execution_report(execution).await.unwrap();
        runtime.seal_cycle(cycle, CommandContext::default()).await.unwrap();
        owner.shutdown().await.unwrap();
    })
    .await
    .expect("removed plugin story exceeded its hang guard");
}
