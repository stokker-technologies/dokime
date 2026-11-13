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
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::{ControlledClock, Host, RenderSettings};
use dokime_storage::ExecutionState;

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

#[tokio::test]
async fn execution_shutdown_joins_runs_after_observer_loss() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Admit Two Executions on the Controlled Clock
        let root = super::support::tempdir()?;
        let mut settings = super::support::settings();
        settings.clock = Arc::new(ControlledClock::new(0));
        let host = Host::open(root.path(), settings.clone())?;
        let entry = host.add_managed(&super::support::thermal_fixture()?).await?;
        let origin = host
            .materialize(
                &entry,
                "thermal.cycle",
                HashMap::from([(
                    "limit".into(),
                    v1::Value {
                        kind: Some(v1::value::Kind::Number(85.0)),
                    },
                )]),
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
            assert_eq!(snapshot.execution.termination, v1::Termination::Cancelled as i32);
            assert!(host.streams_closed(execution)?);
            assert!(
                snapshot.evidence.iter().any(|row| row.kind == "log"
                    && row.operation == host.catalog(&entry).unwrap().suites[0].templates[0].cleanup.clone().unwrap())
            );
            assert!(!host.capabilities(execution)?.cancel);
            assert!(host.next_prompt(execution).await.is_err());
            saved.push(snapshot);
        }

        // Reopen Only Durable Evidence, Without a Live Run or Question
        drop(host);
        let host = Host::open(root.path(), settings)?;
        for (execution, snapshot) in executions.iter().zip(saved) {
            assert_eq!(host.execution(execution)?, snapshot);
        }
        assert!(host.pending_prompts()?.is_empty());
        host.shutdown().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}

#[tokio::test]
async fn prompts_settle_answers_and_shutdown_waiters() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Refuse Cancel While the Admitted Execution Is Still Queued
        let root = super::support::tempdir()?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::thermal_fixture()?).await?;
        let origin = host
            .materialize(
                &entry,
                "thermal.cycle",
                HashMap::from([(
                    "limit".into(),
                    v1::Value {
                        kind: Some(v1::value::Kind::Number(85.0)),
                    },
                )]),
            )
            .await?;
        let cycle = host.start_cycle(vec![origin.clone()]).await?;
        let execution = cycle.executions[0].clone();
        assert!(matches!(host.cancel(&execution), Err(dokime_runtime_next::Error::Invalid(_))));
        assert!(!host.capabilities(&execution)?.cancel);

        // Stop Leaves the Current Question Available Until Its Safe Boundary
        let question = host.next_prompt(&execution).await?;
        host.stop_execution(&execution)?;
        assert_eq!(host.execution(&execution)?.state, ExecutionState::Open);
        assert_eq!(host.pending_prompts()?.len(), 1);
        let answer = dokime_protocol::context::PromptsCreateResponse {
            prompt: question.prompt,
            accepted: true,
            action: "continue".into(),
            ..Default::default()
        };
        host.record_answer(&execution, answer.clone())?;
        assert!(host.record_answer(&execution, answer).is_err());
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
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}

#[tokio::test]
async fn reports_publish_atomically_and_close_admission() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Run and Seal the Evidence Before Rendering
        let root = super::support::tempdir()?;
        let settings = super::support::settings();
        let host = Host::open(root.path(), settings.clone())?;
        let entry = host.add_managed(&super::support::thermal_fixture()?).await?;
        let origin = host
            .materialize(
                &entry,
                "thermal.cycle",
                HashMap::from([(
                    "limit".into(),
                    v1::Value {
                        kind: Some(v1::value::Kind::Number(85.0)),
                    },
                )]),
            )
            .await?;
        let cycle = host.start_cycle(vec![origin]).await?;
        let execution = cycle.executions[0].clone();
        let question = host.next_prompt(&execution).await?;
        super::support::answer(&host, &execution, question)?;
        cycle.wait().await?;
        let before = host.execution(&execution)?;
        let document = v1::Document::new("Execution Evidence");
        let snapshot = host.snapshot(&execution)?;
        let output = host.render_document(
            &snapshot,
            document,
            RenderSettings {
                page_size: NonZeroUsize::new(2).unwrap(),
                display_points: NonZeroUsize::new(2).unwrap(),
            },
        )?;
        assert!(host.reports(&execution)?.is_empty());
        assert_eq!(host.execution(&execution)?, before);

        // Explicit Publication Commits the Whole Document and HTML
        let report = host.publish_report(&output)?;
        assert_eq!(report.html, output.html()?);
        assert_eq!(host.reports(&execution)?, vec![report.clone()]);
        assert_eq!(host.execution(&execution)?, before);

        // Shutdown Closes Publication but Preserves Storage Reads and Pure Rendering
        host.shutdown().await?;
        assert!(host.publish_report(&output).is_err());
        assert_eq!(host.reports(&execution)?, vec![report.clone()]);
        let rendered = host.render_document(
            &snapshot,
            v1::Document::new("Preview"),
            RenderSettings {
                page_size: NonZeroUsize::new(2).unwrap(),
                display_points: NonZeroUsize::new(2).unwrap(),
            },
        )?;
        assert!(!rendered.html()?.is_empty());
        assert_eq!(host.reports(&execution)?, vec![report.clone()]);
        drop(snapshot);
        drop(host);
        let host = Host::open(root.path(), settings)?;
        assert_eq!(host.reports(&execution)?, vec![report]);
        assert_eq!(host.execution(&execution)?, before);
        host.shutdown().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
