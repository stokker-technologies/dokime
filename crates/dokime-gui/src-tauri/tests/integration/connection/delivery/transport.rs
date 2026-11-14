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
use dokime_protocol::{client, v1};
use dokime_sdk::client::v1 as library;
use std::convert::Infallible;
use std::task::Context;
use std::task::Poll;
use tonic::server::Grpc;
use tonic::server::NamedService;
use tonic::server::UnaryService;

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

#[derive(Clone)]
pub struct Storage(pub Service);

struct Metadata(Service);
struct FileInformation;

struct Download(Service);

/* === Implementations === */

impl NamedService for Content {
    const NAME: &'static str = "dokime.client.v1.content.Artifacts";
}
impl NamedService for Storage {
    const NAME: &'static str = "dokime.client.v1.Files";
}

impl tonic::codegen::Service<HttpRequest> for Content {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = BoxFuture<HttpResponse, Self::Error>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            Ok(match request.uri().path() {
                "/dokime.client.v1.content.Artifacts/Get" => {
                    Grpc::new(tonic_prost::ProstCodec::default()).unary(Metadata(service), request).await
                }
                _ => Status::unimplemented("Read Artifact: Fixture Route Is Unavailable").into_http(),
            })
        })
    }
}

impl tonic::codegen::Service<HttpRequest> for Storage {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = BoxFuture<HttpResponse, Self::Error>;
    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            Ok(match request.uri().path() {
                "/dokime.client.v1.Files/Read" => Grpc::new(tonic_prost::ProstCodec::default()).unary(Download(service), request).await,
                _ => Status::unimplemented("Read File Bytes: Fixture Route Is Unavailable").into_http(),
            })
        })
    }
}

impl UnaryService<client::content::ArtifactsGetRequest> for Metadata {
    type Response = client::content::ArtifactsGetResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<client::content::ArtifactsGetRequest>) -> Self::Future {
        self.0.download.lock().unwrap().calls += 1;
        Box::pin(async move {
            let bytes = bytes();
            let digest = Sha256::digest(&bytes);
            let metadata = v1::ArtifactMetadata {
                artifact_id: request.into_inner().name,
                filename: "evidence.bin".into(),
                size_bytes: bytes.len() as u64,
                sha256: digest.iter().map(|byte| format!("{byte:02x}")).collect(),
                state: v1::ArtifactState::Complete as i32,
                ..Default::default()
            };
            let mut reply = client::content::ArtifactsGetResponse {
                metadata: Some(metadata),
                ..Default::default()
            };
            let file = reply.file.get_or_insert_default();
            file.length = bytes.len() as u64;
            file.sha256 = digest.to_vec();
            let metadata = file.metadata.get_or_insert_default();
            metadata.id = "fixture-artifact".into();
            metadata.media_type = "application/octet-stream".into();
            Ok(Response::new(reply))
        })
    }
}

impl UnaryService<client::FilesGetRequest> for FileInformation {
    type Response = client::FilesGetResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<client::FilesGetRequest>) -> Self::Future {
        Box::pin(async move {
            Ok(Response::new(client::FilesGetResponse {
                file: request.into_inner().file,
                chunk_bytes: 4096,
                message_bytes: 32768,
                resolved_at: Some(library::ReadCut {
                    positions: vec![library::Position {
                        history: Some(library::History {
                            store: "fixture".into(),
                            epoch: "content".into(),
                        }),
                        sequence: 1,
                    }],
                    revisions: Vec::new(),
                }),
            }))
        })
    }
}

impl UnaryService<client::FilesReadRequest> for Download {
    type Response = client::FilesReadResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<client::FilesReadRequest>) -> Self::Future {
        let settings = self.0.download.lock().unwrap().clone();
        Box::pin(async move {
            let request = request.into_inner();
            let original = bytes();
            let begin = request.offset as usize;
            let end = begin.saturating_add(request.max_bytes as usize).min(original.len());
            if begin > end {
                return Err(Status::out_of_range("Read File Bytes: Invalid Offset"));
            }
            if let Some(gate) = settings.gate {
                gate.entered.notify_one();
                gate.release.acquire().await.unwrap().forget();
            }
            let mut data = original[begin..end].to_vec();
            if settings.corrupt && !data.is_empty() {
                data[0] ^= 1;
            }
            let mut reply = client::FilesReadResponse {
                data,
                total_length: original.len() as u64,
                sha256: Sha256::digest(&original).to_vec(),
                ..Default::default()
            };
            let position = reply.resolved_at.get_or_insert_default();
            let entry = library::Position {
                history: Some(library::History {
                    store: "fixture".into(),
                    epoch: "content".into(),
                }),
                sequence: 1,
            };
            position.positions.push(entry);
            Ok(Response::new(reply))
        })
    }
}

/* === Public Functions === */

pub fn bytes() -> Vec<u8> {
    (0..16_391).map(|index| (index % 251) as u8).collect()
}

/* === Private Functions === */

/* === Tests === */
