/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Third Party Imports
use prost::Message;
use sha2::{Digest, Sha256};
use tauri::Manager;

// Local Imports
use super::invoke;
use crate::support::{
    bridge::{Harness, fenced},
    server::Fixture,
};
use dokime_gui::rpc::Bridge;
use dokime_sdk::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn header(size_bytes: u64) -> v1::UploadArtifactHeader {
    v1::UploadArtifactHeader {
        scope: Some(v1::ArtifactScope {
            cycle_id: Some("desktop-cycle".into()),
            execution_id: Some("desktop-content".into()),
            ..Default::default()
        }),
        filename: "upload-sink-fixture.bin".into(),
        content_type: Some("application/octet-stream".into()),
        size_bytes,
        ..Default::default()
    }
}

fn open(harness: &Harness, generation: u64, size_bytes: u64) -> u64 {
    let reply = invoke(&harness.window, "artifact_upload_open", fenced(generation, &header(size_bytes).encode_to_vec())).unwrap();
    let tauri::ipc::InvokeResponseBody::Raw(handle) = reply else {
        panic!("Upload Returns a Native Handle");
    };
    assert_eq!(handle.len(), 16);
    assert_eq!(u64::from_le_bytes(handle[..8].try_into().unwrap()), generation);
    u64::from_le_bytes(handle[8..].try_into().unwrap())
}

fn write_payload(id: u64, sequence: u64, data: &[u8]) -> Vec<u8> {
    let mut bytes = id.to_le_bytes().to_vec();
    bytes.extend(sequence.to_le_bytes());
    bytes.extend(data);
    bytes
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_sink_completes() {
    // Upload Into an Open Imported Evidence Container
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let id = open(&harness, generation, 6);
    invoke(&harness.window, "artifact_upload_write", fenced(generation, &write_payload(id, 1, b"foo"))).unwrap();
    invoke(&harness.window, "artifact_upload_write", fenced(generation, &write_payload(id, 2, b"bar"))).unwrap();

    // Finish Requires the Host's Verified Completion and Returns Canonical Metadata
    let finished = invoke(&harness.window, "artifact_upload_finish", fenced(generation, &id.to_le_bytes())).unwrap();
    let tauri::ipc::InvokeResponseBody::Raw(metadata) = finished else {
        panic!("Upload Finishes with Canonical Metadata");
    };
    let metadata = v1::ArtifactMetadata::decode(&metadata[8..]).unwrap();
    assert_eq!(metadata.size_bytes, 6);
    assert_eq!(metadata.sha256, Sha256::digest(b"foobar").iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    assert_eq!(metadata.state, v1::ArtifactState::Complete as i32);
    harness.app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_refuses_gap() {
    // A Skipped Native Sequence Cannot Publish an Artifact
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let id = open(&harness, generation, 3);
    let error = invoke(&harness.window, "artifact_upload_write", fenced(generation, &write_payload(id, 2, b"foo"))).unwrap_err();
    assert_eq!(error["kind"], "invalid_ack");
    harness.app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_owner_cancels() {
    // Releasing the WebView Invalidates Every Owned Upload
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let id = open(&harness, generation, 3);
    harness.app.state::<Bridge>().cancel_owner("main");
    let error = invoke(&harness.window, "artifact_upload_write", fenced(generation, &write_payload(id, 1, b"foo"))).unwrap_err();
    assert_eq!(error["kind"], "unknown_stream");
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upload_sink_opens() {
    // Native Open Returns Before Input Has Finished
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    assert!(open(&harness, fixture.generation(), 1) > 0);
    harness.app.state::<Bridge>().cancel_owner("main");
    fixture.close().await;
}
