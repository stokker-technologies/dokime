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
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use harmos::{
    Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use dokime_domain::operations::history;
use dokime_runtime::RuntimeHandle;

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
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut history::GetCheckpointDocument,
        _: &mut (),
    ) -> Result<Response<dokime_domain::model::history::wire::CheckpointDocument>, Status> {
        let checkpoint = super::wire::checkpoint(request.checkpoint.take())?;
        let document = self
            .resources
            .runtime
            .execution_checkpoint_document(checkpoint)
            .await
            .map_err(super::wire::checkpoint_failure)?;
        let project =
            || -> harmos::Result<dokime_domain::model::history::wire::CheckpointDocument> {
                let response = dokime_domain::model::history::wire::CheckpointDocument {
                    checkpoint: Some(
                        super::wire::checkpoint_wire(document.checkpoint)
                            .map_err(original_error)?,
                    ),
                    document: Some(
                        wire::execution_document(document.document).map_err(original_error)?,
                    ),
                    acquisition_finished: document.acquisition_finished,
                };
                if response.encoded_len() > self.resources.maximum {
                    return Err(exhausted());
                }
                Ok(response)
            };
        project()
            .map(Into::into)
            .map_err(|source| super::wire::checkpoint_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Get>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum: maximum.min(4 * 1024 * 1024),
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn original_error(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    harmos::Error::backend(
        "project original checkpoint execution document",
        harmos::BackendFailure::Corrupt,
        source,
    )
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint execution document",
    }
}

/* === Tests === */
