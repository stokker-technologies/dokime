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

use dokime_operations::host::Output;
use dokime_protocol::host::evidence::NotesCreateRequest;
use dokime_protocol::host::evidence::NotesCreateResponse;
use dokime_protocol::host::evidence::NotesGetRequest;
use dokime_protocol::host::evidence::NotesGetResponse;
use dokime_protocol::host::evidence::NotesListRequest;
use dokime_protocol::host::evidence::NotesListResponse;
use dokime_protocol::host::evidence::NotesSetRequest;
use dokime_protocol::host::evidence::NotesSetResponse;
use dokime_protocol::host::evidence::NotesWatchRequest;
use dokime_protocol::host::evidence::NotesWatchResponse;
use dokime_protocol::host::evidence::notes_server::Notes;
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
impl Notes for Services {
    async fn get(&self, request: Request<NotesGetRequest>) -> Result<Response<NotesGetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Get", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::notes::get::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn set(&self, request: Request<NotesSetRequest>) -> Result<Response<NotesSetResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Set", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::notes::set::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn list(&self, request: Request<NotesListRequest>) -> Result<Response<NotesListResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/List", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::notes::list::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    type WatchStream = Output<NotesWatchResponse>;
    async fn watch(&self, request: Request<NotesWatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Watch", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::notes::watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
    async fn create(&self, request: Request<NotesCreateRequest>) -> Result<Response<NotesCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.evidence.Notes/Create", Callers::Clients).map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::evidence::notes::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
