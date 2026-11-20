// Dokime — SPDX-License-Identifier: Apache-2.0
// Native Artifact Slices Use Product File Uploads.

/* === Imports === */

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{Bridge, Context, Error, read_u64};
use crate::connection::ConnectionSession;
use crate::rpc::protocol;
use dokime_protocol::host::content;
use dokime_protocol::host::files;
use sha2::{Digest, Sha256};
use tauri::{
    Runtime, State, WebviewWindow, async_runtime,
    ipc::{Request, Response},
};
use tokio::sync;
use tokio_util::sync::CancellationToken;

/* === Structs === */

#[derive(Clone, Default)]
pub(super) struct ArtifactUploads(Arc<Mutex<Registry>>);

#[derive(Default)]
struct Registry {
    next_id: u64,
    entries: HashMap<u64, Arc<Upload>>,
}

struct Upload {
    context: Arc<Context>,
    cancel: CancellationToken,
    writer: sync::Mutex<Writer>,
}

struct Writer {
    request: files::UploadsAppendRequest,
    artifact: content::ArtifactsCreateRequest,
    next_sequence: u64,
    digest: Sha256,
    closed: bool,
    chunk_bytes: usize,
}

/* === Implementations === */

impl ArtifactUploads {
    async fn open(&self, context: Context, artifact: content::ArtifactsCreateRequest) -> Result<Response, Error> {
        context.check()?;
        let context = Arc::new(context);
        let (id, upload) = {
            let mut registry = self.0.lock().expect("artifact uploads poisoned");
            if registry.entries.len() >= 2 {
                return Err(Error::LimitExceeded);
            }
            registry.next_id = registry.next_id.checked_add(1).ok_or(Error::LimitExceeded)?;
            let id = registry.next_id;
            let upload = Arc::new(Upload {
                context: context.clone(),
                cancel: CancellationToken::new(),
                writer: sync::Mutex::new(Writer {
                    request: Default::default(),
                    artifact,
                    next_sequence: 1,
                    digest: Sha256::new(),
                    closed: false,
                    chunk_bytes: 0,
                }),
            });
            registry.entries.insert(id, upload.clone());
            (id, upload)
        };
        let result = async {
            let client = context.client();
            let mut writer = upload.writer.lock().await;
            let mut request = files::UploadsCreateRequest::default();
            let metadata = request.metadata.get_or_insert_with(Default::default);
            metadata.id = format!("desktop-{}", writer.artifact.request_id);
            metadata.scope =
                writer.artifact.scope.as_ref().and_then(|scope| scope.execution_id.as_ref()).map(|id| format!("execution/{id}"));
            metadata.media_type = writer.artifact.content_type.clone();
            request.request_id = format!("desktop-{}-{id}-open", context.generation);
            let production: files::UploadsCreateResponse =
                context.execute(client.unary("/dokime.host.v1.files.Uploads/Create", request)).await?;
            writer.chunk_bytes = usize::try_from(production.chunk_bytes).map_err(|_| Error::LimitExceeded)?;
            writer.chunk_bytes =
                writer.chunk_bytes.min(usize::try_from(production.message_bytes.saturating_sub(1024)).map_err(|_| Error::LimitExceeded)?);
            if writer.chunk_bytes == 0 {
                return Err(Error::LimitExceeded);
            }
            writer.request.upload = production.upload;
            if writer.request.upload.is_none() {
                return Err(Error::InvalidRequest);
            }
            Ok::<_, Error>(())
        }
        .await;
        if let Err(error) = result {
            self.remove(&context.owner, context.generation, id);
            return Err(error);
        }
        let mut response = context.generation.to_le_bytes().to_vec();
        response.extend(id.to_le_bytes());
        Ok(Response::new(response))
    }

    fn get(&self, owner: &str, generation: u64, id: u64) -> Result<Arc<Upload>, Error> {
        self.0
            .lock()
            .expect("artifact uploads poisoned")
            .entries
            .get(&id)
            .filter(|upload| upload.context.owner == owner && upload.context.generation == generation)
            .cloned()
            .ok_or(Error::UnknownStream)
    }

    async fn write(&self, owner: &str, generation: u64, id: u64, sequence: u64, bytes: Vec<u8>) -> Result<(), Error> {
        if bytes.len() > protocol::UPLOAD_SLICE_BYTES as usize {
            return Err(Error::LimitExceeded);
        }
        let upload = self.get(owner, generation, id)?;
        let mut writer = upload.writer.lock().await;
        if writer.closed || sequence != writer.next_sequence {
            return Err(Error::InvalidAck);
        }
        upload.context.check()?;
        let client = upload.context.client();
        for chunk in bytes.chunks(writer.chunk_bytes) {
            let mut request = writer.request.clone();
            request.data = chunk.to_vec();
            request.request_id = format!("desktop-{generation}-{id}-{sequence}-{}", request.offset);
            let reply: files::UploadsAppendResponse = tokio::select! {
                biased;
                _ = upload.cancel.cancelled() => return Err(Error::Cancelled),
                reply = upload.context.execute(client.unary("/dokime.host.v1.files.Uploads/Append", request)) => reply?,
            };
            let production = reply.upload.ok_or(Error::InvalidRequest)?;
            let next = writer.request.offset.checked_add(chunk.len() as u64).ok_or(Error::LimitExceeded)?;
            if production.accepted_length != next {
                return Err(Error::InvalidAck);
            }
            writer.request.offset = next;
            writer.request.upload = Some(production);
        }
        writer.digest.update(&bytes);
        writer.next_sequence += 1;
        Ok(())
    }

