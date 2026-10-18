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
use std::process::{Command, Stdio};
// Third Party Imports
use crate::support::wire::WireMessage;
use serde_json::json;
// Local Imports
use crate::support::bridge::{Harness, Local};
use crate::support::{artifact, server::Fixture};
use dokime_domain::model;
use dokime_gui::rpc::Bridge;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/* === Tests === */
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn typescript_facade_bridge() {
    let fixture = Fixture::new().await;
    fixture.runtime.handle().set_brand(Some(
        dokime_runtime::domains::installed_plugins::load_brand(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../branding"),
        )
        .unwrap(),
    ));
    let artifact_id = artifact::seed(&fixture.runtime.handle(), &fixture.artifacts).await;
    let harness = Harness::new(fixture.session.clone());
    let snapshot = fixture.session.status();
    let mut child = Command::new("node")
        .env("DOKIME_TEST_GENERATION", &snapshot.generation)
        .env("DOKIME_TEST_BACKEND_ID", &snapshot.backend.as_ref().unwrap().backend_id)
        .env("DOKIME_TEST_ARTIFACT_ID", artifact_id)
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test-rpc-interop.mjs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let status = harness.drive(
        &mut child,
        |command, bytes| match command {
            "fixture_cancel_streams" => {
                use tauri::Manager;
                harness.app.state::<Bridge>().cancel_owner("main");
                Local::Reply(json!({"value": null}))
            }
            // These fixtures exercise TS-generated message semantics against canonical Harmos types.
            "fixture_value" => {
                let value = model::values::wire::Value::from_wire(bytes).unwrap();
                assert_eq!(value.nodes.len(), 5);
                let Some(model::values::wire::ValueKind::Object(fields)) = &value.nodes[0].kind else {
                    panic!("the fixture value has an object root");
                };
                assert_eq!(
                    fields.fields.iter().map(|field| (field.key.as_str(), field.value)).collect::<Vec<_>>(),
                    vec![("minimum", 1), ("maximum", 2), ("null", 3), ("unset", 4)]
                );
                assert!(matches!(value.nodes[1].kind, Some(model::values::wire::ValueKind::Integer(i64::MIN))));
                assert!(matches!(value.nodes[2].kind, Some(model::values::wire::ValueKind::Integer(i64::MAX))));
                assert!(matches!(value.nodes[3].kind, Some(model::values::wire::ValueKind::Null)));
                assert!(value.nodes[4].kind.is_none());
                Local::Reply(json!({ "bytes": base64(&value.wire_bytes()) }))
            }
            "fixture_admission" => {
                let value = model::execution::wire::Admission::from_wire(bytes).unwrap();
                assert_eq!(value.cursor.as_ref().unwrap().position, u64::MAX);
                assert_eq!(value.accepted_at.as_ref().unwrap().seconds, -1);
                assert_eq!(value.accepted_at.as_ref().unwrap().nanos, 999_999_999);
                Local::Reply(json!({ "bytes": base64(&value.wire_bytes()) }))
            }
            _ => Local::Dispatch,
        },
        |line| panic!("unexpected observation {line}"),
    );
    assert!(status.success(), "TypeScript/native interop failed");
    fixture.close().await;
}
