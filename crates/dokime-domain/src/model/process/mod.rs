/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A system process a step runs. The SDK serves it inside the plugin process,
// so its working directory, execution log, records and produced artifacts are
// the calling execution's own: called, it runs to its exit; submitted, its
// instance cancels it with SIGTERM, a grace period and a kill on Unix, and
// TerminateProcess on Windows.

/* === Modules === */

pub mod wire;

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
