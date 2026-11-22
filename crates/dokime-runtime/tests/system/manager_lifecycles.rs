/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manager Lifecycle Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::Termination;
use dokime::Value;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::model::value;
use dokime_runtime::ControlledClock;

use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{ReportSubject, ReportsCreateRequest, ReportsPublishRequest, report_subject};
use dokime_runtime::{Host, services};
use dokime_storage::ExecutionState;
use dokime_storage::Store;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn execution_shutdown_joins_runs_after_observer_loss() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Admit Two Executions on the Controlled Clock
            let root = super::support::tempdir()?;
            let mut settings = super::support::settings();
            settings.clock = Arc::new(ControlledClock::new(0));
            let host = Host::open(root.path(), settings.clone(), super::support::service_settings()).await?;
            let entry = host.add_managed("thermal", &super::support::thermal_fixture()?, &[]).await?;
            let cleanup = host.catalog(&entry)?.suites[0].templates[0].cleanup.clone().unwrap();
            let origin = host
                .materialize(
                    &entry,
                    "thermal.thermal.cycle",
                    HashMap::from([("limit".into(), Value { kind: Some(value::Kind::Number(85.0)) })]),
                )
                .await?;
            let cycle = host.start_cycle(vec![origin.clone(), origin.clone()]).await?;
            let executions = cycle.executions.clone();
            host.next_prompt(&executions[0]).await?;
            host.next_prompt(&executions[1]).await?;

            // Abandon Observation and Await the Manager's Shutdown
            drop(cycle);
            host.shutdown().await?;
            assert!(host.pending_prompts()?.is_empty());
            assert!(host.start_cycle(vec![origin]).await.is_err());
            let mut saved = Vec::new();
            for execution in &executions {
                let snapshot = host.execution(execution)?;
                assert_eq!(snapshot.state, ExecutionState::Sealed);
                assert_eq!(snapshot.execution.termination, Termination::Cancelled as i32);
                assert!(host.streams_closed(execution)?);
                assert!(snapshot.evidence.iter().any(|row| row.kind == "log" && row.operation == cleanup));
                assert!(!host.capabilities(execution)?.cancel);
                assert!(host.next_prompt(execution).await.is_err());
                saved.push(snapshot);
            }

            // Reopen Only Durable Evidence, Without a Live Run or Question
            drop(host);
            let host = Host::open(root.path(), settings, super::support::service_settings()).await?;
            for (execution, snapshot) in executions.iter().zip(saved) {
                assert_eq!(host.execution(execution)?, snapshot);
            }
            assert!(host.pending_prompts()?.is_empty());
            host.shutdown().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn prompts_settle_answers_and_shutdown_waiters() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Refuse Cancel While the Admitted Execution Is Still Queued
            let root = super::support::tempdir()?;
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("thermal", &super::support::thermal_fixture()?, &[]).await?;
            let origin = host
                .materialize(
                    &entry,
                    "thermal.thermal.cycle",
                    HashMap::from([("limit".into(), Value { kind: Some(value::Kind::Number(85.0)) })]),
                )
                .await?;
            let cycle = host.start_cycle(vec![origin.clone()]).await?;
            let execution = cycle.executions[0].clone();
            assert!(matches!(host.cancel(&execution), Err(dokime_runtime::Error::Invalid(_))));
            assert!(!host.capabilities(&execution)?.cancel);

            // Stop Leaves the Current Question Available Until Its Safe Boundary
            let question = host.next_prompt(&execution).await?;
            host.stop_execution(&execution)?;
            assert_eq!(host.execution(&execution)?.state, ExecutionState::Open);
            assert_eq!(host.pending_prompts()?.len(), 1);
            let answer = PromptsCreateResponse {
                prompt: question.prompt,
                accepted: true,
                action: "0".into(),
                ..Default::default()
            };
            let receipt = host.record_answer(&execution, "answer", &[1], answer.clone())?;
            assert_eq!(host.record_answer(&execution, "answer", &[1], answer.clone())?, receipt);
            assert!(host.record_answer(&execution, "answer", &[2], answer).is_err());
            cycle.wait().await?;
            assert!(host.pending_prompts()?.is_empty());
            assert!(host.execution(&execution)?.evidence.iter().any(|row| row.kind == "client:answer"));

            // Shutdown Settles an Unanswered Question and Refuses Late Answers
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            let question = host.next_prompt(&execution).await?;
            host.shutdown().await?;
            cycle.wait().await?;
            assert!(host.pending_prompts()?.is_empty());
            assert!(super::support::answer(&host, &execution, question).is_err());
            assert!(host.next_prompt(&execution).await.is_err());
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn reports_publish_atomically_and_close_admission() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Run and Seal the Evidence Before Rendering
            let root = super::support::tempdir()?;
            let settings = super::support::settings();
            let host = Host::open(root.path(), settings.clone(), super::support::service_settings()).await?;
            let entry = host.add_managed("thermal", &super::support::thermal_fixture()?, &[]).await?;
            let origin = host
                .materialize(
                    &entry,
                    "thermal.thermal.cycle",
                    HashMap::from([("limit".into(), Value { kind: Some(value::Kind::Number(85.0)) })]),
                )
                .await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            let question = host.next_prompt(&execution).await?;
            super::support::answer(&host, &execution, question)?;
            cycle.wait().await?;
            let before = host.execution(&execution)?;
            host.shutdown().await?;
            drop(host);
            let server =
                services::serve(([127, 0, 0, 1], 0).into(), root.path(), settings.clone(), super::support::service_settings()).await?;
            let mut reports = ReportsClient::connect(format!("http://{}", server.endpoint())).await?;
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution(execution.clone())),
            };
            let prepared = reports
                .create(ReportsCreateRequest {
                    subject: Some(subject.clone()),
                    request_id: "lifecycle-report".into(),
                    ..Default::default()
                })
                .await?
                .into_inner();
            let file = prepared.file.unwrap();
            let store = Store::open(root.path())?;
            assert!(store.reports(&execution)?.is_empty());
            assert_eq!(store.read(&execution)?, before);
            let request = ReportsPublishRequest {
                name: file.metadata.unwrap().id,
                subject: Some(subject),
                expected_revision: file.sha256,
                request_id: "lifecycle-publish".into(),
            };
            reports.publish(request.clone()).await?;
            let report = store.reports(&execution)?.remove(0);
            assert_eq!(store.read(&execution)?, before);

            // Shutdown Closes Publication and Preserves Published Reports.
            server.close().await?;
            assert!(reports.publish(request).await.is_err());
            assert_eq!(store.reports(&execution)?, vec![report.clone()]);
            assert_eq!(store.read(&execution)?, before);
            drop(store);
            let host = Host::open(root.path(), settings, super::support::service_settings()).await?;
            assert_eq!(host.reports(&execution)?, vec![report]);
            assert_eq!(host.execution(&execution)?, before);
            host.shutdown().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
