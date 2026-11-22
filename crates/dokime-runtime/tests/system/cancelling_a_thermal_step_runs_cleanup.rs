/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cancelling a Thermal Step Runs Cleanup

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Termination;
use dokime::Value;
use dokime_runtime::Host;
use dokime_runtime::RunState;
use dokime_storage::ExecutionState;
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
fn cancelling_a_thermal_step_runs_cleanup() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Add the Shared Suite as a Managed Plugin
            let root = super::support::tempdir()?;
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("thermal", &super::support::thermal_fixture()?, &[]).await?;
            let parameters = HashMap::from([(
                "limit".into(),
                Value {
                    kind: Some(dokime_protocol::model::value::Kind::Number(85.0)),
                },
            )]);
            let origin = host.materialize(&entry, "thermal.thermal.cycle", parameters).await?;

            // Cancel While the Hold Step Waits for the Operator
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            host.next_prompt(&execution).await?;
            host.cancel(&execution)?;
            let result = cycle.wait().await?;
            let snapshot = &result.executions[0];
            assert_eq!(
                result.states,
                vec![RunState::Queued, RunState::Initializing, RunState::Stepping, RunState::CleaningUp, RunState::Sealed]
            );
            assert_eq!(snapshot.state, ExecutionState::Sealed);
            assert_eq!(snapshot.execution.termination, Termination::Cancelled as i32);
            assert_eq!(result.counts.evaluated, 0);
            assert_eq!(result.counts.not_evaluated, 1);
            assert_eq!(result.counts.verdict, None);
            assert_eq!(snapshot.logs()?.len(), 2);
            let cleanup = host.catalog(&entry)?.suites[0].templates[0].cleanup.clone().unwrap();
            assert!(snapshot.evidence.iter().any(|entry| entry.kind == "log" && entry.operation == cleanup));
            assert!(snapshot.samples.is_empty());
            assert!(host.cancel(&execution).is_err());
            assert!(host.streams_closed(&execution)?);
            host.remove_plugin(&entry).await?;

            // Reopen the Cancelled Execution with Cleanup Evidence Intact
            host.shutdown().await?;
            drop(host);
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            assert_eq!(host.execution(&execution)?, *snapshot);
            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

/* === Tests === */
