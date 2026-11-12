/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SPDX-License-Identifier: Apache-2.0

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::{Host, PluginProcess};
use dokime_storage::{ExecutionState, OutcomeCounts};

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
async fn csharp_plugin_matches_rust_thermal_evidence() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Run the Rust Reference and Each SDK Placement with the Same Controlled Inputs
        let mut reference = None;
        for placement in [None].into_iter().chain([false, true].into_iter().map(Some)) {
            let root = tempfile::tempdir()?;
            let host = Host::open(root.path(), super::support::settings())?;
            let mut remote = None;
            let entry = match placement {
                None => host.add_managed(&super::support::thermal_fixture()?).await?,
                Some(managed) => {
                    let executable = super::support::language_fixture("csharp")?;
                    if managed {
                        host.add_managed(&executable).await?
                    } else {
                        let child = PluginProcess::start(&executable, true, &super::support::settings().placement).await?;
                        let entry = host.add_remote(child.endpoint()).await?;
                        remote = Some(child);
                        entry
                    }
                }
            };

            // Materialize the Complete Frozen Case and Await the Door Event
            let parameters = HashMap::from([(
                "limit".into(),
                v1::Value {
                    kind: Some(v1::value::Kind::Number(85.0)),
                },
            )]);
            let origin = host.materialize(&entry, "thermal.cycle", parameters).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            let prompt = host.next_prompt(&execution).await?;
            host.answer(
                &execution,
                v1::OperatorAnswer {
                    prompt: prompt.prompt,
                    action: "yes".into(),
                    accepted: true,
                    ..Default::default()
                },
            )?;
            let result = cycle.wait().await?;
            let snapshot = result.executions[0].clone();
            assert_eq!(snapshot.state, ExecutionState::Sealed);
            assert_eq!(snapshot.execution.termination, v1::Termination::Completed as i32);
            assert_eq!(
                result.counts,
                OutcomeCounts {
                    evaluated: 1,
                    not_evaluated: 0,
                    declared: 0,
                    verdict: Some(v1::ResultStatus::Passed)
                }
            );
            assert_eq!(snapshot.samples.len(), 3);
            assert_eq!(snapshot.samples.iter().map(|sample| sample.source_ns).collect::<Vec<_>>(), vec![0, 1_000_000_000, 2_000_000_000]);
            assert_eq!(snapshot.records("thermal.door").len(), 1);
            assert!(host.streams_closed(&execution)?);
            // csharp_frozen_record_schema_decodes_door
            super::support::thermal::frozen_record_schema_decodes_door(&snapshot);
            if let Some(expected) = &reference {
                super::support::thermal::equivalent(expected, &snapshot);
            } else {
                reference = Some(snapshot.clone());
            }

            // Reopen the Sealed Evidence After Removing Every Live Plugin
            host.remove_plugin(&entry).await?;
            host.shutdown().await?;
            if let Some(mut child) = remote {
                child.close().await?;
            }
            drop(host);
            let reopened = Host::open(root.path(), super::support::settings())?;
            assert_eq!(reopened.execution(&execution)?, snapshot);
            reopened.shutdown().await?;
        }
        Ok(())
    })
    .await?
}

/* === Tests === */
