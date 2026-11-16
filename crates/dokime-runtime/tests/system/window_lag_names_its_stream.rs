/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Stream Whose Windows Fall Behind Past the Lag Limit Interrupts Its Execution and Names Its Stream

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, error::Error, num::NonZeroU32, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use super::support;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::model::Termination;
use dokime_runtime::{Host, HostSettings, PlacementSettings, services::WallClock};

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
fn window_lag_names_its_stream() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // A Real-Clock Host With 10 ms Windows Tolerates One Window of Lag; Each Answer Takes 100 ms.
        let root = support::tempdir()?;
        let defaults = support::settings();
        let settings = HostSettings {
            clock: Arc::new(WallClock),
            placement: PlacementSettings {
                interval_window: Duration::from_millis(10),
                interval_lag: NonZeroU32::MIN,
                ..defaults.placement.clone()
            },
            ..defaults
        };
        let host = Host::open(root.path(), settings, support::service_settings()).await?;
        let entry = host.add_managed("boundary", &support::boundary_fixture()?, &[]).await?;
        let origin = host.materialize(&entry, "execution.lag", HashMap::new()).await?;
        let cycle = host.start_cycle(vec![origin]).await?.wait().await?;
        let execution = &cycle.executions[0];

        // The Failed Grant Ends the Execution Interrupted, and Its Reason and Stream Failure Name the Stream.
        let outcome = execution.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.clone());
        let Some(Attribution::NotEvaluated(outcome)) = outcome else {
            panic!("Fall Behind a Window: Expected an Unevaluated Outcome, Found {outcome:?}");
        };
        assert_eq!(Termination::try_from(outcome.termination)?, Termination::Interrupted, "Fall Behind a Window: {}", outcome.reason);
        assert!(outcome.reason.contains("execution.window.lag"), "Fall Behind a Window: Name the Stream: {}", outcome.reason);
        let failures = execution
            .evidence
            .iter()
            .filter(|entry| entry.kind == "streams:failure")
            .map(|entry| String::from_utf8_lossy(&entry.payload).into_owned())
            .collect::<Vec<_>>();
        assert!(
            failures.iter().any(|failure| failure.contains("execution.window.lag")),
            "Fall Behind a Window: Name the Stream in Its Failure: {failures:?}"
        );
        host.shutdown().await?;
        Ok(())
    }))
}
