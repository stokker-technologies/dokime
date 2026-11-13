// Dokime — SPDX-License-Identifier: Apache-2.0
// Native Artifact Slices Use Library-Owned File Production.

/* === Imports === */

use super::*;
use crate::rpc::protocol::{FRAME_BYTES as DESKTOP_FRAME_BYTES, UPLOAD_SLICE_BYTES as DESKTOP_UPLOAD_SLICE_BYTES};
use dokime_protocol::client::content;
use dokime_sdk::client::v1 as library;
use sha2::{Digest, Sha256};

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
    writer: tokio::sync::Mutex<Writer>,
}

struct Writer {
    request: library::StorageAppendFileBytesRequest,
    artifact: content::ArtifactsCreateRequest,
    next_sequence: u64,
    digest: Sha256,
    closed: bool,
}

/* === Implementations === */

impl ArtifactUploads {
    async fn open(&self, context: Context, artifact: content::ArtifactsCreateRequest) -> Result<tauri::ipc::Response, RpcError> {
        context.check()?;
        let context = Arc::new(context);
        let (id, upload) = {
            let mut registry = self.0.lock().expect("artifact uploads poisoned");
            if registry.entries.len() >= 2 {
                return Err(RpcError::LimitExceeded);
            }
            registry.next_id = registry.next_id.checked_add(1).ok_or(RpcError::LimitExceeded)?;
            let id = registry.next_id;
            let upload = Arc::new(Upload {
                context: context.clone(),
                cancel: CancellationToken::new(),
                writer: tokio::sync::Mutex::new(Writer {
                    request: Default::default(),
                    artifact,
                    next_sequence: 1,
                    digest: Sha256::new(),
                    closed: false,
                }),
            });
            registry.entries.insert(id, upload.clone());
            (id, upload)
        };
        let result = async {
            let client = context.client();
            let capabilities: library::StorageCapabilitiesResponse =
                context.execute(client.unary("/harmos.v1.Storage/Capabilities", library::StorageCapabilitiesRequest {})).await?;
            let window = capabilities.mutations.ok_or(RpcError::InvalidRequest)?;
            let mut writer = upload.writer.lock().await;
            let mut request = library::StorageCreateFileProductionRequest::default();
            let metadata = request.metadata.get_or_insert_with(Default::default);
            metadata.id = format!("desktop-{}", writer.artifact.request_id);
            metadata.scope = writer.artifact.scope.as_ref().and_then(|scope| scope.execution_id.clone());
            metadata.media_type = writer.artifact.content_type.clone();
            let mutation = request.mutation.get_or_insert_with(Default::default);
            mutation.window = window.id.clone();
            mutation.key = format!("desktop-{}-{id}-open", context.generation);
            let production: library::StorageCreateFileProductionResponse =
                context.execute(client.unary("/harmos.v1.Storage/CreateFileProduction", request)).await?;
            writer.request.session = production.production.and_then(|value| value.session);
            if writer.request.session.is_none() {
                return Err(RpcError::InvalidRequest);
            }
            let mutation = writer.request.mutation.get_or_insert_with(Default::default);
            mutation.window = window.id;
            Ok::<_, RpcError>(())
        }
        .await;
        if let Err(error) = result {
            self.remove(&context.owner, context.generation, id);
            return Err(error);
        }
        let mut response = context.generation.to_le_bytes().to_vec();
        response.extend(id.to_le_bytes());
        Ok(tauri::ipc::Response::new(response))
    }

    fn get(&self, owner: &str, generation: u64, id: u64) -> Result<Arc<Upload>, RpcError> {
        self.0
            .lock()
            .expect("artifact uploads poisoned")
            .entries
            .get(&id)
            .filter(|upload| upload.context.owner == owner && upload.context.generation == generation)
            .cloned()
            .ok_or(RpcError::UnknownStream)
    }

    async fn write(&self, owner: &str, generation: u64, id: u64, sequence: u64, bytes: Vec<u8>) -> Result<(), RpcError> {
        if bytes.len() > DESKTOP_UPLOAD_SLICE_BYTES as usize {
            return Err(RpcError::LimitExceeded);
        }
        let upload = self.get(owner, generation, id)?;
        let mut writer = upload.writer.lock().await;
        if writer.closed || sequence != writer.next_sequence {
            return Err(RpcError::InvalidAck);
        }
        upload.context.check()?;
        let mut request = writer.request.clone();
        request.data = bytes.clone();
        request.mutation.as_mut().ok_or(RpcError::InvalidRequest)?.key = format!("desktop-{generation}-{id}-{sequence}");
        let client = upload.context.client();
        let reply: library::StorageAppendFileBytesResponse = tokio::select! {
            biased;
            _ = upload.cancel.cancelled() => return Err(RpcError::Cancelled),
            reply = upload.context.execute(client.unary("/harmos.v1.Storage/AppendFileBytes", request)) => reply?,
        };
        let production = reply.production.ok_or(RpcError::InvalidRequest)?;
        let next = writer.request.offset.checked_add(bytes.len() as u64).ok_or(RpcError::LimitExceeded)?;
        if production.length != next {
            return Err(RpcError::InvalidAck);
        }
        writer.request.offset = next;
        writer.request.session = production.session;
        writer.digest.update(&bytes);
        writer.next_sequence += 1;
        Ok(())
    }

