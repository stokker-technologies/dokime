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
use std::io::{Cursor, Read};
use std::sync::Arc;

// Third Party Imports
use harmos_protocol::model::FileReference;
use tonic::transport::Channel;
use zip::ZipArchive;

// Local Imports
use dokime::Termination;
use dokime::Value;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::model::value;
use dokime_runtime::ControlledClock;

use dokime_protocol::host::evidence_client::EvidenceClient;
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{
    EvidenceArchiveRequest, EvidenceSubject, FilesReadRequest, ReportSubject, ReportsCreateRequest, ReportsPublishRequest,
    evidence_subject, report_subject,
};
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

async fn read(files: &mut FilesClient<Channel>, file: FileReference) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    while (bytes.len() as u64) < file.length {
        let request = FilesReadRequest {
            file: Some(file.clone()),
            offset: bytes.len() as u64,
            max_bytes: 0,
            at: None,
        };
        let reply = files.read(request).await?.into_inner();
        assert!(!reply.data.is_empty());
        bytes.extend(reply.data);
    }
    Ok(bytes)
}

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
fn sealing_freezes_the_report_download_and_export_deliver() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Every Termination Freezes Its Report Before the Cycle Completes
            let root = super::support::tempdir()?;
            let settings = super::support::settings();
            let host = Host::open(root.path(), settings.clone(), super::support::service_settings()).await?;
            let entry = host.add_managed("execution", &super::support::execution_fixture()?, &[]).await?;
            let mut frozen = Vec::new();
            for (failures, stop, termination) in
                [(0, false, Termination::Completed), (2, false, Termination::Failed), (0, true, Termination::Stopped)]
            {
                let origin =
                    host.materialize(&entry, "execution.execution.capture", super::support::parameters(17.0, failures, true)).await?;
                let cycle = host.start_cycle(vec![origin]).await?;
                let execution = cycle.executions[0].clone();
                let prompt = host.next_prompt(&execution).await?;
                if stop {
                    host.stop_execution(&execution)?;
                }
                super::support::answer(&host, &execution, prompt)?;
                cycle.wait().await?;
                assert_eq!(host.execution(&execution)?.execution.termination, termination as i32);
                let mut reports = host.reports(&execution)?;
                assert_eq!(reports.len(), 1, "Freeze One Report for {termination:?}");
                frozen.push((execution, reports.remove(0)));
            }
            host.shutdown().await?;
            drop(host);

            // Download and Export Deliver the Frozen Report Without Publishing Another
            let server =
                services::serve(([127, 0, 0, 1], 0).into(), root.path(), settings.clone(), super::support::service_settings()).await?;
            let endpoint = format!("http://{}", server.endpoint());
            let mut reports = ReportsClient::connect(endpoint.clone()).await?;
            let mut evidence = EvidenceClient::connect(endpoint.clone()).await?;
            let mut files = FilesClient::connect(endpoint).await?;
            let store = Store::open(root.path())?;
            for (execution, report) in &frozen {
                let download = ReportsCreateRequest {
                    subject: Some(ReportSubject {
                        kind: Some(report_subject::Kind::Execution(execution.clone())),
                    }),
                    request_id: format!("download-{execution}"),
                    ..Default::default()
                };
                let downloaded = reports.create(download.clone()).await?.into_inner();
                assert_eq!(downloaded.report.unwrap().identifier, report.identifier);
                let file = downloaded.file.unwrap();
                assert_eq!(read(&mut files, file.clone()).await?, report.html);
                let publish = ReportsPublishRequest {
                    subject: download.subject.clone(),
                    name: file.metadata.as_ref().unwrap().id.clone(),
                    expected_revision: file.sha256,
                    request_id: format!("publish-{execution}"),
                };
                assert_eq!(reports.publish(publish).await?.into_inner().report.unwrap().identifier, report.identifier);
                let request = EvidenceArchiveRequest {
                    subject: Some(EvidenceSubject {
                        kind: Some(evidence_subject::Kind::Execution(execution.clone())),
                    }),
                    at: None,
                };
                let archive = evidence.archive(request.clone()).await?.into_inner().file.unwrap();
                assert_eq!(evidence.archive(request).await?.into_inner().file.unwrap(), archive);
                let mut html = Vec::new();
                ZipArchive::new(Cursor::new(read(&mut files, archive).await?))?.by_name("report.html")?.read_to_end(&mut html)?;
                assert_eq!(html, report.html);
                assert_eq!(store.reports(execution)?, vec![report.clone()]);

                // Shutdown Closes Admission and Preserves the Frozen Report
                if execution == &frozen[2].0 {
                    server.close().await?;
                    assert!(reports.create(download).await.is_err());
                    break;
                }
            }
            drop(store);
            let host = Host::open(root.path(), settings, super::support::service_settings()).await?;
            for (execution, report) in &frozen {
                assert_eq!(host.reports(execution)?, vec![report.clone()]);
            }
            host.shutdown().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
