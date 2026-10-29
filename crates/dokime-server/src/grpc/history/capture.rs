/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod wire;

use crate::grpc::transcode::Transcode;

use std::sync::Arc;

use dokime::harmos::{Context, Response};
use tonic::Status;

use dokime::app::LiveRuntime;
use dokime::operations::history;
use dokime::operations::hosting::{Host, Serve, Served};

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(history::GetCheckpointDocument),
}

impl Serve<history::GetCheckpointDocument> for Resources {
    fn serve<'a>(
        &'a self, mut request: history::GetCheckpointDocument, _: Context<'a, history::GetCheckpointDocument>,
    ) -> Served<'a, dokime::domain::model::history::wire::CheckpointDocument> {
        Box::pin(async move {
            let checkpoint = super::wire::checkpoint(request.checkpoint.take())?;
            let document = dokime::app::live_runtime(&self.runtime)
                .await?
                .execution_checkpoint_document(checkpoint)
                .await
                .map_err(super::wire::checkpoint_failure)?;
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

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<history::GetCheckpointDocument>>(Host::<history::GetCheckpointDocument>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

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
