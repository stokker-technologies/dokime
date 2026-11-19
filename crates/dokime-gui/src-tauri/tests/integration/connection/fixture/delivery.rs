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
use std::{
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};

// Third Party Imports
use sha2::{Digest, Sha256};
use tonic::server::{Grpc, NamedService, UnaryService};
use tonic::{
    Request, Response, Status,
    codegen::{self, BoxFuture},
};
use tonic_prost::ProstCodec;

// Local Imports
use dokime_sdk::History;
use dokime_sdk::Position;
use dokime_sdk::ReadCut;

use super::{Gate, HttpRequest, HttpResponse, Service};
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::content::ArtifactsGetRequest;

use dokime_client_internal::ArtifactsGetResponse;
use dokime_sdk::ArtifactMetadata;
use dokime_sdk::ArtifactState;
use dokime_sdk::FilesGetResponse;
use dokime_sdk::FilesReadResponse;

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
    const NAME: &'static str = "dokime.host.v1.content.Artifacts";
}
impl NamedService for Storage {
    const NAME: &'static str = "dokime.host.v1.Files";
}

impl codegen::Service<HttpRequest> for Content {
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
                "/dokime.host.v1.content.Artifacts/Get" => Grpc::new(ProstCodec::default()).unary(Metadata(service), request).await,
                _ => Status::unimplemented("Read Artifact: Fixture Route Is Unavailable").into_http(),
            })
        })
    }
}

impl codegen::Service<HttpRequest> for Storage {
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
                "/dokime.host.v1.Files/Get" => Grpc::new(ProstCodec::default()).unary(FileInformation, request).await,
                "/dokime.host.v1.Files/Read" => Grpc::new(ProstCodec::default()).unary(Download(service), request).await,
                _ => Status::unimplemented("Read File Bytes: Fixture Route Is Unavailable").into_http(),
            })
        })
    }
}

impl UnaryService<ArtifactsGetRequest> for Metadata {
    type Response = ArtifactsGetResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<ArtifactsGetRequest>) -> Self::Future {
        self.0.download.lock().unwrap().calls += 1;
        Box::pin(async move {
            let bytes = bytes();
            let digest = Sha256::digest(&bytes);
            let metadata = ArtifactMetadata {
                artifact_id: request.into_inner().name,
                filename: "evidence.bin".into(),
                size_bytes: bytes.len() as u64,
                sha256: digest.iter().map(|byte| format!("{byte:02x}")).collect(),
                state: ArtifactState::Complete as i32,
                ..Default::default()
            };
            let mut reply = ArtifactsGetResponse {
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

impl UnaryService<FilesGetRequest> for FileInformation {
    type Response = FilesGetResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<FilesGetRequest>) -> Self::Future {
        Box::pin(async move {
            Ok(Response::new(FilesGetResponse {
                file: request.into_inner().file,
                chunk_bytes: 4 * 1024 * 1024,
                message_bytes: 8 * 1024 * 1024,
                resolved_at: Some(ReadCut {
                    positions: vec![Position {
                        history: Some(History {
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

impl UnaryService<FilesReadRequest> for Download {
    type Response = FilesReadResponse;
    type Future = BoxFuture<Response<Self::Response>, Status>;
    fn call(&mut self, request: Request<FilesReadRequest>) -> Self::Future {
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
            let mut reply = FilesReadResponse {
                data,
                total_length: original.len() as u64,
                sha256: Sha256::digest(&original).to_vec(),
                ..Default::default()
            };
            let position = reply.resolved_at.get_or_insert_default();
            let entry = Position {
                history: Some(History {
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
    (0..4 * 1024 * 1024 + 391).map(|index| (index % 251) as u8).collect()
}

/* === Private Functions === */

/* === Tests === */
