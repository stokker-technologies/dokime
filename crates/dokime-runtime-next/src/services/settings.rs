/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::settings_server::Settings for Services {
    async fn get_plugin_settings(
        &self, request: tonic::Request<v1::SettingsGetPluginSettingsRequest>,
    ) -> Result<tonic::Response<v1::PluginRuntimeSettings>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_get_plugin_settings::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetPluginSettingsStream = dokime_operations_next::client::Output<v1::SettingsGetPluginSettingsRunOutput>;
    async fn run_get_plugin_settings(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsGetPluginSettingsRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetPluginSettingsStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::settings_get_plugin_settings_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::settings_get_plugin_settings_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::settings_get_plugin_settings_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::SettingsGetPluginSettingsRunOutput {
                value: Some(v1::settings_get_plugin_settings_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::settings_get_plugin_settings::handle(&context, request).await },
        )
        .await
    }
    async fn get_settings(
        &self, request: tonic::Request<v1::SettingsGetSettingsRequest>,
    ) -> Result<tonic::Response<v1::SettingsFrame>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_get_settings::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetSettingsStream = dokime_operations_next::client::Output<v1::SettingsGetSettingsRunOutput>;
    async fn run_get_settings(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsGetSettingsRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetSettingsStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::settings_get_settings_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::settings_get_settings_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::settings_get_settings_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::SettingsGetSettingsRunOutput {
                value: Some(v1::settings_get_settings_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::settings_get_settings::handle(&context, request).await },
        )
        .await
    }
    async fn list_storage_sources(
        &self, request: tonic::Request<v1::SettingsListStorageSourcesRequest>,
    ) -> Result<tonic::Response<v1::StorageSources>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_list_storage_sources::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListStorageSourcesStream = dokime_operations_next::client::Output<v1::SettingsListStorageSourcesRunOutput>;
    async fn run_list_storage_sources(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsListStorageSourcesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListStorageSourcesStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::settings_list_storage_sources_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::settings_list_storage_sources_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::settings_list_storage_sources_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SettingsListStorageSourcesRunOutput { value: Some(v1::settings_list_storage_sources_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::settings_list_storage_sources::handle(&context, request).await },
        ).await
    }
    async fn read_application_settings(
        &self, request: tonic::Request<v1::SettingsReadApplicationSettingsRequest>,
    ) -> Result<tonic::Response<v1::SettingsSnapshot>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_read_application_settings::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunReadApplicationSettingsStream = dokime_operations_next::client::Output<v1::SettingsReadApplicationSettingsRunOutput>;
    async fn run_read_application_settings(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsReadApplicationSettingsRunInput>>,
    ) -> Result<tonic::Response<Self::RunReadApplicationSettingsStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::settings_read_application_settings_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::settings_read_application_settings_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::settings_read_application_settings_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::SettingsReadApplicationSettingsRunOutput {
                value: Some(v1::settings_read_application_settings_run_output::Value::Response(reply)),
            },
            |context, request| async move {
                dokime_operations_next::client::settings_read_application_settings::handle(&context, request).await
            },
        )
        .await
    }
    async fn reorder_storage_sources(
        &self, request: tonic::Request<v1::SettingsReorderStorageSourcesRequest>,
    ) -> Result<tonic::Response<v1::StorageSources>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_reorder_storage_sources::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunReorderStorageSourcesStream = dokime_operations_next::client::Output<v1::SettingsReorderStorageSourcesRunOutput>;
    async fn run_reorder_storage_sources(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsReorderStorageSourcesRunInput>>,
    ) -> Result<tonic::Response<Self::RunReorderStorageSourcesStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::settings_reorder_storage_sources_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::settings_reorder_storage_sources_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::settings_reorder_storage_sources_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SettingsReorderStorageSourcesRunOutput { value: Some(v1::settings_reorder_storage_sources_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::settings_reorder_storage_sources::handle(&context, request).await },
        ).await
    }
    async fn set_storage_source_enabled(
        &self, request: tonic::Request<v1::SettingsSetStorageSourceEnabledRequest>,
    ) -> Result<tonic::Response<v1::StorageSources>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_set_storage_source_enabled::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunSetStorageSourceEnabledStream = dokime_operations_next::client::Output<v1::SettingsSetStorageSourceEnabledRunOutput>;
    async fn run_set_storage_source_enabled(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsSetStorageSourceEnabledRunInput>>,
    ) -> Result<tonic::Response<Self::RunSetStorageSourceEnabledStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.mutation_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::settings_set_storage_source_enabled_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::settings_set_storage_source_enabled_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::settings_set_storage_source_enabled_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::SettingsSetStorageSourceEnabledRunOutput {
                value: Some(v1::settings_set_storage_source_enabled_run_output::Value::Response(reply)),
            },
            |context, request| async move {
                dokime_operations_next::client::settings_set_storage_source_enabled::handle(&context, request).await
            },
        )
        .await
    }
    async fn update_application_settings(
        &self, request: tonic::Request<v1::SettingsUpdateApplicationSettingsRequest>,
    ) -> Result<tonic::Response<v1::SettingsSnapshot>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_update_application_settings::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunUpdateApplicationSettingsStream = dokime_operations_next::client::Output<v1::SettingsUpdateApplicationSettingsRunOutput>;
    async fn run_update_application_settings(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsUpdateApplicationSettingsRunInput>>,
    ) -> Result<tonic::Response<Self::RunUpdateApplicationSettingsStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.mutation_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::settings_update_application_settings_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::settings_update_application_settings_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::settings_update_application_settings_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::SettingsUpdateApplicationSettingsRunOutput {
                value: Some(v1::settings_update_application_settings_run_output::Value::Response(reply)),
            },
            |context, request| async move {
                dokime_operations_next::client::settings_update_application_settings::handle(&context, request).await
            },
        )
        .await
    }
    async fn update_plugin_settings(
        &self, request: tonic::Request<v1::SettingsUpdatePluginSettingsRequest>,
    ) -> Result<tonic::Response<v1::PluginRuntimeSettings>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_update_plugin_settings::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunUpdatePluginSettingsStream = dokime_operations_next::client::Output<v1::SettingsUpdatePluginSettingsRunOutput>;
    async fn run_update_plugin_settings(
        &self, request: tonic::Request<tonic::Streaming<v1::SettingsUpdatePluginSettingsRunInput>>,
    ) -> Result<tonic::Response<Self::RunUpdatePluginSettingsStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::settings_update_plugin_settings_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::settings_update_plugin_settings_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::settings_update_plugin_settings_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SettingsUpdatePluginSettingsRunOutput { value: Some(v1::settings_update_plugin_settings_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::settings_update_plugin_settings::handle(&context, request).await },
        ).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
