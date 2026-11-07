/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Data Contracts

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
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Supply {
    pub voltage_rms_v: f64,
    pub load_ohm: f64,
    pub frequency_hz: f64,
    pub phase_deg: f64,
    pub current_limit_a: f64,
}

#[dokime::message]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    pub elapsed_s: f64,
    pub voltage_v: f64,
    pub current_a: f64,
    pub voltage_rms_v: f64,
    pub current_rms_a: f64,
    pub frequency_hz: f64,
    pub phase_deg: f64,
    pub input_power_w: f64,
    pub dc_voltage_v: f64,
    pub dc_current_a: f64,
}
#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct Charger {
    pub device_id: u64,
    pub supply: Supply,
    pub power: Power,
    pub fault: Option<Fault>,
    pub log: Option<String>,
}

#[dokime::message]
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub reading: Reading,
    pub charger: Charger,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Restart {
    pub blocked: bool,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct EncodedSummary {
    pub json: Vec<u8>,
}

/* === Enums === */

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Power {
    Off,
    Running,
    Faulted,
}

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Overcurrent,
    PowerLoss,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
