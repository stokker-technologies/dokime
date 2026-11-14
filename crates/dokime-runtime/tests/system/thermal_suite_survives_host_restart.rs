/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Thermal Suite Survives Host Restart

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Termination;
use dokime::Value;
use dokime::client::ResultStatus;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_runtime::Host;
use dokime_runtime::PluginProcess;
use dokime_runtime::RunState;
use dokime_storage::ExecutionState;
use dokime_storage::OutcomeCounts;
use std::collections::HashMap;
use std::error::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn thermal_suite_survives_host_restart() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare a Fresh Evidence Root and the Shared SDK Plugin
        let root = super::support::tempdir()?;
        let executable = super::support::thermal_fixture()?;
        let settings = super::support::settings();
        let mut remote = PluginProcess::start(&executable, true, &settings.placement).await?;
        let host = Host::open(root.path(), settings.clone())?;
        let mut snapshots = Vec::new();

        // Add the Same Suite Through Remote and Managed Entries
        for managed in [false, true] {
            let entry = if managed { host.add_managed(&executable).await? } else { host.add_remote(remote.endpoint()).await? };
            let catalog = host.catalog(&entry)?;
            assert_eq!(catalog.suites[0].templates[0].metadata.as_ref().unwrap().identifier, "thermal.cycle");

            // Materialize Purely and Freeze Catalog and Process Identity
            let parameters = HashMap::from([("limit".into(), Value { kind: Some(ValueKind::Number(85.0)) })]);
            let origin = host.materialize(&entry, "thermal.cycle", parameters.clone()).await?;
            assert_eq!(origin, host.materialize(&entry, "thermal.cycle", parameters).await?);
            assert!(host.executions()?.is_empty() || snapshots.len() == 1);
            let Some(OriginKind::PluginRun(run)) = origin.kind.as_ref() else { panic!("Expected PluginRun") };
            assert_eq!(run.digest, host.description(&entry)?.declarations.unwrap().digest);
            assert_eq!(run.started_at, host.description(&entry)?.started_at);
            assert_eq!(run.test_case.as_ref().unwrap().steps.len(), 2);

            let start_ns = host.now_ns();

            // Run the Cycle and Answer the Door Prompt During Hold
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            let prompt = host.next_prompt(&execution).await?;
            host.answer(
                &execution,
                PromptsCreateResponse {
                    prompt: prompt.prompt,
                    action: "yes".into(),
                    accepted: true,
                    ..Default::default()
                },
            )?;
            let result = cycle.wait().await?;
            let snapshot = &result.executions[0];
            assert_eq!(snapshot.state, ExecutionState::Sealed);
            assert_eq!(snapshot.execution.termination, Termination::Completed as i32);
            assert_eq!(
                result.counts,
                OutcomeCounts {
                    evaluated: 1,
                    not_evaluated: 0,
                    declared: 0,
                    verdict: Some(ResultStatus::Passed)
                }
            );
            assert_eq!(
                result.states,
                vec![
                    RunState::Queued,
                    RunState::Initializing,
                    RunState::Stepping,
                    RunState::Evaluating,
                    RunState::CleaningUp,
                    RunState::Finished
                ]
            );
            assert_eq!(snapshot.samples.len(), 3);
            assert_eq!(
                snapshot.samples.iter().map(|sample| sample.source_ns).collect::<Vec<_>>(),
                vec![start_ns, start_ns + 1_000_000_000, start_ns + 2_000_000_000]
            );
            assert!(snapshot.samples.iter().all(|sample| sample.receipt_ns >= sample.source_ns));
            assert!(snapshot.samples.windows(2).all(|pair| pair[0].position < pair[1].position));
            assert_eq!(snapshot.records("thermal.door").len(), 1);
            assert_eq!(snapshot.logs()?.len(), 2);
            assert_eq!(snapshot.progress()?.iter().map(|progress| progress.completed).collect::<Vec<_>>(), vec![1, 2]);
            assert!(!snapshot.seal.is_empty());
            assert!(host.cancel(&execution).is_err());
            assert!(host.streams_closed(&execution)?);
            snapshots.push(snapshot.clone());
            host.remove_plugin(&entry).await?;
        }

        // Restart the Host and Read Identical Sealed Evidence
        host.shutdown().await?;
        drop(host);
        let host = Host::open(root.path(), settings)?;
        for snapshot in snapshots {
            assert_eq!(host.execution(&snapshot.execution.identifier)?, snapshot);
        }
        host.shutdown().await?;
        remote.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
}

/* === Tests === */
