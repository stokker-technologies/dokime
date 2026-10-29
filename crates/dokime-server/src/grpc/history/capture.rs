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
use dokime::harmos::{Context, RegistrationExt, Response};
use tonic::Status;

// Local Imports
use dokime::RuntimeHandle;
use dokime::operations::history;
use dokime::operations::hosting::{Host, Serve, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<history::GetCheckpointDocument> for Resources {
    fn serve<'a>(
        &'a self, mut request: history::GetCheckpointDocument, _: Context<'a, history::GetCheckpointDocument>,
    ) -> Served<'a, dokime::domain::model::history::wire::CheckpointDocument> {
        Box::pin(async move {
            let checkpoint = super::wire::checkpoint(request.checkpoint.take())?;
            let document = self.runtime.execution_checkpoint_document(checkpoint).await.map_err(super::wire::checkpoint_failure)?;
            let response = document
                .try_map(|document| -> dokime::runtime::Result<_> {
                    let response = dokime::domain::model::history::wire::CheckpointDocument {
                        checkpoint: Some(super::wire::checkpoint_wire(document.checkpoint, self.maximum).map_err(original_error)?),
                        document: Some(wire::execution_document(document.document).map_err(original_error)?),
                        acquisition_finished: document.acquisition_finished,
                    };
                    Ok(response)
                })
                .map_err(super::wire::checkpoint_failure)?;
            if response.wire_len()? > self.maximum {
                return Err(Status::resource_exhausted("historical response exceeds transport limit"));
            }
            Ok::<_, Status>(Response::from_owner(response.owner()))
        })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<history::GetCheckpointDocument>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies).operation::<history::GetCheckpointDocument>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
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
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint execution document",
    }
}

/* === Tests === */
