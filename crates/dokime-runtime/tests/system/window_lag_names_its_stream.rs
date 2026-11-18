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
use std::{collections::HashMap, error::Error, num::NonZeroU32, time::Duration};

// Third Party Imports

// Local Imports
use super::support;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::model::Termination;
use dokime_runtime::{Host, HostSettings, PlacementSettings};

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
        // A Controlled Host With 10 ms Windows Tolerates One Window of Lag; Each Lag Window Waits 100 ms of Host Time.
        let root = support::tempdir()?;
        let defaults = support::settings();
        let settings = HostSettings {
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

        // The Failed Grant Ends the Execution Interrupted With One Stream Failure, Which Is Its Reason.
        let outcome = execution.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.clone());
        let Some(Attribution::NotEvaluated(outcome)) = outcome else {
            panic!("Fall Behind a Window: Expected an Unevaluated Outcome, Found {outcome:?}");
        };
        assert_eq!(Termination::try_from(outcome.termination)?, Termination::Interrupted, "Fall Behind a Window: {}", outcome.reason);
        let failures = execution
            .evidence
            .iter()
            .filter(|entry| entry.kind == "streams:failure")
            .map(|entry| String::from_utf8_lossy(&entry.payload).into_owned())
            .collect::<Vec<_>>();
        assert_eq!(failures, std::slice::from_ref(&outcome.reason), "Fall Behind a Window: Report One Stream Failure");

        // It Names the Lagging Stream Alone and the Lag Setting That Resolves It.
        let streams = ["execution.boundary", "execution.windows.window.host", "execution.windows.window.setting", "execution.overflow.window.overflow"];
        assert!(outcome.reason.contains("execution.lag.window.lag"), "Fall Behind a Window: Name the Stream: {}", outcome.reason);
        assert!(
            streams.iter().all(|stream| !outcome.reason.contains(stream)),
            "Fall Behind a Window: Name Only the Lagging Stream: {}",
            outcome.reason
        );
        assert!(outcome.reason.contains("interval_lag_windows"), "Fall Behind a Window: Name the Lag Setting: {}", outcome.reason);
        host.shutdown().await?;
        Ok(())
    }))
}
