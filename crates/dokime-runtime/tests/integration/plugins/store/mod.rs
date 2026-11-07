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
pub(crate) mod installation;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn plugin_store_leases() {
    activation::broken_configured_loud().await;
    installation::direct_installation().await;
    installation::replacement_preserves_bytes().await;
    installation::collisions_preserve_installations().await;
    installation::refused_result_preserved().await;
    installation::unavailable_bytes_metadata().await;
    installation::installed_survive_cleanup().await;
    #[cfg(unix)]
    installation::failed_settings_bytes().await;
    installation::removal_cleanup_retry().await;
    installation::disabled_reserves_owners().await;
    installation::bounded_pages_unknown().await;
    installation::startup_reclaims_extraction().await;
    installation::app_roots_independent().await;
    installation::unauthorized_files_kept().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn plugin_store_leases_concurrent() {
    leases::failed_group_holds().await;
    leases::declared_streams_independent().await;
    leases::two_executions_rows().await;
    leases::two_holds_reading().await;
    leases::foreign_stream_refuses().await;
}

/* === Tests === */
