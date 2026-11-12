/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.backend.collect_diagnostics";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::CollectBackendDiagnostics, v1::BackendCollectDiagnosticsOutput>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::CollectBackendDiagnostics,
) -> Result<super::Output<v1::BackendCollectDiagnosticsOutput>> {
    super::complete(ID, context.settings.read_timeout, async {
        use std::io::Write;
        use sha2::{Digest, Sha256};
        let identity = context.host.identity();
        let mut entries = Vec::new();
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        let metadata = serde_json::json!({"backend_id": identity.backend_id, "epoch": identity.epoch, "product_version": identity.product_version, "api": "dokime.v1"});
        let bytes = serde_json::to_vec(&metadata).map_err(|_| Error::Invalid("Collect Diagnostics: Retry After Restarting the Host".into()))?;
        writer.start_file("backend.json", options).map_err(|error| Error::Invalid(error.to_string()))?;
        writer.write_all(&bytes).map_err(|error| Error::Invalid(error.to_string()))?;
        entries.push(v1::DiagnosticsBundleEntry { path: "backend.json".into(), size_bytes: bytes.len() as u64, present: true, ..Default::default() });
        let (events, _, _) = context.host.diagnostic_events(None)?;
        let budget = request.max_log_tail_bytes.unwrap_or(context.settings.message_bytes as u64).min(context.settings.message_bytes as u64) as usize;
        let mut lines = Vec::new();
        let mut used = 0;
        for event in events.into_iter().rev() {
            let line = serde_json::to_vec(&serde_json::json!({"sequence": event.sequence,"level": event.level,"target":event.target,"message":event.message,"fields":event.fields})).map_err(|error| Error::Invalid(error.to_string()))?;
            if used + line.len() + 1 > budget { break; }
            used += line.len() + 1;
            lines.push(line);
        }
        lines.reverse();
        let mut logs = Vec::new();
        for line in lines { logs.extend(line); logs.push(b'\n'); }
        writer.start_file("logs.jsonl", options).map_err(|error| Error::Invalid(error.to_string()))?;
        writer.write_all(&logs).map_err(|error| Error::Invalid(error.to_string()))?;
        entries.push(v1::DiagnosticsBundleEntry { path: "logs.jsonl".into(), size_bytes: logs.len() as u64, present: true, ..Default::default() });
        let bytes = writer.finish().map_err(|error| Error::Invalid(error.to_string()))?.into_inner();
        let size_bytes = bytes.len() as u64;
        let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let header = v1::DiagnosticsBundleHeader { filename: "dokime-diagnostics.zip".into(), content_type: "application/zip".into(), generated_at: Some(super::projection::timestamp(context.host.now_ns())), total_size_bytes: Some(size_bytes), entries };
        let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        let chunk_bytes = context.settings.message_bytes / 2;
        let timeout = context.settings.read_timeout;
        tokio::spawn(async move {
            let work = async {
                let frame = v1::BackendCollectDiagnosticsOutput { value: Some(v1::backend_collect_diagnostics_output::Value::Item(v1::DiagnosticsBundleFrame { payload: Some(v1::diagnostics_bundle_frame::Payload::Header(header)) })) };
                if sender.send(Ok(frame)).await.is_err() { return; }
                for (index, chunk) in bytes.chunks(chunk_bytes).enumerate() {
                    let frame = v1::BackendCollectDiagnosticsOutput { value: Some(v1::backend_collect_diagnostics_output::Value::Item(v1::DiagnosticsBundleFrame { payload: Some(v1::diagnostics_bundle_frame::Payload::Chunk(v1::DiagnosticsBundleChunk { offset: (index * chunk_bytes) as u64, data: chunk.to_vec() })) })) };
                    if sender.send(Ok(frame)).await.is_err() { return; }
                }
                let frame = v1::BackendCollectDiagnosticsOutput { value: Some(v1::backend_collect_diagnostics_output::Value::Item(v1::DiagnosticsBundleFrame { payload: Some(v1::diagnostics_bundle_frame::Payload::Complete(v1::DiagnosticsBundleCompletion { size_bytes, sha256: digest })) })) };
                if sender.send(Ok(frame)).await.is_err() { return; }
                let _ = sender.send(Ok(v1::BackendCollectDiagnosticsOutput { value: Some(v1::backend_collect_diagnostics_output::Value::Response(v1::BackendCollectDiagnosticsCompletion {})) })).await;
            };
            if tokio::time::timeout(timeout, work).await.is_err() { let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("Collect Diagnostics: Retry After the Transfer Deadline"))); }
        });
        Ok(super::Output::new(receiver))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
