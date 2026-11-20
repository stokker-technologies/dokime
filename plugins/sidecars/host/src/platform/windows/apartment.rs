/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Multithreaded Apartment the Host's COM and WinRT Probes Share

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::OnceLock;

// Third Party Imports
use windows::Win32::System::Com::CoIncrementMTAUsage;

// Local Imports
use dokime::{Error, Result};

/* === Constants === */

static RETAINED: OnceLock<std::result::Result<(), String>> = OnceLock::new();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Keep the Apartment Alive Until the Process Exits.
///
/// When its last thread leaves, COM unloads every in-process server, yet system worker threads
/// still run inside them and windows-rs keeps their activation factories cached, so either one
/// faults on unmapped code.
pub fn retain() -> Result<()> {
    // SAFETY: Incrementing the usage has no preconditions; the usage is deliberately never released.
    let retained = RETAINED.get_or_init(|| unsafe { CoIncrementMTAUsage() }.map(drop).map_err(|error| error.to_string()));
    retained.clone().map_err(|error| Error::Invalid(format!("Join the Host COM Apartment: {error}; Restart the Plugin")))
}

/* === Private Functions === */

/* === Tests === */
