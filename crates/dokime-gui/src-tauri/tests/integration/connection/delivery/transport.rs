/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Controlled Canonical Content Stream

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use sha2::{Digest, Sha256};
use tonic::{Request, Response, Status, codegen::BoxFuture};

// Local Imports
use super::{Gate, HttpRequest, HttpResponse, Service};
use dokime_sdk::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
pub struct DownloadConfig {
    pub calls: usize,
    pub corrupt: bool,
    pub gate: Option<Arc<Gate>>,
}

#[derive(Clone)]
pub struct Content(pub Service);

struct Download(Service);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl tonic::server::NamedService for Content {
    const NAME: &'static str = "dokime.v1.Content";
}

impl tonic::codegen::Service<HttpRequest> for Content {
    type Response = HttpResponse;
    type Error = std::convert::Infallible;
    type Future = BoxFuture<HttpResponse, Self::Error>;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                "/dokime.v1.Content/DownloadArtifact" => {
                    tonic::server::Grpc::new(tonic_prost::ProstCodec::default()).server_streaming(Download(service), request).await
                }
                _ => Status::unimplemented("Fixture Route Is Not Served").into_http(),
            };
            Ok(response)
        })
    }
}

impl tonic::server::ServerStreamingService<v1::ContentDownloadArtifactRequest> for Download {
    type Response = v1::ContentDownloadArtifactOutput;
    type ResponseStream = tokio_stream::wrappers::ReceiverStream<Result<Self::Response, Status>>;
    type Future = BoxFuture<Response<Self::ResponseStream>, Status>;

    fn call(&mut self, request: Request<v1::ContentDownloadArtifactRequest>) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let settings = {
                let mut settings = service.download.lock().unwrap();
                settings.calls += 1;
                settings.clone()
            };
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            tokio::spawn(async move {
                // Send Contiguous Data with a Separately Verifiable Completion
                let bytes = bytes();
                let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                let metadata = v1::ArtifactMetadata {
                    artifact_id: request.into_inner().artifact_id,
                    filename: "evidence.bin".into(),
                    size_bytes: bytes.len() as u64,
                    sha256: digest.clone(),
                    state: v1::ArtifactState::Complete as i32,
                    ..Default::default()
                };
                let frame = |kind| v1::ContentDownloadArtifactOutput {
                    value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                        payload: Some(v1::ArtifactFramePayload { kind: Some(kind) }),
                    })),
                };
                if sender.send(Ok(frame(v1::artifact_frame_payload::Kind::Metadata(metadata)))).await.is_err() {
                    return;
                }
                for (index, chunk) in bytes.chunks(8192).enumerate() {
                    let chunk = v1::ArtifactChunk {
                        offset: (index * 8192) as u64,
                        data: chunk.to_vec(),
                    };
                    if sender.send(Ok(frame(v1::artifact_frame_payload::Kind::Chunk(chunk)))).await.is_err() {
                        return;
                    }
                }

                // Hold Completion Until the Story Releases or Cancels the Stream
                if let Some(gate) = settings.gate {
                    gate.entered.notify_one();
                    tokio::select! {
                        permit = gate.release.acquire() => { permit.unwrap().forget(); }
                        _ = sender.closed() => return,
                    }
                }
                let complete = v1::ArtifactCompletion {
                    size_bytes: bytes.len() as u64,
                    sha256: if settings.corrupt { "00".repeat(32) } else { digest },
                };
                if sender.send(Ok(frame(v1::artifact_frame_payload::Kind::Complete(complete)))).await.is_err() {
                    return;
                }
                let _ = sender
                    .send(Ok(v1::ContentDownloadArtifactOutput {
                        value: Some(v1::content_download_artifact_output::Value::Response(Default::default())),
                    }))
                    .await;
            });
            Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(receiver)))
        })
    }
}

/* === Public Functions === */

pub fn bytes() -> Vec<u8> {
    (0..16_391).map(|index| (index % 251) as u8).collect()
}

/* === Private Functions === */

/* === Tests === */
