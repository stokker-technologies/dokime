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
impl dokime_protocol::client::evidence::notes_server::Notes for Services {
    async fn get(
        &self, request: tonic::Request<dokime_protocol::client::evidence::NotesGetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::NotesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<dokime_protocol::client::evidence::NotesSetRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::NotesSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<dokime_protocol::client::evidence::NotesListRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::NotesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<dokime_protocol::client::evidence::NotesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<dokime_protocol::client::evidence::NotesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<dokime_protocol::client::evidence::NotesCreateRequest>,
    ) -> Result<tonic::Response<dokime_protocol::client::evidence::NotesCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
