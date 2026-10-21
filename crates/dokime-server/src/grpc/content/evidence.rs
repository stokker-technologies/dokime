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
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;

// Local Imports
use crate::grpc::{
    chunked::{self, Frames},
    error::detailed_error,
};
use dokime_domain::{
    model::{
        content::wire::{ArtifactChunk, ArtifactCompletion},
        exports::{EvidenceExportFrame, EvidenceExportPayload, EvidenceSubject},
        identity::{TestCycleId, TestExecutionId},
    },
    operations::content::DownloadEvidence,
};
use dokime_runtime::{RuntimeHandle, domains::exports::evidence::Error};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct Download {
    resources: Arc<Resources>,
}

struct ExportFrames(EvidenceExportFrame);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Download {
    type Declaration = DownloadEvidence;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<EvidenceExportFrame>;

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut DownloadEvidence, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let subject = request.subject.clone().ok_or_else(|| Status::invalid_argument("evidence subject is required"))?;
        let valid = match &subject {
            EvidenceSubject::Execution(id) => TestExecutionId::from_uuid(id).map(|_| ()),
            EvidenceSubject::Cycle(id) => TestCycleId::from_uuid(id).map(|_| ()),
        };
        valid.map_err(|_| Status::invalid_argument("evidence subject must contain a UUID"))?;
        let bundle = self.resources.runtime.export_evidence(subject).await.map_err(|error| failure(&error))?;
        let metadata = bundle.metadata().clone();
        let size_bytes = metadata.size_bytes;
        let sha256 = Some(metadata.sha256.clone());
        let frames = ExportFrames(EvidenceExportFrame { payload: Some(EvidenceExportPayload::Metadata(metadata)) });
        chunked::Download {
            frames,
            source: chunked::Source::Evidence(Box::new(bundle)),
            size_bytes,
            sha256,
            noun: "evidence export",
            read_failure: "saved evidence export could not be read",
            chunk_bytes: super::download_chunk_bytes(self.resources.maximum),
            message_limit: self.resources.maximum,
        }
        .run(output)
        .await?;
        Ok(().into())
    }
}

impl Frames for ExportFrames {
    type Frame = EvidenceExportFrame;
    fn metadata(&self) -> Self::Frame {
        self.0.clone()
    }
    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        EvidenceExportFrame { payload: Some(EvidenceExportPayload::Chunk(ArtifactChunk { offset, data })) }
    }
    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        EvidenceExportFrame { payload: Some(EvidenceExportPayload::Complete(ArtifactCompletion { size_bytes, sha256 })) }
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Download>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

pub(in crate::grpc) fn failure(error: &Error) -> Status {
    let status = match error {
        Error::Busy | Error::TooLarge => Status::resource_exhausted("evidence export capacity exceeded"),
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
