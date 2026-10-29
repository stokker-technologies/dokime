/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authoring, catalog schemas and document rows.

mod authoring;
mod catalog;
mod documents;

use super::*;

#[tokio::test]
async fn authoring_catalog() {
    authoring::untyped_artifact_refused();
    Box::pin(authoring::declared_row_home()).await;
    Box::pin(catalog::schemas_exact_pages()).await;
    Box::pin(catalog::catalog_suite_extensions()).await;
    Box::pin(documents::current_document_rows()).await;
    Box::pin(documents::summary_complete_telemetry()).await;
    Box::pin(documents::duration_x_axis()).await;
}
