/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin installation, operations and admission ownership.

mod plugin_admission;
mod plugins;

use super::*;

#[tokio::test]
async fn plugin_lifecycle() {
    Box::pin(plugins::plugin_install_identity()).await;
    Box::pin(plugins::full_control_plugin_operations()).await;
    Box::pin(plugins::invalid_uploads_mutate()).await;
    Box::pin(plugins::listener_shutdown_admission()).await;
    Box::pin(plugins::result_budget_atomicity()).await;
    Box::pin(plugins::bounded_installed_metadata()).await;
    Box::pin(plugin_admission::cancelled_admission_ownership()).await;
}
