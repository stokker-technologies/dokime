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
use std::collections::HashMap;

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::{Host, RunState};
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

#[tokio::test]
async fn cancelling_a_thermal_step_runs_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        // Add the Shared Suite as a Managed Plugin
        let root = tempfile::tempdir()?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::thermal_fixture()?).await?;
        let parameters = HashMap::from([(
            "limit".into(),
            v1::Value {
                kind: Some(v1::value::Kind::Number(85.0)),
            },
        )]);
        let origin = host.materialize(&entry, "thermal.cycle", parameters).await?;

        // Cancel While the Hold Step Waits for the Operator
        let cycle = host.start_cycle(vec![origin]).await?;
        let execution = cycle.executions[0].clone();
        host.next_prompt(&execution).await?;
        host.cancel(&execution)?;
        let result = cycle.wait().await?;
        let snapshot = &result.executions[0];
        assert_eq!(
            result.states,
            vec![RunState::Queued, RunState::Initializing, RunState::Stepping, RunState::CleaningUp, RunState::Cancelled]
        );
        assert_eq!(snapshot.state, ExecutionState::Sealed);
        assert_eq!(snapshot.execution.termination, v1::Termination::Cancelled as i32);
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
        let host = Host::open(root.path(), super::support::settings())?;
        assert_eq!(host.execution(&execution)?, *snapshot);
        host.shutdown().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}

/* === Tests === */
