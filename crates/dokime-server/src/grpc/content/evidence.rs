/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Deliver Saved Evidence Through The Shared Transfer Pipeline

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::Context;
use dokime::harmos::operations::Response;
use tonic::Status;

// Local Imports
use crate::grpc::{
    chunked::{self, Frames},
    error::detailed_error,
};
use dokime::app::LiveRuntime;
use dokime::domain::model::{
    content::wire::{ArtifactChunk, ArtifactCompletion},
    exports::{EvidenceExportFrame, EvidenceExportPayload, EvidenceSubject},
    identity::{TestCycleId, TestExecutionId},
};
use dokime::domains::exports::evidence::Error;
use dokime::operations::content::DownloadEvidence;
use dokime::operations::hosting::{Host, Serve, Served};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

struct ExportFrames {
    metadata: Response<EvidenceExportFrame>,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(DownloadEvidence),
}

/* === Traits === */

/* === Implementations === */

impl Serve<DownloadEvidence> for Resources {
    fn serve<'a>(&'a self, request: DownloadEvidence, ctx: Context<'a, DownloadEvidence>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async {
                let subject = request.subject.clone().ok_or_else(|| Status::invalid_argument("evidence subject is required"))?;
                let valid = match &subject {
                    EvidenceSubject::Execution(id) => TestExecutionId::from_uuid(id).map(|_| ()),
                    EvidenceSubject::Cycle(id) => TestCycleId::from_uuid(id).map(|_| ()),
                };
                valid.map_err(|_| Status::invalid_argument("evidence subject must contain a UUID"))?;
                let bundle =
                    dokime::app::live_runtime(&self.runtime).await?.export_evidence(subject).await.map_err(|error| failure(&error))?;
                let metadata = bundle.retained_metadata().map_err(|error| failure(&error))?;
                let size_bytes = metadata.size_bytes;
                let sha256 = Some(metadata.sha256.clone());
                let metadata = metadata
                    .try_map(|metadata| {
                        Ok::<_, dokime::harmos::storage::Error>(EvidenceExportFrame {
                            payload: Some(EvidenceExportPayload::Metadata(metadata)),
                        })
                    })
                    .map_err(|error| failure(&Error::Dependency(error)))?;
                let frames = ExportFrames {
                    metadata: Response::from_owner(metadata.owner()),
                };
                chunked::Download {
                    frames,
                    source: chunked::Source::Evidence(Box::new(bundle)),
                    size_bytes,
                    sha256,
                    noun: "evidence export",
                    read_failure: "saved evidence export could not be read",
                    chunk_bytes: super::download_chunk_bytes(&dokime::app::live_runtime(&self.runtime).await?, self.maximum)?,
                    message_limit: self.maximum,
                }
                .run(ctx.streams())
                .await?;
                Ok(().into())
            })
            .await
        })
    }
}

impl Frames for ExportFrames {
    type Frame = EvidenceExportFrame;
    fn metadata(&self) -> Self::Frame {
        (*self.metadata).clone()
    }
    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.metadata.clone())
    }
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        EvidenceExportFrame {
            payload: Some(EvidenceExportPayload::Chunk(ArtifactChunk { offset, data })),
        }
    }
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        EvidenceExportFrame {
            payload: Some(EvidenceExportPayload::Complete(ArtifactCompletion { size_bytes, sha256 })),
        }
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<DownloadEvidence>>(Host::<DownloadEvidence>::new(served));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

pub(in crate::grpc) fn failure(error: &Error) -> Status {
    let status = match error {
        Error::Busy | Error::TooLarge | Error::Dependency(dokime::harmos::storage::Error::ResourceExhausted { .. }) => {
            Status::resource_exhausted("evidence export capacity exceeded")
        }
        Error::Cancelled => Status::cancelled("evidence export cancelled"),
        Error::Missing { .. } => Status::not_found("saved evidence not found"),
        Error::Unsealed(_) => Status::failed_precondition("evidence must be durably sealed before export"),
        Error::Invalid(_) => Status::data_loss("saved evidence is invalid"),
        _ => Status::internal("evidence export failed"),
    };
    detailed_error(status, error)
}

/* === Private Functions === */

/* === Tests === */
