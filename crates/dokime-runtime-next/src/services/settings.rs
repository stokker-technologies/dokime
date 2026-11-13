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
    async fn get(
        &self, request: tonic::Request<v1::SettingsGetRequest>,
    ) -> Result<tonic::Response<v1::SettingsGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<v1::SettingsSetRequest>,
    ) -> Result<tonic::Response<v1::SettingsSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn schema(
        &self, request: tonic::Request<v1::SettingsSchemaRequest>,
    ) -> Result<tonic::Response<v1::SettingsSchemaResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::settings_schema::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
