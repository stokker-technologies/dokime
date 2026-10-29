/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CLI Story Configuration Uses The Production Bootstrap

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

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

pub fn resolve(root: &Path) -> dokime::config::Config {
    dokime::config::ApplicationOptions {
        application_root: Some(root.to_owned()),
        ..Default::default()
    }
    .resolve()
    .unwrap()
}

/* === Private Functions === */

/* === Tests === */
