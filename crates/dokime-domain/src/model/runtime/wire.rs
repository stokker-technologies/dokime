/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Identity {
    pub backend_id: String,
    pub epoch: String,
    pub display_name: String,
    pub product_version: String,
}

#[harmos_domain::message]
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Health {
    pub backend_id: String,
    pub epoch: String,
    pub readiness: Readiness,
    pub observed_at_seconds: i64,
    pub observed_at_nanoseconds: u32,
    pub message: String,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Readiness {
    Starting,
    Ready,
    Degraded,
    Stopping,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
