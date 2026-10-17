/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(feature = "simulation")]

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::app;
use dokime_runtime::core::SessionId;
use dokime_runtime::state::RuntimeState;

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
fn declaration_boots_runtime() {
    let origin = RuntimeState::new(SessionId::new());

    app::simulate(origin, 42).expect("Dokime's catalogs are simulable");
}