    async fn finish(&self, owner: &str, generation: u64, id: u64) -> Result<Response, Error> {
        let upload = self.get(owner, generation, id)?;
        let mut writer = upload.writer.lock().await;
        if writer.closed {
            return Err(Error::UnknownStream);
        }
        writer.closed = true;
        let client = upload.context.client();
        let mut request = files::UploadsPublishRequest {
            upload: writer.request.upload.clone(),
            length: writer.request.offset,
            sha256: writer.digest.clone().finalize().to_vec(),
            request_id: String::new(),
        };
        request.request_id = format!("desktop-{generation}-{id}-publish");
        let published: files::UploadsPublishResponse =
            upload.context.execute(client.unary("/dokime.host.v1.files.Uploads/Publish", request)).await?;
        let mut request = writer.artifact.clone();
        request.file = published.upload;
        let artifact: content::ArtifactsCreateResponse =
            upload.context.execute(client.unary("/dokime.host.v1.content.Artifacts/Create", request)).await?;
        self.remove(owner, generation, id);
        upload.context.reply(artifact.artifact.ok_or(Error::InvalidRequest)?)
    }

    async fn cancel(&self, owner: &str, generation: u64, id: u64) -> Result<(), Error> {
        let upload = self.get(owner, generation, id)?;
        upload.cancel.cancel();
        let mut writer = upload.writer.lock().await;
        writer.closed = true;
        let mut request = files::UploadsDeleteRequest {
            upload: writer.request.upload.clone(),
            request_id: String::new(),
        };
        request.request_id = format!("desktop-{generation}-{id}-cancel");
        let client = upload.context.client();
        let result: Result<files::UploadsDeleteResponse, Error> =
            upload.context.execute(client.unary("/dokime.host.v1.files.Uploads/Delete", request)).await;
        self.remove(owner, generation, id);
        result.map(|_| ())
    }

    fn remove(&self, owner: &str, generation: u64, id: u64) {
        let mut registry = self.0.lock().expect("artifact uploads poisoned");
        if registry.entries.get(&id).is_some_and(|upload| upload.context.owner == owner && upload.context.generation == generation) {
            registry.entries.remove(&id);
        }
    }

    pub(super) fn cancel_owner(&self, owner: &str) {
        let uploads = {
            let mut registry = self.0.lock().expect("artifact uploads poisoned");
            let ids = registry.entries.iter().filter(|(_, upload)| upload.context.owner == owner).map(|(id, _)| *id).collect::<Vec<_>>();
            ids.into_iter().filter_map(|id| registry.entries.remove(&id).map(|upload| (id, upload))).collect::<Vec<_>>()
        };
        for (id, upload) in uploads {
            upload.cancel.cancel();
            async_runtime::spawn(async move {
                let mut writer = upload.writer.lock().await;
                writer.closed = true;
                if writer.request.upload.is_none() {
                    return;
                }
                let request = files::UploadsDeleteRequest {
                    upload: writer.request.upload.clone(),
                    request_id: format!("desktop-{}-{id}-abandon", upload.context.generation),
                };
                let client = upload.context.client();
                let result: Result<files::UploadsDeleteResponse, _> = client.unary("/dokime.host.v1.files.Uploads/Delete", request).await;
                if let Err(error) = result {
                    tracing::debug!(?error, "Abandon Artifact Upload: Delete File Production Failed");
                }
            });
        }
    }
}

/* === Public Functions === */

#[tauri::command]
pub async fn artifact_upload_open<R: Runtime>(
    session: State<'_, ConnectionSession>, bridge: State<'_, Bridge>, webview: WebviewWindow<R>, request: Request<'_>,
) -> Result<Response, Error> {
    Box::pin(async move {
        let (context, header) = bridge.prepare::<content::ArtifactsCreateRequest>(&session, webview.label(), request.body())?;
        bridge.uploads.open(context, header).await
    })
    .await
}

#[tauri::command]
pub async fn artifact_upload_write<R: Runtime>(
    session: State<'_, ConnectionSession>, bridge: State<'_, Bridge>, webview: WebviewWindow<R>, request: Request<'_>,
) -> Result<(), Error> {
    Box::pin(async move {
        let (generation, payload) = super::request(request.body(), protocol::FRAME_BYTES as usize)?;
        session.lease(&generation.to_string())?;
        if payload.len() < 16 {
            return Err(Error::InvalidRequest);
        }
        bridge.uploads.write(webview.label(), generation, read_u64(payload)?, read_u64(&payload[8..])?, payload[16..].to_vec()).await
    })
    .await
}

#[tauri::command]
pub async fn artifact_upload_finish<R: Runtime>(
    session: State<'_, ConnectionSession>, bridge: State<'_, Bridge>, webview: WebviewWindow<R>, request: Request<'_>,
) -> Result<Response, Error> {
    Box::pin(async move {
        let (generation, payload) = super::request(request.body(), protocol::FRAME_BYTES as usize)?;
        session.lease(&generation.to_string())?;
        if payload.len() != 8 {
            return Err(Error::InvalidRequest);
        }
        bridge.uploads.finish(webview.label(), generation, read_u64(payload)?).await
    })
    .await
}

#[tauri::command]
pub async fn artifact_upload_cancel<R: Runtime>(
    session: State<'_, ConnectionSession>, bridge: State<'_, Bridge>, webview: WebviewWindow<R>, request: Request<'_>,
) -> Result<(), Error> {
    Box::pin(async move {
        let (generation, payload) = super::request(request.body(), protocol::FRAME_BYTES as usize)?;
        session.lease(&generation.to_string())?;
        if payload.len() != 8 {
            return Err(Error::InvalidRequest);
        }
        bridge.uploads.cancel(webview.label(), generation, read_u64(payload)?).await
    })
    .await
}

/* === Tests === */
