/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compose the Starter Plugin

/* === Modules === */

pub mod operations;
pub mod suites;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::plugin::Record;
use dokime::{Plugin, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ExamplePlugin;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn confirmation() -> Record<bool> {
    Record::new("confirmation", "Confirmation")
}

pub fn plugin() -> Result<Plugin> {
    let plugin = Plugin::of::<ExamplePlugin>(env!("CARGO_PKG_VERSION"))
        .record(confirmation())
        .suite(suites::capture::suite()?)
        .operation(operations::echo::operation());
    Ok(plugin)
}

/* === Private Functions === */

/* === Tests === */
