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
use dokime::plugin::{Context, DeclarationTypes, Plugin, Record};
use dokime::{Error, Message, Result, encoding};

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

/// Protection Opened the Charger Supply.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Faulted {
    pub fault: String,
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

/// The Charger Was Left De-Energized, With the Procedure's Decision.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct Stopped {
    pub accepted: bool,
    pub tripped_at_step: u64,
}

/// The Supply at the Start and End of a Steady-State Hold.
#[derive(dokime::Record)]
#[harmos(package = "obc")]
pub struct HoldEvent {
    pub event: String,
    pub elapsed_s: f64,
    pub voltage_rms_v: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Shared Charger Events Used by Several Templates.
pub fn catalog<C: DeclarationTypes>(plugin: Plugin<C>) -> Plugin<impl DeclarationTypes> {
    plugin.record(initialized()).record(started()).record(completed()).record(faulted()).record(stopped()).record(hold())
}
pub fn initialized() -> Record<Initialized> {
    Record::new("obc.records.charger.initialized", "Charger Initialized")
}
pub fn started() -> Record<Started> {
    Record::new("obc.records.charger.started", "Charger Energized")
}
pub fn completed() -> Record<Completed> {
    Record::new("obc.records.charger.completed", "Measurement Completed")
}
pub fn faulted() -> Record<Faulted> {
    Record::new("obc.records.charger.faulted", "Protection Tripped")
}
pub fn stopped() -> Record<Stopped> {
    Record::new("obc.records.charger.stopped", "Charger Disconnected")
}

pub fn hold() -> Record<HoldEvent> {
    Record::new("obc.records.charger.hold", "Supply Hold Event")
}

/// Decode the Last Record of a Type, Optionally Within One Step.
pub async fn last<T: Message>(ctx: &Context, record: &Record<T>, step: &str) -> Result<Option<T>> {
    let records = ctx.records(record.identifier(), step, 0).await?;
    records
        .last
        .map(|captured| <T as encoding::Message>::decode(captured.payload.as_slice()))
        .transpose()
        .map_err(|error| Error::Invalid(format!("Read {}: {error}", record.identifier())))
}

/* === Private Functions === */

/* === Tests === */
