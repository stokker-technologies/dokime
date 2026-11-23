/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Intentional Interval Gaps Remain Execution Evidence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    error::Error,
    num::{NonZeroU32, NonZeroUsize},
    time::Duration,
};

// Third Party Imports
use serde_json::Value;

// Local Imports
use super::support;
use dokime_protocol::model::Termination;
use dokime_runtime::Host;
use dokime_storage::Store;

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
fn window_lag_recovers() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // Let the Fixture Advance Virtual Time Beyond a Window, Then Resume on the Next Timely Wake.
        let root = support::tempdir()?;
        let mut settings = support::settings();
        settings.placement.interval_window = Duration::from_millis(10);
        settings.placement.interval_lag = NonZeroU32::new(2).unwrap();
        let host = Host::open(root.path(), settings, support::service_settings()).await?;
        let entry = host.add_managed("boundary", &support::boundary_fixture()?, &[]).await?;
        let origin = host.materialize(&entry, "execution.execution.lag", HashMap::new()).await?;
        let cycle = host.start_cycle(vec![origin]).await?.wait().await?;
        let execution = &cycle.executions[0];

        // Intentional Holes Retain Their Extent Without Interrupting the Execution or Losing Later Samples.
        assert_eq!(Termination::try_from(execution.execution.termination)?, Termination::Completed);
        assert!(!execution.evidence.iter().any(|evidence| evidence.kind == "streams:failure"));
        let gaps = execution.evidence.iter().filter(|evidence| evidence.kind == "streams:gap").collect::<Vec<_>>();
        assert!(!gaps.is_empty());
        for evidence in gaps {
            let gap: Value = serde_json::from_slice(&evidence.payload)?;
            assert_eq!(gap["stream"], "execution.window.lag");
            let missed = gap["missed"].as_u64().unwrap();
            let start_ns = gap["start_ns"].as_i64().unwrap();
            let end_ns = gap["end_ns"].as_i64().unwrap();
            assert!(missed > 0);
            assert_eq!(end_ns - start_ns, i64::try_from(missed)? * 1_000_000);
            assert_eq!(evidence.source_ns, start_ns);
            assert!(execution.samples.iter().any(|sample| sample.stream == gap["stream"] && sample.source_ns >= end_ns));
            assert!(
                !execution.samples.iter().any(|sample| sample.stream == gap["stream"] && (start_ns..end_ns).contains(&sample.source_ns))
            );
        }
        host.shutdown().await?;
        drop(host);

        // Reopening the Root Preserves the Same Gap Evidence After Execution Settlement.
        let store = Store::open(root.path())?;
        let through = store.cut(&execution.execution.identifier)?.frontier;
        let saved = store.evidence_page(&execution.execution.identifier, 0, through, NonZeroUsize::new(1000).unwrap())?;
        assert_eq!(saved, execution.evidence);
        Ok(())
    }))
}
