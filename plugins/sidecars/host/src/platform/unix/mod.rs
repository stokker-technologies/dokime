/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Unix Native Calls

/* === Modules === */

pub mod adapters;
pub mod bluetooth;
pub mod connectivity;
pub mod controls;
pub mod echo;
pub mod power;
pub mod wifi;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Unix Collections Need No COM Apartment Resource.
pub struct Lease;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Lease {
    pub fn acquire() -> Result<Self> {
        Ok(Self)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
