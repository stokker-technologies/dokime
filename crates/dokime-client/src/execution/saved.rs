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
use super::{TestExecutionHandle, invalid, validate_checkpoint};
use crate::{Result, contracts};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl TestExecutionHandle {
    pub async fn context_at(
        &self, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint,
    ) -> Result<dokime_domain::model::history::wire::CheckpointContext> {
        validate_checkpoint(&self.id, checkpoint)?;
        self.client.verify(false).await?;
        let context = self.client.call(contracts::history::GetCheckpointContext { checkpoint: Some(checkpoint.clone()) }).await?;
        if context.checkpoint.as_ref() != Some(checkpoint) {
            return Err(invalid("history returned a different checkpoint"));
        }
        Ok(context)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
