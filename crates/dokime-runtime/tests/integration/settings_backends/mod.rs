/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings, identity, installed-plugin settings, branding, themes and secret transfer persist and reopen through the real backends.

/* === Modules === */
pub(crate) mod config_application;
mod distribution;
pub(crate) mod identity;
pub(crate) mod installed_plugins;
pub(crate) mod runtime_settings;
pub(crate) mod secrets_transfer;

/* === Re-Exports === */

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn settings_backends() {
    runtime_settings::live_status_answered().await;
    runtime_settings::dispatcher_alone_probe().await;
    runtime_settings::discovered_settings_until().await;
    runtime_settings::resolves_one_record().await;
    runtime_settings::shared_secret_identifiers().await;
    runtime_settings::source_settings_absent().await;
    runtime_settings::application_preferences_reopen_and_reject_stale_or_invalid_updates().await;
    config_application::explicit_root_local();
    config_application::data_root_reopens().await;
    identity::existing_identity_republishing().await;
    identity::legacy_files_ignored().await;
    installed_plugins::settings::boot_health_transaction().await;
    installed_plugins::settings::install_definition_harmos().await;
    installed_plugins::settings::artifact_registration_visible().await;
    installed_plugins::settings::composed_plugin_process().await;
    installed_plugins::brand::organization_overlay();
    installed_plugins::brand::organization_validation();
    installed_plugins::brand::installer_sidebar_overlay();
    installed_plugins::brand::installer_sidebar_validation();
    installed_plugins::brand::generated_organization_stylesheet_matches_runtime_resolution();
    installed_plugins::brand::organization_toml_types();
    installed_plugins::brand::app_brand_label();
    installed_plugins::brand::missing_logo_failing();
    installed_plugins::brand::runtime_appearance_brand().await;
    installed_plugins::theme::local_woff2_uris();
    installed_plugins::theme::missing_local_refused();
    installed_plugins::theme::parent_directory_refused();
    installed_plugins::theme::remote_absolute_refused();
    installed_plugins::theme::unsupported_font_refused();
    #[cfg(unix)]
    installed_plugins::theme::symlink_font_refused();
    installed_plugins::theme::compiled_bundle_filesystem();
    secrets_transfer::secret_metadata_rejects_old_name_field();
    secrets_transfer::saved_file_roundtrip().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn settings_backends_concurrent() {
    identity::concurrent_creators_identity().await;
}
