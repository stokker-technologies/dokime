/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod authoring;
#[cfg(feature = "host")]
pub mod captured;

/* === Re-Exports === */
pub use authoring::*;
#[cfg(feature = "host")]
pub use captured::{ExecutionContext, ExecutionCycleContext};

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
