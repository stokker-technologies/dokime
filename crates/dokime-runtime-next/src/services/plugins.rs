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
    async fn get_installation(
        &self, request: tonic::Request<v1::PluginsGetInstallationRequest>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.GetInstallation: Not Yet Served by the New Host"))
    }
    type RunGetInstallationStream = dokime_operations_next::client::Output<v1::PluginsGetInstallationRunOutput>;
    async fn run_get_installation(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsGetInstallationRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetInstallationStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunGetInstallation: Not Yet Served by the New Host"))
    }
    async fn install(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsInstallInput>>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.Install: Not Yet Served by the New Host"))
    }
    type RunInstallStream = dokime_operations_next::client::Output<v1::PluginsInstallRunOutput>;
    async fn run_install(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsInstallRunInput>>,
    ) -> Result<tonic::Response<Self::RunInstallStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunInstall: Not Yet Served by the New Host"))
    }
    async fn list_admitted_plugins(
        &self, request: tonic::Request<v1::PluginsListAdmittedPluginsRequest>,
    ) -> Result<tonic::Response<v1::AdmittedPluginPage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.ListAdmittedPlugins: Not Yet Served by the New Host"))
    }
    type RunListAdmittedPluginsStream = dokime_operations_next::client::Output<v1::PluginsListAdmittedPluginsRunOutput>;
    async fn run_list_admitted_plugins(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsListAdmittedPluginsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListAdmittedPluginsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunListAdmittedPlugins: Not Yet Served by the New Host"))
    }
    async fn list_installations(
        &self, request: tonic::Request<v1::PluginsListInstallationsRequest>,
    ) -> Result<tonic::Response<v1::InstallationPage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.ListInstallations: Not Yet Served by the New Host"))
    }
    type RunListInstallationsStream = dokime_operations_next::client::Output<v1::PluginsListInstallationsRunOutput>;
    async fn run_list_installations(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsListInstallationsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListInstallationsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunListInstallations: Not Yet Served by the New Host"))
    }
    async fn reload(&self, request: tonic::Request<v1::PluginsReloadRequest>) -> Result<tonic::Response<v1::ClientEmpty>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.Reload: Not Yet Served by the New Host"))
    }
    type RunReloadStream = dokime_operations_next::client::Output<v1::PluginsReloadRunOutput>;
    async fn run_reload(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsReloadRunInput>>,
    ) -> Result<tonic::Response<Self::RunReloadStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunReload: Not Yet Served by the New Host"))
    }
    async fn remove(&self, request: tonic::Request<v1::RemovePlugin>) -> Result<tonic::Response<v1::RemovePluginResponse>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.Remove: Not Yet Served by the New Host"))
    }
    type RunRemoveStream = dokime_operations_next::client::Output<v1::PluginsRemoveRunOutput>;
    async fn run_remove(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsRemoveRunInput>>,
    ) -> Result<tonic::Response<Self::RunRemoveStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunRemove: Not Yet Served by the New Host"))
    }
    async fn set_enabled(
        &self, request: tonic::Request<v1::PluginsSetEnabledRequest>,
    ) -> Result<tonic::Response<v1::PluginInstallation>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.SetEnabled: Not Yet Served by the New Host"))
    }
    type RunSetEnabledStream = dokime_operations_next::client::Output<v1::PluginsSetEnabledRunOutput>;
    async fn run_set_enabled(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsSetEnabledRunInput>>,
    ) -> Result<tonic::Response<Self::RunSetEnabledStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunSetEnabled: Not Yet Served by the New Host"))
    }
    async fn list(&self, request: tonic::Request<v1::ListPlugins>) -> Result<tonic::Response<v1::PluginEntries>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListStream = dokime_operations_next::client::Output<v1::PluginsListRunOutput>;
    async fn run_list(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsListRunInput>>,
    ) -> Result<tonic::Response<Self::RunListStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunList: Not Yet Served by the New Host"))
    }
    async fn add(&self, request: tonic::Request<v1::AddRemotePlugin>) -> Result<tonic::Response<v1::Plugin>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_add::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunAddStream = dokime_operations_next::client::Output<v1::PluginsAddRunOutput>;
    async fn run_add(
        &self, request: tonic::Request<tonic::Streaming<v1::PluginsAddRunInput>>,
    ) -> Result<tonic::Response<Self::RunAddStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Plugins.RunAdd: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
