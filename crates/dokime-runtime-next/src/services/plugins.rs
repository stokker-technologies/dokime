/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugins Resource Service

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
impl dokime_protocol::client::plugins_server::Plugins for Services {
    async fn get_installation(
        &self, request: tonic::Request<dokime_protocol::client::PluginsGetInstallationRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::PluginInstallation>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_get_installation::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn install(
        &self, request: tonic::Request<tonic::Streaming<dokime_protocol::client::PluginsInstallInput>>,
    ) -> Result<tonic::Response<dokime_protocol::client::PluginInstallation>, tonic::Status> {
        use tokio_stream::StreamExt;
        let input = request.into_inner().map(|input| {
            input.map(|input| match input.value {
                Some(dokime_protocol::client::plugins_install_input::Value::Request(request)) => {
                    dokime_operations_next::client::plugins_install::Upload::Request(request)
                }
                Some(dokime_protocol::client::plugins_install_input::Value::Item(item)) => {
                    dokime_operations_next::client::plugins_install::Upload::Item(item)
                }
                None => dokime_operations_next::client::plugins_install::Upload::Cancel,
            })
        });
        let reply = dokime_operations_next::client::plugins_install::handle(&self.context, input)
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_admitted_plugins(
        &self, request: tonic::Request<dokime_protocol::client::PluginsListAdmittedPluginsRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::AdmittedPluginPage>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_list_admitted_plugins::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list_installations(
        &self, request: tonic::Request<dokime_protocol::client::PluginsListInstallationsRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::InstallationPage>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_list_installations::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn reload(
        &self, request: tonic::Request<dokime_protocol::client::PluginsReloadRequest>,
    ) -> Result<tonic::Response<v1::ClientEmpty>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_reload::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn remove(
        &self, request: tonic::Request<dokime_protocol::client::RemovePlugin>,
    ) -> Result<tonic::Response<dokime_protocol::client::RemovePluginResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_remove::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set_enabled(
        &self, request: tonic::Request<dokime_protocol::client::PluginsSetEnabledRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::PluginInstallation>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_set_enabled::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::ListPlugins>,
    ) -> Result<tonic::Response<dokime_protocol::client::PluginEntries>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn add(
        &self, request: tonic::Request<dokime_protocol::client::AddRemotePlugin>,
    ) -> Result<tonic::Response<dokime_protocol::client::Plugin>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_add::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
