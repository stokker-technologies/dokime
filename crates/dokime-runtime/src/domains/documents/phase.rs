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
pub(crate) use crate::transactions::documents::AddExecutionDocumentParts;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{RuntimeHandle, runtime::Error, unavailable_rejection};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn commit_execution_document_parts(&self, update: AddExecutionDocumentParts) -> Result<(), Error> {
        self.commit(update).await.map_err(|error| unavailable_rejection(error.to_string()))?;

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
