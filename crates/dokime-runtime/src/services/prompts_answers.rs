/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompts Answers Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

use dokime_protocol::host::prompts::AnswersCreateRequest;
use dokime_protocol::host::prompts::AnswersCreateResponse;
use dokime_protocol::host::prompts::answers_server::Answers;
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
impl Answers for Services {
    async fn create(&self, request: Request<AnswersCreateRequest>) -> Result<Response<AnswersCreateResponse>, Status> {
        dokime_operations::authorize("dokime.host.v1.prompts.Answers/Create", Callers::Clients)
            .map_err(dokime_operations::caller_status)?;
        let reply = dokime_operations::host::prompts::answers::create::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations::host::status)?;
        Ok(Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
