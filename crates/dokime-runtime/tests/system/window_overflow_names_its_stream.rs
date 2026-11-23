/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Window Whose Rows Overflow the Shared Publication Pool Interrupts Its Execution and Names Its Stream

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, error::Error};

// Third Party Imports

// Local Imports
use super::support;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::model::Termination;
use dokime_runtime::Host;

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
fn window_overflow_names_its_stream() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // One 10 s Window at 1 MHz Answers More Rows Than the Library's Shared Publication Pool Holds.
        let root = support::tempdir()?;
        let host = Host::open(root.path(), support::settings(), support::service_settings()).await?;
        let entry = host.add_managed("boundary", &support::boundary_fixture()?, &[]).await?;
        let origin = host.materialize(&entry, "execution.execution.overflow", HashMap::new()).await?;
        let cycle = host.start_cycle(vec![origin]).await?.wait().await?;
        let execution = &cycle.executions[0];

        // The Execution Is Interrupted, and Its Stream Failure Names the Overflowing Stream.
        let outcome = execution.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.clone());
        let Some(Attribution::NotEvaluated(outcome)) = outcome else {
            panic!("Overflow a Window: Expected an Unevaluated Outcome, Found {outcome:?}");
        };
        assert_eq!(Termination::try_from(outcome.termination)?, Termination::Interrupted, "Overflow a Window: {}", outcome.reason);
        let failures = execution
            .evidence
            .iter()
            .filter(|entry| entry.kind == "streams:failure")
            .map(|entry| String::from_utf8_lossy(&entry.payload).into_owned())
            .collect::<Vec<_>>();
        assert!(
            failures.iter().any(|failure| failure.contains("execution.window.overflow")),
            "Overflow a Window: Name the Stream in Its Failure: {failures:?}; Outcome: {}",
            outcome.reason
        );
        host.shutdown().await?;
        Ok(())
    }))
}
