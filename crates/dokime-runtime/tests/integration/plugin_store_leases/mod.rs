/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The plugin store installs, activates and leases plugins end to end.

/* === Modules === */
pub(crate) mod activation;
pub(crate) mod leases;
pub(crate) mod store;

/* === Re-Exports === */
pub(crate) use crate::service_support::prelude::*;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

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
async fn plugin_store_leases() {
    activation::broken_configured_loud().await;
    store::direct_installation().await;
    store::replacement_preserves_bytes().await;
    store::collisions_preserve_installations().await;
    store::refused_result_preserved().await;
    store::unavailable_bytes_metadata().await;
    store::installed_survive_cleanup().await;
    #[cfg(unix)]
    store::failed_settings_bytes().await;
    store::removal_cleanup_retry().await;
    store::disabled_reserves_owners().await;
    store::bounded_pages_unknown().await;
    store::startup_reclaims_extraction().await;
    store::app_roots_independent().await;
    store::unauthorized_files_kept().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn plugin_store_leases_concurrent() {
    leases::failed_group_holds().await;
    leases::declared_streams_independent().await;
    leases::two_executions_rows().await;
    leases::two_holds_reading().await;
    leases::foreign_stream_refuses().await;
}
