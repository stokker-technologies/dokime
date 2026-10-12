/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod wire;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use prost::Message;
use tonic::{Request, Response, Status};
// Local Imports
use super::value;
use dokime_app::{
    RuntimeHandle,
    capabilities::installed_plugins::{PluginCredentialSecretUpdate, PluginSettingsUpdate},
    config::{Density, ThemeMode},
};
use dokime_protocol::v1::{self, settings_server::Settings as SettingsService};
use wire::{encode_frame, encode_plugin_settings, identifier, plugin_error};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl SettingsService for Service {
    async fn read_storage_policy(
        &self,
        _: Request<v1::ReadStoragePolicyRequest>,
    ) -> Result<Response<v1::StoragePolicyView>, Status> {
        self.runtime
            .storage_policy()
            .await
            .map(|view| Response::new(policy_view(view)))
            .map_err(super::execution::status)
    }

    async fn update_storage_policy(
        &self,
        request: Request<v1::UpdateStoragePolicyRequest>,
    ) -> Result<Response<v1::StoragePolicyView>, Status> {
        let request = request.into_inner();
        let policy = request
            .policy
            .ok_or_else(|| Status::invalid_argument("policy is required"))?;
        self.runtime
            .update_storage_policy(
                request.expected_revision,
                dokime_app::config::StoragePolicy {
                    save_interval_ms: policy.save_interval_ms,
                    compaction_interval_ms: policy.compaction_interval_ms,
                    local_retention_ms: policy.local_retention_ms,
                },
            )
            .await
            .map(|view| Response::new(policy_view(view)))
            .map_err(super::execution::status)
    }

    async fn read_application_settings(
        &self,
        _: Request<v1::ReadApplicationSettingsRequest>,
    ) -> Result<Response<v1::ApplicationSettings>, Status> {
        Ok(Response::new(application_settings(
            self.runtime.application_settings(),
        )))
    }

    async fn update_appearance(
        &self,
        request: Request<v1::UpdateAppearanceRequest>,
    ) -> Result<Response<v1::ApplicationSettings>, Status> {
        let appearance = request
            .into_inner()
            .appearance
            .ok_or_else(|| Status::invalid_argument("appearance is required"))?;
        let appearance = dokime_app::config::Appearance {
            theme_mode: match appearance.theme_mode() {
                v1::ThemeMode::Unspecified | v1::ThemeMode::System => ThemeMode::System,
                v1::ThemeMode::Light => ThemeMode::Light,
                v1::ThemeMode::Dark => ThemeMode::Dark,
            },
            density: match appearance.density() {
                v1::Density::Unspecified | v1::Density::Comfortable => Density::Comfortable,
                v1::Density::Compact => Density::Compact,
                v1::Density::Touch => Density::Touch,
            },
            theme_family: appearance.theme_family,
        };
        self.runtime
            .update_appearance(appearance)
            .await
            .map(|view| Response::new(application_settings(view)))
            .map_err(super::execution::status)
    }

    async fn update_data_root(
        &self,
        request: Request<v1::UpdateDataRootRequest>,
    ) -> Result<Response<v1::ApplicationSettings>, Status> {
        let data_root = request.into_inner().data_root.map(std::path::PathBuf::from);
        self.runtime
            .update_data_root(data_root)
            .await
            .map(|view| Response::new(application_settings(view)))
            .map_err(super::execution::status)
    }

    async fn list_storage_sources(
        &self,
        _request: Request<v1::ListStorageSourcesRequest>,
    ) -> Result<Response<v1::StorageSources>, Status> {
        self.storage_sources_reply().await
    }

    async fn reorder_storage_sources(
        &self,
        request: Request<v1::ReorderStorageSourcesRequest>,
    ) -> Result<Response<v1::StorageSources>, Status> {
        self.runtime
            .reorder_storage_sources(request.into_inner().identifiers)
            .await
            .map_err(super::execution::status)?;
        self.storage_sources_reply().await
    }

    async fn set_storage_source_enabled(
        &self,
        request: Request<v1::SetStorageSourceEnabledRequest>,
    ) -> Result<Response<v1::StorageSources>, Status> {
        let request = request.into_inner();
        self.runtime
            .set_storage_source_enabled(&request.identifier, request.enabled)
            .await
            .map_err(super::execution::status)?;
        self.storage_sources_reply().await
    }

    async fn get_settings(
        &self,
        _request: Request<v1::GetSettingsRequest>,
    ) -> Result<Response<v1::SettingsFrame>, Status> {
        let frame = encode_frame(self.runtime.settings_frame().await, self.max_message_bytes)?;
        Ok(Response::new(frame))
    }

    async fn get_plugin_settings(
        &self,
        request: Request<v1::GetPluginSettingsRequest>,
    ) -> Result<Response<v1::PluginRuntimeSettings>, Status> {
        let plugin_id = identifier(request.into_inner().plugin_id, "plugin_id")?;
        let settings = self
            .runtime
            .plugin_settings(&plugin_id)
            .await
            .map_err(plugin_error)?;
        Ok(Response::new(encode_plugin_settings(
            settings,
            self.max_message_bytes,
        )?))
    }

    async fn update_plugin_settings(
        &self,
        request: Request<v1::UpdatePluginSettingsRequest>,
    ) -> Result<Response<v1::PluginRuntimeSettings>, Status> {
        let request = request.into_inner();
        let plugin_id = identifier(request.plugin_id, "plugin_id")?;
        let values = value::decode_object(request.values)?;
        let settings = self
            .runtime
            .save_plugin_settings(&plugin_id, PluginSettingsUpdate { values })
            .await
            .map_err(plugin_error)?;
        Ok(Response::new(encode_plugin_settings(
            settings,
            self.max_message_bytes,
        )?))
    }

    async fn update_plugin_credential(
        &self,
        request: Request<v1::UpdatePluginCredentialRequest>,
    ) -> Result<Response<v1::PluginRuntimeSettings>, Status> {
        let request = request.into_inner();
        let plugin_id = identifier(request.plugin_id, "plugin_id")?;
        let credential_id = wire::bounded_text(request.credential_id, "credential_id")?;
        let secret = wire::bounded_text(request.secret, "secret")?;
        let settings = self
            .runtime
            .save_plugin_credential_secret(
                &plugin_id,
                &credential_id,
                PluginCredentialSecretUpdate { secret },
            )
            .await
            .map_err(plugin_error)?;
        Ok(Response::new(encode_plugin_settings(
            settings,
            self.max_message_bytes,
        )?))
    }
}

