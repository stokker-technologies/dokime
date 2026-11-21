/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Telemetry Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use plugin_host::collectors::network::{Change, Pending};

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
fn the_initial_snapshot_keeps_events_already_seen_and_seeds_unknown_links() {
    let mut pending = Pending::new(BTreeMap::new());
    pending.push(Change::new("link_up", "nic".into(), None), Some(true));
    pending.seed(BTreeMap::from([("nic".into(), false), ("other".into(), false)]));
    pending.push(Change::new("link_up", "nic".into(), None), Some(true));
    pending.push(Change::new("link_down", "other".into(), None), Some(false));
    assert_eq!(pending.pop().unwrap().unwrap().adapter_id, "nic");
    assert!(pending.pop().unwrap().is_none());
    pending.push(Change::new("link_down", "nic".into(), None), Some(false));
    assert_eq!(pending.pop().unwrap().unwrap().adapter_id, "nic");
    assert!(pending.pop().unwrap().is_none());
}
