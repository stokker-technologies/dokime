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
    pub kind: Radio,
    pub on: bool,
}

#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct AdapterState {
    pub id: String,
    pub enabled: bool,
}

/* === Enums === */

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Radio {
    Wifi,
    Bluetooth,
    Cellular,
}

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioPower {
    On,
    Off,
}

/* === Traits === */

/* === Implementations === */

impl Radio {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wifi => "wifi",
            Self::Bluetooth => "bluetooth",
            Self::Cellular => "cellular",
        }
    }
}

impl std::fmt::Display for Radio {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl RadioPower {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

impl std::fmt::Display for RadioPower {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
