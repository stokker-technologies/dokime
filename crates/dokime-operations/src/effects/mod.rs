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

#[harmos::operation(id = "dokime.execution.environment", response = Environment, policy = crate::ORDINARY, visibility = Public,
    resources = [crate::hosting::Host<ReadEnvironment>, harmos::operations::HostContext])]
#[derive(Clone, Default, PartialEq)]
pub struct ReadEnvironment {}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Environment {
    pub backend_pid: u32,
    pub in_step: bool,
}

#[harmos::operation(id = "dokime.execution.effect.prepare", response = Ticket, policy = crate::ORDINARY, visibility = Public,
    resources = [crate::hosting::Host<PrepareEffect>, harmos::operations::HostContext])]
#[derive(Clone, PartialEq)]
pub struct PrepareEffect {
    pub resource: String,
    pub inverse_operation: String,
    pub inverse: Vec<u8>,
    pub description: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Ticket {
    // Execution Prepares Get Unique Tickets; Standalone Calls Return An Empty Id.
    pub id: String,
}

#[harmos::operation(id = "dokime.execution.effect.applied", policy = crate::ORDINARY, visibility = Public,
    resources = [crate::hosting::Host<ConfirmEffect>, harmos::operations::HostContext])]
#[derive(Clone, PartialEq)]
pub struct ConfirmEffect {
    pub ticket: String,
    pub at: String,
    // This Call Rolled Back To Its Prior State; Earlier Pending Changes Stay Journaled.
    pub restored: bool,
}

// Every Registered Inverse Answers Only After The Native Change Has Settled.
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct EffectResult {
    pub at: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct StartEvidence {
    pub payload_json: String,
}

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ReadEnvironment(ReadEnvironment),
    PrepareEffect(PrepareEffect),
    ConfirmEffect(ConfirmEffect),
}

/* === Traits === */

/* === Implementations === */

#[cfg(feature = "runtime")]
mod runtime {
    crate::hosted!(super::ReadEnvironment, super::PrepareEffect, super::ConfirmEffect);
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
