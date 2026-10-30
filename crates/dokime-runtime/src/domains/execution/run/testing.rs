/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Per-Execution Drain Invocation Counts For Lifecycle Integration Stories.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    sync::{LazyLock, Mutex},
};

// Third Party Imports

// Local Imports
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */
static COUNTS: LazyLock<Mutex<BTreeMap<TestExecutionId, BTreeMap<&'static str, usize>>>> = LazyLock::new(Default::default);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn record(execution: &TestExecutionId, stage: &'static str) {
    *COUNTS.lock().unwrap().entry(execution.clone()).or_default().entry(stage).or_default() += 1;
}

pub fn drain_counts(execution: &TestExecutionId) -> BTreeMap<&'static str, usize> {
    COUNTS.lock().unwrap().remove(execution).unwrap_or_default()
}

/* === Private Functions === */

/* === Tests === */
