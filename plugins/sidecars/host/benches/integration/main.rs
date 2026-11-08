/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[allow(dead_code)]
#[path = "../../src/host/cpu/kernel.rs"]
mod kernel;

mod load;

/* === Re-Exports === */

pub use plugin_host::{Error, Result};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use criterion::{criterion_group, criterion_main};

// Local Imports

/* === Constants === */

/* === Macros === */

criterion_group!(benches, load::benchmark);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
