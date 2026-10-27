/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod wire;

/* === Re-Exports === */

/* === Imports === */

use crate::grpc::transcode::Transcode;

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use dokime::RuntimeHandle;
use dokime::operations::history;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct Get {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Get {
    type Declaration = history::GetCheckpointDocument;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut history::GetCheckpointDocument, _: &mut (),
    ) -> Result<Response<dokime::domain::model::history::wire::CheckpointDocument>, Status> {
        let checkpoint = super::wire::checkpoint(request.checkpoint.take())?;
        let document = self.resources.runtime.execution_checkpoint_document(checkpoint).await.map_err(super::wire::checkpoint_failure)?;
        let response = document
            .try_map(|document| -> dokime::runtime::Result<_> {
                let response = dokime::domain::model::history::wire::CheckpointDocument {
                    checkpoint: Some(super::wire::checkpoint_wire(document.checkpoint, self.resources.maximum).map_err(original_error)?),
                    document: Some(wire::execution_document(document.document).map_err(original_error)?),
                    acquisition_finished: document.acquisition_finished,
                };
                Ok(response)
            })
            .map_err(super::wire::checkpoint_failure)?;
        if response.wire_len()? > self.resources.maximum {
            return Err(Status::resource_exhausted("historical response exceeds transport limit"));
        }
        Ok(Response::from_owner(response.owner()))
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Get>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn original_error(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("project original checkpoint execution document", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted { resource: "canonical checkpoint execution document" }
}

/* === Tests === */
