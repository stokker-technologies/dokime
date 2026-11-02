/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation policies bound each operation by what it does, not by one default.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use harmos::operations::OperationDeclaration;

// Local Imports
use dokime_operations::plugins::{Install, Remove};

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

// A slow machine may take minutes to install and reload a debug plugin, so the
// host bounds installation as one transfer and removal,
// which deletes the bundle's files, has the heavy-mutation deadline.
#[dokime_tracing::trace_test]
#[test]
fn slow_installs_complete() {
    assert!(Install::POLICY.timeout >= Duration::from_secs(3_600));
    assert!(Install::POLICY.attempt_timeout >= Duration::from_secs(3_600));
    assert_eq!(Remove::POLICY.timeout, Duration::from_secs(120));
}
