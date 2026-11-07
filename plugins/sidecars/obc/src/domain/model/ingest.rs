/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Scalar and Wide Ingest Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::model::prompts::checkpoint::SteadyCheck;
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::stream(id = "obc.benchmarks.scalar", window = 4096, label = "Scalar Ingest")]
pub struct Scalar {
    pub value: f64,
}

#[dokime::stream(id = "obc.benchmarks.wide", window = 4096, label = "Wide Ingest")]
pub struct Wide {
    pub value: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
    pub g: f64,
    pub h: f64,
    pub i: f64,
    pub j: f64,
    pub k: f64,
    pub l: f64,
    pub m: f64,
    pub n: f64,
    pub o: f64,
}

#[dokime::stream(id = "obc.benchmarks.payload", window = 4096, label = "Payload Ingest")]
pub struct Payload {
    pub value: f64,
    pub data: String,
}

/* === Enums === */

#[dokime::catalog]
pub enum Streams {
    Scalar(Scalar),
    Wide(Wide),
    Payload(Payload),
}

#[dokime::catalog]
pub enum Prompts {
    SteadyCheck(SteadyCheck),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
