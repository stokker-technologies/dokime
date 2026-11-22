/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Writer Admission and Durable Intake Handoff

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

// Third Party Imports
use harmos_protocol::model::Receipt;
use harmos_runtime::storage::StoredObject;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

// Local Imports
use crate::{Error, Result};
use dokime_storage::persisted::shapes::{Execution, State};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Gate per Admitted Execution, Shared by Its Native Evidence/Preview Writers.
#[derive(Clone)]
pub struct Gate {
    execution: String,
    closed: Arc<AtomicBool>,
    writers: Arc<RwLock<()>>,
}

pub struct Permit {
    _guard: OwnedRwLockReadGuard<()>,
    execution: String,
}

pub struct Closed {
    pub(super) execution: String,
    _guard: OwnedRwLockWriteGuard<()>,
}

/// Supplied by the Intake Owner After Sampling, Cleanup/Captures and Accepted
/// Batches Have Settled. The Receipt Covers Every Accepted Batch for This Owner.
pub struct DrainedIntake {
    pub execution: String,
    pub receipt: Receipt,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Gate {
    pub fn new(execution: &Execution) -> Result<Self> {
        execution.validate().map_err(|source| Error::Invalid(format!("Admit Execution Writers: {source}")))?;
        Ok(Self { execution: execution.key.clone(), closed: Arc::new(AtomicBool::new(execution.state != State::Open)), writers: Arc::new(RwLock::new(())) })
    }

    /// Retain the Permit Until the Accepted Write Reaches Its Durable Completion.
    pub async fn enter(&self) -> Result<Permit> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Precondition("Append Execution Evidence: Select an Open Execution".into()));
        }
        let guard = self.writers.clone().read_owned().await;
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Precondition("Append Execution Evidence: Select an Open Execution".into()));
        }
        Ok(Permit { _guard: guard, execution: self.execution.clone() })
    }

    /// Close Admission and Join Every Writer Already Holding a Permit.
    pub async fn close(&self) -> Closed {
        self.closed.store(true, Ordering::Release);
        let guard = self.writers.clone().write_owned().await;
        Closed { execution: self.execution.clone(), _guard: guard }
    }
}

impl Permit {
    pub(super) fn require(&self, execution: &Execution) -> Result<()> {
        if self.execution != execution.key {
            return Err(Error::Permission("Append Execution Evidence: Enter Admission for This Execution Owner".into()));
        }
        Ok(())
    }
}

impl Closed {
    pub fn require(&self, execution: &Execution) -> Result<()> {
        if self.execution != execution.key {
            return Err(Error::Permission("Settle Execution: Close Admission for This Execution Owner".into()));
        }
        Ok(())
    }
}

impl DrainedIntake {
    pub fn require(&self, closed: &Closed, execution: &Execution) -> Result<()> {
        closed.require(execution)?;
        if self.execution != execution.key || !self.receipt.completed || self.receipt.durable_at.is_none() {
            return Err(Error::Precondition("Close Execution: Drain Every Accepted Batch to Durable Completion for This Owner".into()));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
