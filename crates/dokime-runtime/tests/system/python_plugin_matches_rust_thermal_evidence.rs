/* ***
 *
 * dokime-rust-workspace
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

// Third Party Imports

// Local Imports
use dokime::Termination;
use dokime::Value;
use dokime::client::ResultStatus;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::model::value;
use dokime_runtime::Host;
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

#[test]
fn python_plugin_matches_rust_thermal_evidence() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Run the Rust Reference and Each SDK Placement with the Same Controlled Inputs
            let mut reference = None;
            for placement in [None].into_iter().chain([false].into_iter().map(Some)) {
                let root = super::support::tempdir()?;
                let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
                let mut python = None;
                let entry = match placement {
                    None => host.add_managed("thermal", &super::support::thermal_fixture()?, &[]).await?,
                    Some(_) => {
                        let child = super::support::PythonPlugin::start().await?;
                        let entry = host.add_remote("thermal", &child.url).await?;
                        python = Some(child);
                        entry
                    }
                };

                // Materialize the Complete Frozen Case and Await the Door Event
                let parameters = HashMap::from([("limit".into(), Value { kind: Some(value::Kind::Number(85.0)) })]);
                let origin = host.materialize(&entry, "thermal.cycle", parameters).await?;
                let cycle = host.start_cycle(vec![origin]).await?;
                let execution = cycle.executions[0].clone();
                let prompt = host.next_prompt(&execution).await?;
                host.answer(
                    &execution,
                    PromptsCreateResponse {
                        prompt: prompt.prompt,
                        action: "0".into(),
                        accepted: true,
                        ..Default::default()
                    },
                )?;
                let result = cycle.wait().await?;
                let snapshot = result.executions[0].clone();
                assert_eq!(snapshot.state, ExecutionState::Sealed);
                assert_eq!(
                    snapshot.execution.termination,
                    Termination::Completed as i32,
                    "Execution Outcome: {:?}",
                    snapshot.execution.outcome
                );
                assert_eq!(
                    result.counts,
                    OutcomeCounts {
                        evaluated: 1,
                        not_evaluated: 0,
                        declared: 0,
                        verdict: Some(ResultStatus::Passed)
                    }
                );
                assert_eq!(snapshot.samples.len(), 2502);
                assert_eq!(
                    snapshot
                        .samples
                        .iter()
                        .filter(|sample| sample.stream == "thermal.cycle.temperature")
                        .map(|sample| sample.source_ns)
                        .collect::<Vec<_>>(),
                    vec![1_000_000_000, 2_000_000_000]
                );
                assert_eq!(snapshot.records("thermal.cycle.door").len(), 1);
                assert!(host.streams_closed(&execution)?);
                super::support::thermal::frozen_record_schema_decodes_door(&snapshot);
                if let Some(expected) = &reference {
                    super::support::thermal::equivalent(expected, &snapshot);
                } else {
                    reference = Some(snapshot.clone());
                }

                // Reopen the Sealed Evidence After Removing Every Live Plugin
                host.remove_plugin(&entry).await?;
                host.shutdown().await?;
                if let Some(child) = python {
                    child.close().await?;
                }
                drop(host);
                let reopened = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
                assert_eq!(reopened.execution(&execution)?, snapshot);
                reopened.shutdown().await?;
            }
            Ok(())
        })
        .await?
    }))
}

/* === Tests === */
