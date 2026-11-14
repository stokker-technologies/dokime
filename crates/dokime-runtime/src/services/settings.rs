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
impl dokime_protocol::host::settings_server::Settings for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::SettingsGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SettingsGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Settings/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::settings::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::SettingsSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SettingsSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Settings/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::settings::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn schema(
        &self, request: tonic::Request<dokime_protocol::host::SettingsSchemaRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::SettingsSchemaResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.Settings/Schema", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::settings::schema::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
