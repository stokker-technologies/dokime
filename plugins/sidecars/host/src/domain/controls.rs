/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Observable Host Control State: Radios, Network Adapters and the Active Power Plan

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fmt;

// Third Party Imports

// Local Imports
use dokime::{Error, Message, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Control State a Mutating Control Captures Before Its Write and Restores as Its Inverse.
#[derive(Message)]
#[dokime(package = "host")]
pub struct ControlState {
    pub radios: Vec<RadioState>,
    pub adapters: Vec<AdapterState>,
    pub power_plan: Option<String>,
}

/// One Radio, Its Kind as `Radio::as_str` Names It, and Whether It Is On.
#[derive(Message)]
#[dokime(package = "host")]
pub struct RadioState {
    pub id: String,
    pub kind: String,
    pub on: bool,
}

#[derive(Message)]
#[dokime(package = "host")]
pub struct AdapterState {
    pub id: String,
    pub enabled: bool,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Radio {
    Wifi,
    Bluetooth,
    Cellular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadioPower {
    On,
    Off,
}

/* === Traits === */

/* === Implementations === */

impl Radio {
    pub const ALL: [Self; 3] = [Self::Wifi, Self::Bluetooth, Self::Cellular];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wifi => "wifi",
            Self::Bluetooth => "bluetooth",
            Self::Cellular => "cellular",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|radio| radio.as_str() == value)
            .ok_or_else(|| Error::Invalid(format!("Select Radio: Choose wifi, bluetooth or cellular Instead of {value:?}")))
    }
}

impl fmt::Display for Radio {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    pub const fn from_on(on: bool) -> Self {
        if on { Self::On } else { Self::Off }
    }
}

impl fmt::Display for RadioPower {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl RadioState {
    pub fn radio(&self) -> Result<Radio> {
        Radio::parse(&self.kind)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
