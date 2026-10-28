/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bound verified reads independently from serialized installation mutations.

use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tonic::Status;

pub(super) struct Admission {
    reads: Arc<Semaphore>,
    mutations: Arc<Semaphore>,
}

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
