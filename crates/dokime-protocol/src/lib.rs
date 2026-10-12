/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[cfg(feature = "export")]
mod error;
pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/dokime.v1.rs"));
}

/* === Re-Exports === */
#[cfg(feature = "export")]
pub use error::{Error, Result};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
pub const API_VERSION: &str = "0.6.0";

pub const API_PACKAGE: &str = "dokime.v1";
pub const DISCOVERY_METHODS: [&str; 3] = [
    "/dokime.v1.Backend/GetIdentity",
    "/dokime.v1.Backend/GetHealth",
    "/dokime.v1.Backend/GetCapabilities",
];

pub const DEV_PROTOCOL: u32 = 4;

pub const READINESS_PROTOCOL: u64 = 3;

pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/dokime.v1.bin"));

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn compatible_api(package: &str, version: &str) -> bool {
    fn release(version: &str) -> Option<(&str, &str)> {
        let parts: Vec<_> = version.split('.').collect();
        if parts.len() != 3
            || parts.iter().any(|part| {
                part.is_empty()
                    || !part.bytes().all(|byte| byte.is_ascii_digit())
                    || (part.len() > 1 && part.starts_with('0'))
            })
        {
            return None;
        }

        Some((parts[0], parts[1]))
    }

    package == API_PACKAGE
        && release(version).is_some_and(|version| Some(version) == release(API_VERSION))
}

/* === Private Functions === */

/* === Tests === */
