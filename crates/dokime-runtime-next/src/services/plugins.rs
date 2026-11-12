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
impl v1::plugins_server::Plugins for Services {
    async fn get_installation(&self, request: tonic::Request<v1::PluginsGetInstallationRequest>) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.GetInstallation: Not Yet Served by the New Host"))
    }
    type RunGetInstallationStream = dokime_operations_next::client::Output<v1::PluginsGetInstallationRunOutput>;
    async fn run_get_installation(&self, request: tonic::Request<tonic::Streaming<v1::PluginsGetInstallationRunInput>>) -> Result<tonic::Response<Self::RunGetInstallationStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunGetInstallation: Not Yet Served by the New Host"))
    }
    async fn install(&self, request: tonic::Request<tonic::Streaming<v1::PluginsInstallInput>>) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.Install: Not Yet Served by the New Host"))
    }
    type RunInstallStream = dokime_operations_next::client::Output<v1::PluginsInstallRunOutput>;
    async fn run_install(&self, request: tonic::Request<tonic::Streaming<v1::PluginsInstallRunInput>>) -> Result<tonic::Response<Self::RunInstallStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunInstall: Not Yet Served by the New Host"))
    }
    async fn list_admitted_plugins(&self, request: tonic::Request<v1::PluginsListAdmittedPluginsRequest>) -> Result<tonic::Response<v1::AdmittedPluginPage>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_list_admitted_plugins::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListAdmittedPluginsStream = dokime_operations_next::client::Output<v1::PluginsListAdmittedPluginsRunOutput>;
    async fn run_list_admitted_plugins(&self, request: tonic::Request<tonic::Streaming<v1::PluginsListAdmittedPluginsRunInput>>) -> Result<tonic::Response<Self::RunListAdmittedPluginsStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::plugins_list_admitted_plugins_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::plugins_list_admitted_plugins_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::plugins_list_admitted_plugins_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::PluginsListAdmittedPluginsRunOutput { value: Some(v1::plugins_list_admitted_plugins_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::plugins_list_admitted_plugins::handle(&context, request).await },
        ).await
    }
    async fn list_installations(&self, request: tonic::Request<v1::PluginsListInstallationsRequest>) -> Result<tonic::Response<v1::InstallationPage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.ListInstallations: Not Yet Served by the New Host"))
    }
    type RunListInstallationsStream = dokime_operations_next::client::Output<v1::PluginsListInstallationsRunOutput>;
    async fn run_list_installations(&self, request: tonic::Request<tonic::Streaming<v1::PluginsListInstallationsRunInput>>) -> Result<tonic::Response<Self::RunListInstallationsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunListInstallations: Not Yet Served by the New Host"))
    }
    async fn reload(&self, request: tonic::Request<v1::PluginsReloadRequest>) -> Result<tonic::Response<v1::ClientEmpty>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_reload::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunReloadStream = dokime_operations_next::client::Output<v1::PluginsReloadRunOutput>;
    async fn run_reload(&self, request: tonic::Request<tonic::Streaming<v1::PluginsReloadRunInput>>) -> Result<tonic::Response<Self::RunReloadStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::plugins_reload_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::plugins_reload_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::plugins_reload_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::PluginsReloadRunOutput { value: Some(v1::plugins_reload_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::plugins_reload::handle(&context, request).await },
        ).await
    }
    async fn remove(&self, request: tonic::Request<v1::RemovePlugin>) -> Result<tonic::Response<v1::RemovePluginResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_remove::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRemoveStream = dokime_operations_next::client::Output<v1::PluginsRemoveRunOutput>;
    async fn run_remove(&self, request: tonic::Request<tonic::Streaming<v1::PluginsRemoveRunInput>>) -> Result<tonic::Response<Self::RunRemoveStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::plugins_remove_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::plugins_remove_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::plugins_remove_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::PluginsRemoveRunOutput { value: Some(v1::plugins_remove_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::plugins_remove::handle(&context, request).await },
        ).await
    }
    async fn set_enabled(&self, request: tonic::Request<v1::PluginsSetEnabledRequest>) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.SetEnabled: Not Yet Served by the New Host"))
    }
    type RunSetEnabledStream = dokime_operations_next::client::Output<v1::PluginsSetEnabledRunOutput>;
    async fn run_set_enabled(&self, request: tonic::Request<tonic::Streaming<v1::PluginsSetEnabledRunInput>>) -> Result<tonic::Response<Self::RunSetEnabledStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunSetEnabled: Not Yet Served by the New Host"))
    }
    async fn list(&self, request: tonic::Request<v1::ListPlugins>) -> Result<tonic::Response<v1::PluginEntries>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_list::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListStream = dokime_operations_next::client::Output<v1::PluginsListRunOutput>;
    async fn run_list(&self, request: tonic::Request<tonic::Streaming<v1::PluginsListRunInput>>) -> Result<tonic::Response<Self::RunListStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::plugins_list_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::plugins_list_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::plugins_list_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::PluginsListRunOutput { value: Some(v1::plugins_list_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::plugins_list::handle(&context, request).await },
        ).await
    }
    async fn add(&self, request: tonic::Request<v1::AddRemotePlugin>) -> Result<tonic::Response<v1::Plugin>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_add::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunAddStream = dokime_operations_next::client::Output<v1::PluginsAddRunOutput>;
    async fn run_add(&self, request: tonic::Request<tonic::Streaming<v1::PluginsAddRunInput>>) -> Result<tonic::Response<Self::RunAddStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::plugins_add_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::plugins_add_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::plugins_add_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::PluginsAddRunOutput { value: Some(v1::plugins_add_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::plugins_add::handle(&context, request).await },
        ).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