    async fn finish(&self, owner: &str, generation: u64, id: u64) -> Result<tauri::ipc::Response, RpcError> {
        let upload = self.get(owner, generation, id)?;
        let mut writer = upload.writer.lock().await;
        if writer.closed {
            return Err(RpcError::UnknownStream);
        }
        writer.closed = true;
        let client = upload.context.client();
        let mut request = library::StoragePublishFileProductionRequest {
            session: writer.request.session.clone(),
            length: writer.request.offset,
            sha256: writer.digest.clone().finalize().to_vec(),
            mutation: writer.request.mutation.clone(),
        };
        request.mutation.as_mut().ok_or(RpcError::InvalidRequest)?.key = format!("desktop-{generation}-{id}-publish");
        let published: library::StoragePublishFileProductionResponse =
            upload.context.execute(client.unary("/harmos.v1.Storage/PublishFileProduction", request)).await?;
        let mut request = writer.artifact.clone();
        request.file = published.file;
        let artifact: content::ArtifactsCreateResponse =
            upload.context.execute(client.unary("/dokime.client.v1.content.Artifacts/Create", request)).await?;
        self.remove(owner, generation, id);
        upload.context.reply(artifact.artifact.ok_or(RpcError::InvalidRequest)?)
    }

    async fn cancel(&self, owner: &str, generation: u64, id: u64) -> Result<(), RpcError> {
        let upload = self.get(owner, generation, id)?;
        upload.cancel.cancel();
        let mut writer = upload.writer.lock().await;
        writer.closed = true;
        let mut request = library::StorageDeleteFileProductionRequest {
            session: writer.request.session.clone(),
            mutation: writer.request.mutation.clone(),
        };
        request.mutation.as_mut().ok_or(RpcError::InvalidRequest)?.key = format!("desktop-{generation}-{id}-cancel");
        let client = upload.context.client();
        let result: Result<library::StorageDeleteFileProductionResponse, RpcError> =
            upload.context.execute(client.unary("/harmos.v1.Storage/DeleteFileProduction", request)).await;
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
            tauri::async_runtime::spawn(async move {
                let mut writer = upload.writer.lock().await;
                writer.closed = true;
                if writer.request.session.is_none() {
                    return;
                }
                let mut mutation = writer.request.mutation.clone();
                if let Some(mutation) = mutation.as_mut() {
                    mutation.key = format!("desktop-{}-{id}-abandon", upload.context.generation);
                }
                let request = library::StorageDeleteFileProductionRequest {
                    session: writer.request.session.clone(),
                    mutation,
                };
                let client = upload.context.client();
                let result: Result<library::StorageDeleteFileProductionResponse, _> =
                    client.unary("/harmos.v1.Storage/DeleteFileProduction", request).await;
                if let Err(error) = result {
                    tracing::debug!(?error, "Abandon Artifact Upload: Delete File Production Failed");
                }
            });
        }
    }
}

/* === Public Functions === */

#[tauri::command]
pub async fn artifact_upload_open<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<tauri::ipc::Response, RpcError> {
    Box::pin(async move {
        let (context, header) =
            bridge.prepare::<dokime_protocol::client::content::ArtifactsCreateRequest>(&session, webview.label(), request.body())?;
        bridge.uploads.open(context, header).await
    })
    .await
}

#[tauri::command]
pub async fn artifact_upload_write<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    Box::pin(async move {
        let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
        session.lease(&generation.to_string())?;
        if payload.len() < 16 {
            return Err(RpcError::InvalidRequest);
        }
        bridge.uploads.write(webview.label(), generation, read_u64(payload)?, read_u64(&payload[8..])?, payload[16..].to_vec()).await
    })
    .await
}

#[tauri::command]
pub async fn artifact_upload_finish<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<tauri::ipc::Response, RpcError> {
    Box::pin(async move {
        let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
        session.lease(&generation.to_string())?;
        if payload.len() != 8 {
            return Err(RpcError::InvalidRequest);
        }
        bridge.uploads.finish(webview.label(), generation, read_u64(payload)?).await
    })
    .await
}

#[tauri::command]
pub async fn artifact_upload_cancel<R: tauri::Runtime>(
    session: tauri::State<'_, crate::connection::ConnectionSession>, bridge: tauri::State<'_, Bridge>, webview: tauri::WebviewWindow<R>,
    request: tauri::ipc::Request<'_>,
) -> Result<(), RpcError> {
    Box::pin(async move {
        let (generation, payload) = super::request(request.body(), DESKTOP_FRAME_BYTES as usize)?;
        session.lease(&generation.to_string())?;
        if payload.len() != 8 {
            return Err(RpcError::InvalidRequest);
        }
        bridge.uploads.cancel(webview.label(), generation, read_u64(payload)?).await
    })
    .await
}

/* === Tests === */
