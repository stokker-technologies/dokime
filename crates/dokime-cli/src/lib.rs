/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod cli;
mod commands;
mod connection;
mod error;
mod input;

/* === Re-Exports === */
pub use cli::Cli;
pub use error::{Error, Result};

// Shared entrypoints for the standalone executable and Cargo extension.
pub use commands::{serve, version};

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
