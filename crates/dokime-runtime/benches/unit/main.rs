/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Unit Benchmarks: One Kernel, No I/O

/* === Modules === */
mod evidence;
mod transactions;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use criterion::{criterion_group, criterion_main};

// Local Imports

/* === Constants === */

/* === Macros === */
criterion_group!(benches, evidence::benchmark, transactions::benchmark);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
