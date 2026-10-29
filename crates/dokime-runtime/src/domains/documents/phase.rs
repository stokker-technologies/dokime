/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) use crate::transactions::documents::AddExecutionDocumentParts;

use crate::{RuntimeHandle, runtime::Error, unavailable_rejection};

impl RuntimeHandle {
    pub(crate) async fn commit_execution_document_parts(&self, update: AddExecutionDocumentParts) -> Result<(), Error> {
        self.commit(update).await.map_err(|error| unavailable_rejection(error.to_string()))?;

        Ok(())
    }
}
