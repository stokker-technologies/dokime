/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes Resource Service

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
impl dokime_protocol::host::evidence::notes_server::Notes for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::host::evidence::NotesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::evidence::NotesGetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Get", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::notes::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::host::evidence::NotesSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::evidence::NotesSetResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Set", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::notes::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::host::evidence::NotesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::evidence::NotesListResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/List", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::notes::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations::host::Output<dokime_protocol::host::evidence::NotesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::host::evidence::NotesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Watch", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::notes::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::host::evidence::NotesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::host::evidence::NotesCreateResponse>, tonic::Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Create", harmos_protocol::model::Callers::Clients)
            .map_err(dokime_operations::host::status)?;
        let reply = dokime_operations::host::evidence::notes::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
