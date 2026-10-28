/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) use crate::transactions::traceability::{RecordExecutionTraceability, UpdateExecutionPhase};

use crate::{RuntimeHandle, runtime::Error, unavailable_rejection};

impl RuntimeHandle {
    pub(crate) async fn commit_execution_phase(&self, update: UpdateExecutionPhase) -> Result<(), Error> {
        self.commit(update).await.map_err(|error| unavailable_rejection(error.to_string()))?;
        self.reconcile_sampling().await;
        Ok(())
    }

    pub(crate) async fn commit_execution_traceability(&self, update: RecordExecutionTraceability) -> Result<(), Error> {
        self.commit(update).await.map_err(|error| unavailable_rejection(error.to_string()))?;
        Ok(())
    }
}
