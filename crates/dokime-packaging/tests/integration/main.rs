/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// In-process packaging stories, one module per packaging domain.

/* === Modules === */

pub mod build;

#[path = "../../src/bundle/mod.rs"]
pub mod bundle;

#[path = "bundle.rs"]
mod bundle_stories;
pub mod installer;
mod release;

#[path = "../../src/process/mod.rs"]
pub mod process;

#[cfg(windows)]
mod resources;

/* === Re-Exports === */

pub use dokime_packaging as error;
pub use dokime_packaging::Error;

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
