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
use std::process::Stdio;

// Local Imports
use crate::support::{
    bridge::{Harness, Local},
    server::Fixture,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn typescript_native_bridge() {
    // Start the Real Host and Dispatch Every Frontend Request Through Tauri
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let snapshot = fixture.session.status();
    let mut child = dokime_runtime_next::process::command("node")
        .env("DOKIME_TEST_GENERATION", snapshot.generation)
        .env("DOKIME_TEST_BACKEND_ID", snapshot.backend.unwrap().backend_id)
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-rpc-interop.mjs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let status = harness.drive(&mut child, |_, _| Local::Dispatch, |line| panic!("Unexpected Observation: {line}"));
    fixture.close().await;
    assert!(status.success(), "TypeScript Native Interop Failed");
}
