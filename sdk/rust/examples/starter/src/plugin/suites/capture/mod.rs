/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Capture Suite

/* === Modules === */

pub mod confirmation;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::{Result, Suite};
use confirmation::ConfirmationTestTemplate;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct CaptureSuite;

/* === Enums === */

/* === Traits === */

/* === Implementations === */



/* === Public Functions === */

pub fn suite() -> Result<Suite> {
    Ok(Suite::of::<CaptureSuite>().template(ConfirmationTestTemplate::template()))
}

/* === Private Functions === */



/* === Tests === */


