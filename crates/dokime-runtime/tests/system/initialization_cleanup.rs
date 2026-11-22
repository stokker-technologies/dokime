/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Clean Partially Initialized Execution Resources Exactly Once

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;

// Third Party Imports

// Local Imports
use dokime_protocol::model::{Termination, Value, value};
use dokime_runtime::Host;
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

#[test]
fn initialization_failure_and_cancellation_clean_partial_resources_once() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Admit the Execution Fixture on a Fresh Host.
            let root = super::support::tempdir()?;
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("initialization", &super::support::execution_fixture()?, &[]).await?;
            let cleanup = host.catalog(&entry)?.suites[0].templates[0].cleanup.clone().unwrap();

            for (parameter, cancelled) in [("initialize_failure", false), ("initialize_question", true)] {
                // Initialize Publishes Its Resource Before Failing or Waiting for Cancellation.
                let mut parameters = super::support::parameters(17.0, 0, true);
                parameters.insert(parameter.into(), Value { kind: Some(value::Kind::Boolean(true)) });
                let origin = host.materialize(&entry, "execution.execution.capture", parameters).await?;
                let cycle = host.start_cycle(vec![origin]).await?;
                let execution = cycle.executions[0].clone();
                if cancelled {
                    host.next_prompt(&execution).await?;
                    host.cancel(&execution)?;
                }

                // Join the Run and Inspect Acknowledged Cleanup Evidence.
                let settled = cycle.wait().await?;
                let snapshot = &settled.executions[0];
                assert_eq!(snapshot.state, ExecutionState::Sealed);
                if cancelled {
                    assert_eq!(snapshot.execution.termination, Termination::Cancelled as i32);
                } else {
                    assert_eq!(
                        snapshot.execution.termination,
                        Termination::Failed as i32,
                        "{:?}",
                        snapshot.evidence.iter().map(|row| (&row.kind, String::from_utf8_lossy(&row.payload))).collect::<Vec<_>>()
                    );
                }
                assert_eq!(snapshot.evidence.iter().filter(|row| row.kind == "log" && row.operation == cleanup).count(), 1);
                assert!(!snapshot.evidence.iter().any(|row| row.kind == "step:start"));
                assert!(host.streams_closed(&execution)?);
            }

            // Shutdown Does Not Repeat Execution Cleanup.
            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
