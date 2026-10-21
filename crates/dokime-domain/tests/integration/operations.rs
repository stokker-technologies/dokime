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
use harmos_domain::operations::OperationDeclaration;

// Local Imports
use dokime_domain::operations::plugins::{Consent, Remove, Stage};

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
// host never cuts staging short; consent stays ordinary and removal is a mutation.
#[test]
fn slow_installs_complete() {
    assert!(Stage::POLICY.timeout >= Duration::from_secs(3_600));
    assert!(Stage::POLICY.attempt_timeout >= Duration::from_secs(3_600));
    assert_eq!(Consent::POLICY.timeout, Duration::from_secs(10));
    assert_eq!(Remove::POLICY.timeout, Duration::from_secs(120));
}
