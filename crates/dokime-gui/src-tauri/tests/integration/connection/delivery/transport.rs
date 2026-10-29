/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Controlled Native Content Stream Fixture

use std::sync::Arc;

use sha2::{Digest, Sha256};
use tonic::{Request, Response, Status};

use super::{FixtureFuture, Gate, HttpFuture, HttpRequest, HttpResponse, Service, wire_codec};
use dokime::client::contracts::{Endpoint, content};
use dokime::domain::model::content::wire as model;

type Envelope = dokime::harmos::grpc::OutputEnvelope<model::ArtifactFrame, ()>;

#[derive(Clone, Default)]
pub struct DownloadConfig {
    pub calls: usize,
    pub corrupt: bool,
    pub gate: Option<Arc<Gate>>,
}

#[derive(Clone)]
pub struct Content(pub Service);

struct Download(Service);

impl tonic::server::NamedService for Content {
    const NAME: &'static str = "dokime.v1.Content";
}

impl tonic::codegen::Service<HttpRequest> for Content {
    type Response = HttpResponse;
    type Error = std::convert::Infallible;
    type Future = HttpFuture;

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                content::DownloadArtifact::PATH => {
                    wire_codec::<content::DownloadArtifact, dokime::harmos::operations::Response<Envelope>>()
                        .server()
                        .server_streaming(Download(service), request)
                        .await
                }
                _ => Status::unimplemented("unused test route").into_http(),
            };
            Ok(response)
        })
    }
}

impl tonic::server::ServerStreamingService<dokime::harmos::operations::StreamItem<content::DownloadArtifact>> for Download {
    type Response = dokime::harmos::operations::Response<Envelope>;
    type ResponseStream = tokio_stream::wrappers::ReceiverStream<Result<Self::Response, Status>>;
    type Future = FixtureFuture<Response<Self::ResponseStream>>;

    fn call(&mut self, request: Request<dokime::harmos::operations::StreamItem<content::DownloadArtifact>>) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let request = request.into_inner().into_request().map_err(|error| Status::data_loss(error.to_string()))?.0;
            let settings = {
                let mut settings = service.download.lock().unwrap();
                settings.calls += 1;
                settings.clone()
            };
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            tokio::spawn(async move {
                let bytes = bytes();
                let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
                let metadata = model::ArtifactMetadata {
                    artifact_id: request.artifact_id,
                    upload_id: "fixture".into(),
                    scope: None,
                    filename: "evidence.bin".into(),
                    content_type: Some("application/octet-stream".into()),
                    size_bytes: bytes.len() as u64,
                    sha256: digest.clone(),
                    state: model::ArtifactState::Complete,
                    created_at: None,
                };
                let frame =
                    |payload| dokime::harmos::operations::Response::new(Envelope::Item(model::ArtifactFrame { payload: Some(payload) }));
                if sender.send(Ok(frame(model::ArtifactFramePayload::Metadata(metadata)))).await.is_err() {
                    return;
                }
                for (index, chunk) in bytes.chunks(8192).enumerate() {
                    let chunk = model::ArtifactChunk {
                        offset: (index * 8192) as u64,
                        data: chunk.to_vec(),
                    };
                    if sender.send(Ok(frame(model::ArtifactFramePayload::Chunk(chunk)))).await.is_err() {
                        return;
                    }
                }
                if let Some(gate) = settings.gate {
                    gate.entered.notify_one();
                    tokio::select! {
                        permit = gate.release.acquire() => { permit.unwrap().forget(); }
                        _ = sender.closed() => return,
                    }
                }
                let complete = model::ArtifactCompletion {
                    size_bytes: bytes.len() as u64,
                    sha256: if settings.corrupt { "00".repeat(32) } else { digest },
                };
                if sender.send(Ok(frame(model::ArtifactFramePayload::Complete(complete)))).await.is_err() {
                    return;
                }
                let _ = sender.send(Ok(dokime::harmos::operations::Response::new(Envelope::Response(())))).await;
            });
            Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(receiver)))
        })
    }
}

pub fn bytes() -> Vec<u8> {
    (0..16_391).map(|index| (index % 251) as u8).collect()
}
