/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings Resource Service

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
impl dokime_protocol::client::settings_server::Settings for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::SettingsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SettingsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::client::SettingsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SettingsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn schema(
        &self, request: tonic::Request<dokime_protocol::client::SettingsSchemaRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::SettingsSchemaResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_schema::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
