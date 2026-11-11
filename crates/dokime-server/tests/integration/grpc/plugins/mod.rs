/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin installation, operations and admission ownership.

/* === Modules === */

mod lifecycle;
mod plugin_admission;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn plugin_lifecycle() {
    Box::pin(lifecycle::plugin_install_identity()).await;
    Box::pin(lifecycle::full_control_plugin_operations()).await;
    Box::pin(lifecycle::invalid_uploads_mutate()).await;
    Box::pin(lifecycle::listener_shutdown_admission()).await;
    Box::pin(lifecycle::result_budget_atomicity()).await;
    Box::pin(lifecycle::bounded_installed_metadata()).await;
    Box::pin(plugin_admission::cancelled_admission_ownership()).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn plugin_install_idle_deadlines() {
    lifecycle::plugin_install_idle_deadlines().await;
}
