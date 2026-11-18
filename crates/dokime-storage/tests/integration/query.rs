/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Step Boundary Queries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports
use super::fixture;
use dokime_storage::{Error, Evidence, Result, Store};

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
fn step_boundaries_span_the_first_start_and_last_end_across_pages() -> Result<()> {
    // Save Repeated Attempts and Unrelated Step Markers
    let root = tempfile::tempdir()?;
    let store = Store::open(root.path())?;
    store.create(&fixture::execution())?;
    let mut positions = Vec::new();
    for (step, kind) in [
        ("other", "step:start"),
        ("measure", "step:start"),
        ("measure", "record:reading"),
        ("measure", "step:end"),
        ("measure", "step:start"),
        ("measure", "record:reading"),
        ("other", "step:end"),
        ("measure", "step:end"),
        ("measure", "record:reading"),
        ("open", "step:start"),
        ("open", "record:reading"),
        ("unstarted", "step:end"),
    ] {
        let evidence = Evidence {
            step: step.into(),
            kind: kind.into(),
            ..Default::default()
        };
        positions.push(store.append("execution", evidence)?);
    }
    let snapshot = store.read("execution")?;
    let frontier = store.cut("execution")?.frontier;

    // Preserve Strict Membership and Inclusive Paged Upper Bounds
    for limit in [NonZeroUsize::MIN, NonZeroUsize::new(3).unwrap()] {
        assert_eq!(store.step_range("execution", "measure", frontier, limit)?, (positions[1], positions[7] - 1));
        assert_eq!(store.step_range("execution", "open", frontier, limit)?, (positions[9], frontier));
        assert_eq!(store.step_range("execution", "", frontier, limit)?, (0, frontier));
        for step in ["missing", "unstarted"] {
            assert!(matches!(store.step_range("execution", step, frontier, limit), Err(Error::Invalid(_))));
        }
    }
    for (index, position) in positions.iter().copied().enumerate() {
        assert_eq!(snapshot.step_contains("measure", position), (2..7).contains(&index));
        assert_eq!(snapshot.step_contains("open", position), index > 9);
        assert!(snapshot.step_contains("", position));
        assert!(!snapshot.step_contains("missing", position));
        assert!(!snapshot.step_contains("unstarted", position));
    }

    // Retained Pages Exclude Later Ends and Still Reject Not-Yet-Started Steps
    assert_eq!(store.step_range("execution", "measure", positions[2], NonZeroUsize::MIN)?, (positions[1], positions[2]));
    assert!(store.step_range("execution", "open", positions[8], NonZeroUsize::MIN).is_err());
    Ok(())
}
