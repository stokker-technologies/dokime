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
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::evidence::notes_server::Notes for Services {
    async fn get(
        &self, request: tonic::Request<v1::evidence::NotesGetRequest>,
    ) -> Result<tonic::Response<v1::evidence::NotesGetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn set(
        &self, request: tonic::Request<v1::evidence::NotesSetRequest>,
    ) -> Result<tonic::Response<v1::evidence::NotesSetResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn list(
        &self, request: tonic::Request<v1::evidence::NotesListRequest>,
    ) -> Result<tonic::Response<v1::evidence::NotesListResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::evidence::NotesWatchResponse>;
    async fn watch(
        &self, request: tonic::Request<v1::evidence::NotesWatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn create(
        &self, request: tonic::Request<v1::evidence::NotesCreateRequest>,
    ) -> Result<tonic::Response<v1::evidence::NotesCreateResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::evidence_notes_create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
