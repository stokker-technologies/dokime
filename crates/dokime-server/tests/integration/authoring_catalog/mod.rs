/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Authoring, catalog schemas and document rows.

/* === Modules === */

mod authoring;
mod catalog;
mod documents;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

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