impl Service {
    async fn storage_sources_reply(&self) -> Result<Response<v1::StorageSources>, Status> {
        let sources = self
            .runtime
            .storage_sources()
            .await
            .map_err(super::execution::status)?;
        let response = v1::StorageSources {
            sources: sources
                .into_iter()
                .map(|source| v1::StorageSource {
                    identifier: source.identifier,
                    adapter: source.adapter,
                    priority: source.priority,
                    enabled: source.enabled,
                })
                .collect(),
        };
        if response.encoded_len() > self.max_message_bytes.min(4 * 1024 * 1024) {
            return Err(Status::resource_exhausted(
                "storage sources exceed configured message budget",
            ));
        }
        Ok(Response::new(response))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn application_settings(
    view: dokime_app::config::ApplicationSettingsView,
) -> v1::ApplicationSettings {
    let path = |path: std::path::PathBuf| path.display().to_string();
    let appearance = v1::AppearancePreference {
        theme_mode: match view.appearance.theme_mode {
            ThemeMode::System => v1::ThemeMode::System,
            ThemeMode::Light => v1::ThemeMode::Light,
            ThemeMode::Dark => v1::ThemeMode::Dark,
        } as i32,
        theme_family: view.appearance.theme_family,
        density: match view.appearance.density {
            Density::Comfortable => v1::Density::Comfortable,
            Density::Compact => v1::Density::Compact,
            Density::Touch => v1::Density::Touch,
        } as i32,
    };
    v1::ApplicationSettings {
        appearance: Some(appearance),
        application_root: path(view.application_root),
        configured_data_root: view.configured_data_root.map(path),
        default_data_root: path(view.default_data_root),
        effective_data_root: path(view.effective_data_root),
        restart_required: view.restart_required,
    }
}

fn policy_view(view: dokime_app::config::StoragePolicyView) -> v1::StoragePolicyView {
    let encode = |policy: dokime_app::config::StoragePolicy| v1::StoragePolicy {
        save_interval_ms: policy.save_interval_ms,
        compaction_interval_ms: policy.compaction_interval_ms,
        local_retention_ms: policy.local_retention_ms,
    };
    v1::StoragePolicyView {
        revision: view.revision,
        configured: Some(encode(view.configured)),
        effective: Some(encode(view.effective)),
        restart_required: view.restart_required,
    }
}

/* === Tests === */
