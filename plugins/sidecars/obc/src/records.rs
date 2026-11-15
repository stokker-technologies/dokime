/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Record Messages the Charger Procedures Publish

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::plugin::{Context, Record};
use dokime::{Error, Message, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Charger Was Initialized for This Attempt.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Initialized {}

/// The Supply Was Energized, With a Note on Why.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Started {
    pub note: String,
}

/// Protection or a Power Loss Opened the Charger Supply.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Faulted {
    pub fault: String,
}

/// Power Was Restored and the Fault Latch Cleared.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Cleared {}

/// How Far the Measurement Had Got When the Operator Was Asked.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Checkpoint {
    pub sample_index: u64,
    pub prompt_type: String,
}

/// The Measurement and the Operator's Decision, Together.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Completed {
    pub sample_count: u64,
    pub prompt_action: String,
    pub operator_note: String,
    pub accepted: bool,
}

/// What Happened to the Supply as the Measuring Step Saw It.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct HoldEvent {
    pub event: String,
    pub elapsed_s: f64,
    pub voltage_rms_v: f64,
}

/// The Charger Was Left De-Energized, With the Procedure's Decision.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Stopped {
    pub accepted: bool,
    pub tripped_at_step: u64,
    pub cycles: u64,
}

/// What a Pulse Run Produced Before It Completed.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct RunCompleted {
    pub samples: u64,
}

/// The Operator's Confirmation of the Bench Readings.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Confirmation {
    pub accepted: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Decode the Last Record of a Type, Optionally Within One Step.
pub async fn last<T: Message>(ctx: &Context, record: &Record<T>, step: &str) -> Result<Option<T>> {
    let records = ctx.records(record.identifier(), step, 0).await?;
    records
        .last
        .map(|captured| <T as dokime::encoding::Message>::decode(captured.payload.as_slice()))
        .transpose()
        .map_err(|error| Error::Invalid(format!("Read {}: {error}", record.identifier())))
}

/* === Private Functions === */

/* === Tests === */
