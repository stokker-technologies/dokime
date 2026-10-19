/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History operations: browsing, reading and inspecting recorded executions.

/* === Modules === */

mod browse;
mod capture;
mod construction;
mod context;
mod cycles;
pub(super) mod document;
mod evidence;
mod inspection;
mod read;
mod records;
mod telemetry;
mod window;
mod wire;

/* === Re-Exports === */

pub(super) use inspection::register;
pub(crate) use wire::{traceability, traceability_memory};

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
