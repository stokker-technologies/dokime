/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::message]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControlState {
    pub radios: Vec<RadioState>,
    pub adapters: Vec<AdapterState>,
    pub power_plan: Option<String>,
}

#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct RadioState {
    pub id: String,
    pub kind: String,
    pub on: bool,
}

#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct AdapterState {
    pub id: String,
    pub enabled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
