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

use dokime_protocol::host::SettingsGetRequest;
use dokime_protocol::host::SettingsGetResponse;
use dokime_protocol::host::SettingsSchemaRequest;
use dokime_protocol::host::SettingsSchemaResponse;
use dokime_protocol::host::SettingsSetRequest;
use dokime_protocol::host::SettingsSetResponse;
use dokime_protocol::host::settings_server::Settings;
use harmos_protocol::model::Callers;
use tonic::Request;
use tonic::Response;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl Settings for Services {
    async fn get(&self, request: Request<SettingsGetRequest>) -> Result<Response<SettingsGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Settings/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::settings::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, request: Request<SettingsSetRequest>) -> Result<Response<SettingsSetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Settings/Set", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::settings::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn schema(&self, request: Request<SettingsSchemaRequest>) -> Result<Response<SettingsSchemaResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.Settings/Schema", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::settings::schema::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
