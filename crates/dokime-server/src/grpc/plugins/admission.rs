/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bound verified reads independently from serialized installation mutations.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tonic::Status;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Admission {
    reads: Arc<Semaphore>,
    mutations: Arc<Semaphore>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Admission {
    pub(super) fn new() -> Self {
        Self {
            reads: Arc::new(Semaphore::new(1)),
            mutations: Arc::new(Semaphore::new(1)),
        }
    }

    pub(super) fn read(&self) -> Result<OwnedSemaphorePermit, Status> {
        self.reads.clone().try_acquire_owned().map_err(|_| Status::resource_exhausted("plugin verification is busy"))
    }

    pub(super) fn mutation(&self) -> Result<OwnedSemaphorePermit, Status> {
        self.mutations.clone().try_acquire_owned().map_err(|_| Status::resource_exhausted("plugin management is busy"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
