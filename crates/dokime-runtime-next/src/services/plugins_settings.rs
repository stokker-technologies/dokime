/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugins Settings Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::client::plugins::settings_server::Settings for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::plugins::SettingsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::plugins::SettingsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_settings_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::client::plugins::SettingsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::plugins::SettingsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::plugins_settings_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
