/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Runtime Creation Of Domain Values.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn now_utc() -> dokime_domain::model::time::Timestamp {
    dokime_domain::model::time::Timestamp::from_system_time(std::time::SystemTime::now())
}

/* === Private Functions === */

/* === Tests === */
