/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod browse;
mod capture;
mod construction;
mod context;
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
pub(super) use wire::checkpoint;
pub(crate) use wire::{traceability, traceability_memory};

/* === Imports === */

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
