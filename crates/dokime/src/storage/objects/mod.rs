/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod annotations;
mod artifacts;
pub mod case;
pub mod cycle;
pub mod execution;
mod metadata;
pub mod presentation;
pub mod suite;
mod templates;

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
#[harmos::catalog]
pub enum Objects {
    Case(case::Case),
    Command(super::requests::Command),
    Cycle(cycle::Cycle),
    Execution(execution::Execution),
    Request(super::requests::Request),
    Suite(suite::Suite),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
