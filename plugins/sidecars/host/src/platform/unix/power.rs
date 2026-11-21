/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Power Profile Reads Through ACPI Platform Profile

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::{Error, Result};

/* === Constants === */

pub const PLATFORM_PROFILE: &str = "/sys/firmware/acpi/platform_profile";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The Active ACPI Platform Profile as Plan Identifier and Name.
pub fn active_plan() -> Result<(String, Option<String>)> {
    let profile = std::fs::read_to_string(PLATFORM_PROFILE).map_err(|error| {
        Error::Invalid(format!(
            "Read the Active Power Profile: {PLATFORM_PROFILE} Is Unavailable ({error}); Use Firmware With ACPI Platform Profiles"
        ))
    })?;
    let id = profile.trim().to_owned();
    Ok((id.clone(), Some(id)))
}

/* === Private Functions === */

/* === Tests === */
