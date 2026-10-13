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
// Standard Library Imports
use std::io::{Cursor, Read};
// Third Party Imports
use sha2::{Digest, Sha256};
// Local Imports
use super::{WireClient, authenticated, contracts};
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
pub(super) async fn download(client: &WireClient, cycle_id: &str) -> Vec<u8> {
    let mut stream = client
        .download::<contracts::content::DownloadCycleBundle>(authenticated(
            contracts::content::DownloadCycleBundleRequest {
                cycle_id: cycle_id.into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let metadata = match stream.message().await.unwrap().unwrap().payload.unwrap() {
        contracts::content::CycleBundleFramePayload::Metadata(metadata) => metadata,
        frame => panic!("expected bundle metadata: {frame:?}"),
    };
    assert_eq!(metadata.cycle_id, cycle_id);
    assert_eq!(metadata.filename, format!("{cycle_id}.zip"));
    assert_eq!(metadata.content_type, "application/zip");
    let mut bytes = Vec::new();
    let completion = loop {
        let frame = stream.message().await.unwrap().expect("bundle completion");
        match frame.payload.unwrap() {
            contracts::content::CycleBundleFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, bytes.len() as u64);
                assert!(!chunk.data.is_empty());
                assert!(chunk.data.len() <= 64 * 1024);
                bytes.extend_from_slice(&chunk.data);
            }
            contracts::content::CycleBundleFramePayload::Complete(completion) => break completion,
            frame => panic!("unexpected second metadata: {frame:?}"),
        }
    };
    assert!(
        stream.message().await.unwrap().is_none(),
        "completion requires final OK status"
    );
    assert_eq!(completion.size_bytes, bytes.len() as u64);
    assert_eq!(completion.sha256, digest(&bytes));
    assert_eq!(metadata.sha256, completion.sha256);
    assert_eq!(metadata.size_bytes, completion.size_bytes);
    bytes
}

pub(super) fn verify(bytes: &[u8], cycle_id: &str) -> Vec<u8> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let manifest: serde_json::Value =
        serde_json::from_reader(zip.by_name("manifest.json").unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["cycle_id"], cycle_id);
    let entries = manifest["entries"].as_array().unwrap();
    assert_eq!(
        entries.len() + 1,
        zip.len(),
        "every payload must appear in manifest"
    );
    let mut previous = "";
    for entry in entries {
        let path = entry["path"].as_str().unwrap();
        assert!(previous < path, "manifest paths must be unique and sorted");
        previous = path;
        let mut stored = zip.by_name(path).unwrap();
        assert_eq!(stored.compression(), zip::CompressionMethod::Stored);
        assert!(stored.enclosed_name().is_some());
        let mut payload = Vec::new();
        stored.read_to_end(&mut payload).unwrap();
        assert_eq!(entry["size_bytes"], payload.len() as u64);
        assert_eq!(entry["sha256"], digest(&payload));
    }
    assert!(
        entries
            .iter()
            .any(|entry| entry["path"].as_str().unwrap().ends_with("/notes.json"))
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry["path"].as_str().unwrap().ends_with("/document.json"))
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry["path"].as_str().unwrap().contains("/records/"))
    );
    let mut report = Vec::new();
    zip.by_name("reports/final.html")
        .unwrap()
        .read_to_end(&mut report)
        .unwrap();
    report
}

/* === Private Functions === */
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
/* === Tests === */
