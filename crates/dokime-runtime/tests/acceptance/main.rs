/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-process SDK stories: one behaviour of the runtime per story, with test
// hosts and in-process sidecars, grouped by SDK domain.

#![recursion_limit = "256"]

/* === Modules === */

#[path = "../support/engine/evidence_durability.rs"]
pub mod evidence_support;

mod evidence_durability;

#[cfg(feature = "fault-injection")]
mod cycle_restart;
mod engine;
mod services;

#[path = "../support/services/mod.rs"]
pub mod service_support;

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

/* === Private Functions === */

/* === Tests === */
